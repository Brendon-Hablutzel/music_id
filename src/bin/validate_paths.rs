use std::path::Path;

use diesel::prelude::*;
use music_id::models::Piece;
use music_id::schema::music::dsl::*;
use music_id::{establish_connection, make_string_err};

fn main() -> Result<(), String> {
    println!("Searching for invalid file paths...");

    let mut connection = make_string_err!(establish_connection())?;

    let pieces = make_string_err!(music.load::<Piece>(&mut connection))?;

    let invalid_pieces = pieces.iter().filter_map(|piece| {
        let piece_path = Path::new(&piece.file_path);
        if piece_path.exists() {
            None
        } else {
            Some(piece)
        }
    });

    for piece in invalid_pieces {
        println!("Invalid piece: {piece}");
    }

    println!("Done");

    Ok(())
}
