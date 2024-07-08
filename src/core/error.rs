use std::io;

use rocket::{http::Status, serde::json::Json};
use serde::Serialize;

pub enum ErrorKind {
    Database(mongodb::error::Error),
    IO(io::Error),
    JWToken(jsonwebtoken::errors::Error),
}

#[derive(Debug, Serialize)]
pub struct Error {}

impl Error {
    pub fn custom() -> Self {
        Error {}
    }
}

impl ErrorKind {
    pub fn to_response(self) -> (Status, Json<Error>) {
        match self {
            Self::Database(_error) => (Status::InternalServerError, Json(Error {})),
            Self::IO(_error) => (Status::InternalServerError, Json(Error {})),
            Self::JWToken(_error) => (Status::InternalServerError, Json(Error {})),
        }
    }
}
