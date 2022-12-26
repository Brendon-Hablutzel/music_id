use actix_files::NamedFile;
use actix_web::{get, http::StatusCode, web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use diesel::prelude::*;
use music_id::models::Piece;
use music_id::schema::music::dsl::*;
use music_id::{
    establish_connection, query, to_client_err, to_server_err, AppErrors, ErrorTemplate,
    QuizTemplate,
};
use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{fs, io, path::Path};

async fn media_full(path: web::Path<String>) -> actix_web::Result<NamedFile> {
    let target_title = path.into_inner();

    let pieces = to_server_err!(query!(
        filter title.eq(target_title)
    ))?;

    let piece = to_client_err!(pieces.get(0).ok_or("No entries found"))?;

    let piece_path = Path::new(&piece.file_path);
    Ok(NamedFile::open(piece_path)?)
}

async fn media_partial(path: web::Path<(String, u32, u32)>) -> actix_web::Result<impl Responder> {
    let (target_title, start, end) = path.into_inner();

    // perhaps due to the difficulties in determining the duration of an mp3 file,
    // if the first byte is included, the duration of the entire file is
    // displayed in the browser, rather than the length of the partial segment
    let start = if start == 0 { 1 } else { start };

    if start > end {
        return to_client_err!("Invalid byte range: end cannot be greater than start");
    }

    let pieces = to_server_err!(query!(
        filter title.like(format!("%{target_title}%"))
    ))?;

    let piece = to_client_err!(pieces.get(0).ok_or("Piece not found"))?;

    if end > piece.file_size {
        return to_client_err!("Invalid byte range: end too large");
    }

    let piece_path = Path::new(&piece.file_path);

    let content = to_server_err!(fs::read(piece_path))?;
    let content = (&content[start as usize..end as usize]).to_vec();

    Ok(HttpResponse::Ok().content_type("audio/mpeg").body(content))
}

async fn list_by_artist(path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let artist_name = path.into_inner();

    let pieces = to_server_err!(query!(
        filter artist.eq(format!("{artist_name}"))
    ))?;

    Ok(web::Json(pieces))
}

async fn list_by_title(path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let piece_title = path.into_inner();

    let pieces = to_server_err!(query!(
        filter title.like(format!("%{piece_title}%"))
    ))?;

    Ok(web::Json(pieces))
}

async fn list_all() -> actix_web::Result<impl Responder> {
    let pieces = to_server_err!(query!())?;

    Ok(web::Json(pieces))
}

async fn list_random(path: web::Path<u32>) -> actix_web::Result<impl Responder> {
    let number = path.into_inner();

    sql_function!(fn rand() -> Text);

    let pieces = to_server_err!(query!(
        order rand(),
        limit number.into()
    ))?;

    Ok(web::Json(pieces))
}

#[get("/quiz")]
async fn quiz() -> actix_web::Result<impl Responder> {
    sql_function!(fn rand() -> Text);

    let pieces = to_server_err!(query!(
        order rand(),
        limit 4
    ))?;

    if pieces.len() != 4 {
        return to_client_err!("Invalid number of pieces fetched");
    }

    let pieces = pieces.get(0..4).unwrap();
    let correct = pieces.choose(&mut thread_rng()).unwrap();
    let chunk_size = thread_rng().gen_range(correct.file_size / 20..correct.file_size / 10);
    let start = thread_rng().gen_range(0..correct.file_size - chunk_size);
    let correct_url = &format!(
        "/media/partial/{}/{}/{}",
        correct.title,
        start,
        start + chunk_size
    );

    let quiz_template = QuizTemplate {
        pieces: pieces.try_into().unwrap(),
        correct_piece: correct,
        correct_url,
    }
    .render()
    .unwrap();

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(quiz_template))
}

async fn not_found() -> impl Responder {
    let status_code = StatusCode::NOT_FOUND;
    let error_template = ErrorTemplate {
        code: status_code,
        text: "Page Not Found",
    }
    .render()
    .expect("Failed to render err template");

    HttpResponse::build(status_code)
        .content_type("text/html")
        .body(error_template)
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    println!("Starting server...");
    HttpServer::new(|| {
        App::new()
            .service(
                web::scope("/media")
                    .route("/full/{title}", web::get().to(media_full))
                    .route(
                        "/partial/{title}/{start}/{end}",
                        web::get().to(media_partial),
                    ),
            )
            .service(
                web::scope("/list")
                    .route("/all", web::get().to(list_all))
                    .route("/by_artist/{artist_name}", web::get().to(list_by_artist))
                    .route("/by_title/{title}", web::get().to(list_by_title))
                    .route("/random/{number}", web::get().to(list_random)),
            )
            .service(quiz)
            .default_service(web::route().to(not_found))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
