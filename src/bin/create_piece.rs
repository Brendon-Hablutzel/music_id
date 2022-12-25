use music_id::*;
use std::{io::stdin, path::Path};

fn main() -> Result<(), String> {
    let mut connection = return_string_err!(establish_connection());

    let mut name = String::new();
    let mut artist = String::new();
    let mut path = String::new();

    println!("Name:");
    return_string_err!(stdin().read_line(&mut name));
    let name = name.trim_end();

    println!("Artist:");
    return_string_err!(stdin().read_line(&mut artist));
    let artist = artist.trim_end();

    println!("Path:");
    return_string_err!(stdin().read_line(&mut path));
    let path = path.trim_end();

    if !Path::new(path).exists() {
        return Err("File does not exist".to_owned());
    }

    let res = return_string_err!(create_piece(&mut connection, name, artist, path));
    println!("Created piece: {res} rows affected");

    Ok(())
}
