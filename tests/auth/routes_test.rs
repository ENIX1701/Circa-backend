use actix_web::{App, http::StatusCode, test, web};
use circa_backend::auth;
use circa_backend::auth::service::generate_jwt;
use circa_backend::modules::user::entity::{Model, Role, Status};
use circa_backend::user::repository::UserRepository;
use circa_backend::user::service::UserService;
use sea_orm::{DatabaseBackend, MockDatabase};

const JWT_SECRET: &str = "test_secret";

fn make_jwt_secret() -> web::Data<String> {
    web::Data::new(JWT_SECRET.to_string())
}

fn setup_user_service_with_user() -> web::Data<UserService> {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![Model {
            id: "1".to_string(),
            name: "John".to_string(),
            surname: "Doe".to_string(),
            email: "john@example.com".to_string(),
            phone: "123".to_string(),
            role: Role::Admin,
            status: Status::Active,
        }]])
        .into_connection();

    web::Data::new(UserService::new(UserRepository::new(db)))
}

fn setup_user_service_no_user() -> web::Data<UserService> {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();

    web::Data::new(UserService::new(UserRepository::new(db)))
}

#[actix_web::test]
async fn test_login_success() {
    let app = test::init_service(
        App::new()
            .app_data(setup_user_service_with_user())
            .app_data(make_jwt_secret())
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(serde_json::json!({ "email": "john@example.com" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_login_user_not_found() {
    let app = test::init_service(
        App::new()
            .app_data(setup_user_service_no_user())
            .app_data(make_jwt_secret())
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/login")
        .set_json(serde_json::json!({ "email": "nobody@example.com" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_current_user_with_valid_token() {
    let token_response = generate_jwt("john@example.com", "admin", JWT_SECRET)
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(make_jwt_secret())
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/me")
        .insert_header(("Authorization", format!("Bearer {}", token_response.token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_get_current_user_without_token() {
    let app = test::init_service(
        App::new()
            .app_data(make_jwt_secret())
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/me").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_current_user_with_invalid_token() {
    let app = test::init_service(
        App::new()
            .app_data(make_jwt_secret())
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/me")
        .insert_header(("Authorization", "Bearer invalid.token.here"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_get_current_user_with_wrong_secret_token() {
    let token_response = generate_jwt("john@example.com", "admin", "wrong_secret")
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(make_jwt_secret())
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/me")
        .insert_header(("Authorization", format!("Bearer {}", token_response.token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
