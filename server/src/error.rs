/*
error.rs
*/

use axum::{
    Json,
    http::StatusCode, 
    response::IntoResponse, 
};
use serde_json::{
    json,
};

#[derive(Debug)]
pub enum ApiError {
    StartupError(String),
    DatabaseError(String),
    AuthTokenError(String),
    InternalConversionError(String),
    ExternalAPIError(String),
    Unauthorized(String),
}

/*
Axum uses the IntoResponse trait to turn values into HTTP responses.
By implementing this trait for ApiError, we can now return these errors
from our handlers.
*/
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::StartupError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg
            ),
            ApiError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg
            ),
            ApiError::InternalConversionError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg
            ),
            ApiError::AuthTokenError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg
            ),
            ApiError::ExternalAPIError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg
            ),
            ApiError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                msg
            ),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
