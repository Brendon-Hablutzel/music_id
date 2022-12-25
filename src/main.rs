use actix_files::NamedFile;
use actix_web::{http::header::ContentType, web, App, HttpResponse, HttpServer, Responder};
use diesel::prelude::*;
use music_id::{establish_connection, models::Piece, AppErrors, to_server_err, to_client_err};
use std::{fs, io, path::Path};
use music_id::schema::music::dsl::*;

async fn media_full(path: web::Path<String>) -> actix_web::Result<NamedFile> {
    let target_title = path.into_inner();

    let mut connection = to_server_err!(establish_connection())?;

    let pieces: Vec<Piece> = to_server_err!(music
        .filter(title.like(format!("%{target_title}%")))
        .load::<Piece>(&mut connection))?;

    let piece = to_client_err!(pieces.get(1).ok_or("No entries found"))?;

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

    let mut connection = to_server_err!(establish_connection())?;

    let piece: Vec<Piece> = to_server_err!(music
        .filter(title.like(format!("%{target_title}%")))
        .load::<Piece>(&mut connection))?;

    let piece = to_client_err!(piece.get(0).ok_or("Piece not found"))?;

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

    let mut connection = to_server_err!(establish_connection())?;

    let pieces: Vec<Piece> = to_server_err!(music
        .filter(artist.eq(format!("{artist_name}")))
        .load::<Piece>(&mut connection))?;

    Ok(web::Json(pieces))
}

async fn list_by_title(path: web::Path<String>) -> actix_web::Result<impl Responder> {
    let piece_title = path.into_inner();

    let mut connection = to_server_err!(establish_connection())?;

    let pieces: Vec<Piece> = to_server_err!(music
        .filter(title.like(format!("%{piece_title}%")))
        .load::<Piece>(&mut connection))?;

    Ok(web::Json(pieces))
}

async fn list_all() -> actix_web::Result<impl Responder> {
    let mut connection = to_server_err!(establish_connection())?;

    let pieces = to_server_err!(music.load::<Piece>(&mut connection))?;

    Ok(web::Json(pieces))
}

async fn list_random(path: web::Path<u32>) -> actix_web::Result<impl Responder> {
    let number = path.into_inner();
    let mut connection = to_server_err!(establish_connection())?;

    sql_function!(fn rand() -> Text);

    let pieces = to_server_err!(music
        .order(rand())
        .limit(number.into())
        .load::<Piece>(&mut connection))?;

    Ok(web::Json(pieces))
}

async fn not_found() -> impl Responder {
    HttpResponse::NotFound()
        .content_type(ContentType::html())
        .body("<h1>Error 404</h1>")
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
            .default_service(web::route().to(not_found))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
