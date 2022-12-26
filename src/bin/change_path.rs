use self::schema::music::dsl::*;
use diesel::prelude::*;
use music_id::*;
use std::io::stdin;
use std::path::Path;

fn main() -> Result<(), String> {
    let mut piece_id = String::new();
    let mut new_path = String::new();

    println!("Enter piece ID:");

    make_string_err!(stdin().read_line(&mut piece_id))?;
    let piece_id = make_string_err!(piece_id.trim_end().parse::<i32>())?;

    println!("Enter the new path:");
    make_string_err!(stdin().read_line(&mut new_path))?;
    let new_path = new_path.trim_end();

    if !Path::new(new_path).exists() {
        return Err("File does not exist".to_owned());
    }

    let mut connection = make_string_err!(establish_connection())?;

    let res = make_string_err!(diesel::update(music.filter(id.eq(piece_id)))
        .set(file_path.eq(new_path))
        .execute(&mut connection))?;

    println!("Updated piece: {} rows updated", res);

    Ok(())
}
