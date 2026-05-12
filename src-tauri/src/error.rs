use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error, Serialize)]
#[serde(tag = "kind", content = "message")]
pub enum AppError {
    #[error("database error: {0}")]
    Db(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("not found: {entity} id={id}")]
    NotFound { entity: String, id: String },

    #[error("invalid state transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Db(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
