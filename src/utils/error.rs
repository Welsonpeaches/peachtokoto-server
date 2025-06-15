use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Internal server error: {0}")]
    Internal(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("File system error: {0}")]
    FileSystem(#[from] notify::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Io(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO Error: {}", err),
            ),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::FileSystem(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("File System Error: {}", err),
            ),
        };

        (status, message).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;