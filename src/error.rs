use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use derive_more::Display;
use serde_json;

#[derive(Debug, Display)]
pub enum AppError {
    #[display("Internal server error")]
    InternalServerError,
    #[display("Bad request: {}", _0)]
    BadRequest(String),
    #[display("Not found: {}", _0)]
    NotFound(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(serde_json::json!({"error": self.to_string() }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
        }
    }
}
