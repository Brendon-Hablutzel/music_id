use actix_web::{error, http::StatusCode, HttpResponse};
use askama::Template;
use diesel::{mysql::MysqlConnection, prelude::*};
use dotenvy::dotenv;
use models::Piece;
use std::{env, fmt::Display};

pub mod models;
pub mod schema;

#[macro_export]
macro_rules! make_string_err {
    ($fallible:expr) => {
        $fallible.map_err(|err| err.to_string())
    };
}

#[macro_export]
macro_rules! to_client_err {
    ($text:literal) => {
        Err(AppErrors::ClientError($text.to_owned()).into())
    };
    ($fallible:expr) => {
        $fallible.map_err(|err| AppErrors::ClientError(err.to_string()))
    };
}

#[macro_export]
macro_rules! to_server_err {
    ($fallible:expr) => {
        $fallible.map_err(|err| {
            eprintln!("INTERNAL SERVER ERROR: {}", err.to_string());
            AppErrors::ServerError
        })
    };
}

#[macro_export]
macro_rules! query {
    ($($fname:ident $arg:expr),*) => {
        (|| {
            Ok::<Vec<Piece>, String>(music
                $(
                    .$fname($arg)
                )*
                .load::<Piece>(&mut establish_connection()?)
                .map_err(|err| err.to_string())?
            )
        })()
    }
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate<'a> {
    pub code: StatusCode,
    pub text: &'a str,
}

#[derive(Template)]
#[template(path = "quiz.html")]
pub struct QuizTemplate<'a> {
    pub pieces: &'a [Piece; 4],
    pub correct_piece: &'a Piece,
    pub correct_url: &'a str,
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
