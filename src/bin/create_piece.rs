use diesel::prelude::*;
use music_id::{models::NewPiece, schema::music, *};
use std::{io::stdin, path::Path};

fn main() -> Result<(), String> {
    let mut connection = make_string_err!(establish_connection())?;

    let mut title = String::new();
    let mut artist = String::new();
    let mut file_path = String::new();

    println!("Title:");
    make_string_err!(stdin().read_line(&mut title))?;
    let title = title.trim_end();

    println!("Artist:");
    make_string_err!(stdin().read_line(&mut artist))?;
    let artist = artist.trim_end();

    println!("Path:");
    make_string_err!(stdin().read_line(&mut file_path))?;
    let file_path = file_path.trim_end();

    let path = Path::new(file_path);

    if !path.exists() {
        return Err("File does not exist".to_owned());
    }

    let file_size = make_string_err!(path.metadata())?.len() as u32;

    let new_piece = NewPiece {
        title,
        artist,
        file_path,
        file_size,
    };

    make_string_err!(diesel::insert_into(music::table)
        .values(&new_piece)
        .execute(&mut connection))?;

    Ok(())
}
