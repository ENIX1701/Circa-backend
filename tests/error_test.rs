use actix_web::ResponseError;
use actix_web::http::StatusCode;
use circa_backend::error::AppError;

#[test]
fn test_internal_server_error_status() {
    let err = AppError::InternalServerError;
    assert_eq!(err.status_code(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_bad_request_status() {
    let err = AppError::BadRequest("invalid input".to_string());
    assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_not_found_status() {
    let err = AppError::NotFound("resource missing".to_string());
    assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
}

#[test]
fn test_unauthorized_status() {
    let err = AppError::Unauthorized;
    assert_eq!(err.status_code(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_forbidden_status() {
    let err = AppError::Forbidden;
    assert_eq!(err.status_code(), StatusCode::FORBIDDEN);
}

#[test]
fn test_internal_server_error_display() {
    let err = AppError::InternalServerError;
    assert_eq!(err.to_string(), "Internal server error");
}

#[test]
fn test_bad_request_display() {
    let err = AppError::BadRequest("missing field".to_string());
    assert_eq!(err.to_string(), "Bad request: missing field");
}

#[test]
fn test_not_found_display() {
    let err = AppError::NotFound("User not found".to_string());
    assert_eq!(err.to_string(), "Not found: User not found");
}

#[test]
fn test_error_response_body() {
    let err = AppError::BadRequest("test error".to_string());
    let response = err.error_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_unauthorized_error_response() {
    let err = AppError::Unauthorized;
    let response = err.error_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_forbidden_error_response() {
    let err = AppError::Forbidden;
    let response = err.error_response();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn test_not_found_error_response() {
    let err = AppError::NotFound("item".to_string());
    let response = err.error_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_internal_server_error_response() {
    let err = AppError::InternalServerError;
    let response = err.error_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
