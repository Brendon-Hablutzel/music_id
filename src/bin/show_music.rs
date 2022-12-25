use diesel::prelude::*;
use music_id::models::Piece;
use music_id::schema::music::dsl::*;
use music_id::{establish_connection, return_string_err};

fn main() -> Result<(), String> {
    let mut connection = return_string_err!(establish_connection());

    let results = return_string_err!(music.load::<Piece>(&mut connection));

    println!("Displaying {} pieces", results.len());
    for piece in results {
        println!(
            "{}: {} by {} at {}",
            piece.id, piece.title, piece.artist, piece.file_path
        );
    }

    Ok(())
}
