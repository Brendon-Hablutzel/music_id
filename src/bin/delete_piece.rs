use diesel::prelude::*;
use music_id::schema::music::dsl::*;
use music_id::{establish_connection, make_string_err};
use std::env::args;

fn main() -> Result<(), String> {
    let piece_id = make_string_err!(args().nth(1).ok_or("Piece ID required")?.parse::<i32>())?;

    let mut connection = make_string_err!(establish_connection())?;

    let res =
        make_string_err!(diesel::delete(music.filter(id.eq(piece_id))).execute(&mut connection))?;

    println!("Attempted to delete post {piece_id}: {res} rows affected");

    Ok(())
}
