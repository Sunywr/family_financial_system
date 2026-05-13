use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("internal server error")]
    Internal,
    #[error("database error: {0}")]
    Database(String),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: i32,
    message: String,
    data: Option<()>,
}

impl AppError {
    pub fn internal_with_log<E>(error: E) -> Self
    where
        E: std::fmt::Display,
    {
        error!(error = %error, "internal error");
        Self::Internal
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Internal | AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ErrorBody {
            code: status.as_u16() as i32,
            message: self.to_string(),
            data: None,
        };

        (status, Json(body)).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        error!(error = %error, "database error");
        Self::Database(error.to_string())
    }
}
