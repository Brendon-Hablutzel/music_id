use actix_web::{error, http::StatusCode, HttpResponse};
use askama::Template;
use diesel::{mysql::MysqlConnection, prelude::*};
use dotenvy::dotenv;
use models::NewPiece;
use schema::music;
use std::{env, fmt::Display, path::Path};

pub mod models;
pub mod schema;

#[macro_export]
macro_rules! return_string_err {
    ($fallible:expr) => {
        match $fallible {
            Ok(success) => success,
            Err(failure) => return Err(failure.to_string()),
        }
    };
}

#[macro_export]
macro_rules! to_client_err {
    // ($fallible:expr, $message:literal) => {
    //     match $fallible {
    //         Ok(success) => Ok(success),
    //         Err(err) => Err(AppErrors::ClientError($message.to_string()))
    //     }
    // };
    ($text:literal) => {
        Err(AppErrors::ClientError($text.to_owned()).into())
    };
    ($fallible:expr) => {
        match $fallible {
            Ok(success) => Ok(success),
            Err(err) => Err(AppErrors::ClientError(err.to_string())),
        }
    };
}

#[macro_export]
macro_rules! to_server_err {
    ($fallible:expr) => {
        match $fallible {
            Ok(success) => Ok(success),
            Err(err) => {
                eprintln!("INTERNAL SERVER ERROR: {}", err.to_string());
                Err(AppErrors::ServerError)
            }
        }
    };
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    pub code: StatusCode,
    pub text: &'a str,
}

#[derive(Debug)]
pub enum AppErrors {
    ServerError,
    ClientError(String),
}

impl Display for AppErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppErrors::ServerError => write!(f, "Internal Server Error"),
            AppErrors::ClientError(details) => write!(f, "Bad Request: {details}"),
        }
    }
}

impl error::ResponseError for AppErrors {
    fn error_response(&self) -> HttpResponse {
        let error_template = ErrorTemplate {
            code: self.status_code(),
            text: &self.to_string(),
        }
        .render()
        .expect("Failed to render err template");

        HttpResponse::build(self.status_code())
            .content_type("text/html")
            .body(error_template)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppErrors::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppErrors::ClientError(_) => StatusCode::BAD_REQUEST,
        }
    }
}

pub fn establish_connection() -> Result<MysqlConnection, String> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").map_err(|err| {
        match err {
            env::VarError::NotPresent => "DATABASE_URL environment variable must be set",
            env::VarError::NotUnicode(_) => "Invalid value for DATABASE_URL environment variable",
        }
        .to_owned()
    })?;

    Ok(MysqlConnection::establish(&database_url)
        .map_err(|err| format!("Error establishing database connection: {err}"))?)
}

pub fn create_piece(
    connection: &mut MysqlConnection,
    title: &str,
    artist: &str,
    file_path: &str,
) -> Result<usize, String> {
    let desired_file = Path::new(file_path);

    if !desired_file.exists() {
        return Err("File does not exist".to_owned());
    }

    let file_size = desired_file
        .metadata()
        .map_err(|err| err.to_string())?
        .len() as u32;

    let new_piece = NewPiece {
        title,
        artist,
        file_path,
        file_size,
    };

    Ok(diesel::insert_into(music::table)
        .values(&new_piece)
        .execute(connection)
        .map_err(|err| err.to_string())?)
}
