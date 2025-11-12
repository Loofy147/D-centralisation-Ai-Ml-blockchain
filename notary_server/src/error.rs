use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::multipart::MultipartError;
use serde_json::json;

pub enum AppError {
    Sqlx(sqlx::Error),
    Axum(axum::Error),
    Multipart(MultipartError),
    SerdeJson(serde_json::Error),
    MissingField(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Sqlx(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Axum(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Multipart(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::SerdeJson(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::MissingField(field) => (StatusCode::BAD_REQUEST, format!("Missing field: {}", field)),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Sqlx(e)
    }
}

impl From<axum::Error> for AppError {
    fn from(e: axum::Error) -> Self {
        AppError::Axum(e)
    }
}

impl From<MultipartError> for AppError {
    fn from(e: MultipartError) -> Self {
        AppError::Multipart(e)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::SerdeJson(e)
    }
}
