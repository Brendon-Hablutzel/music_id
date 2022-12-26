use serde::Serialize;
use std::fmt::Display;

use crate::schema::music;
use diesel::prelude::*;

#[derive(Queryable, Serialize, Clone)]
pub struct Piece {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub file_path: String,
    pub file_size: u32,
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: {} by {} at {}",
            self.id, self.title, self.artist, self.file_path
        )
    }
}

impl PartialEq for Piece {
    fn eq(&self, other: &Self) -> bool {
        if self.id == other.id {
            true
        } else {
            false
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[derive(Insertable)]
#[diesel(table_name = music)]
pub struct NewPiece<'a> {
    pub title: &'a str,
    pub artist: &'a str,
    pub file_path: &'a str,
    pub file_size: u32, // max size equivalent to ~4.29 GB
}
