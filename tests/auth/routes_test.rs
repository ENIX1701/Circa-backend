use actix_web::{App, http::StatusCode, test, web};
use circa_backend::auth;
use circa_backend::auth::service::generate_jwt;
use circa_backend::modules::auth::entity as magic_entity;
use circa_backend::modules::user::entity::{Model, Role, Status};
use circa_backend::user::repository::UserRepository;
use circa_backend::user::service::UserService;
use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};

const JWT_SECRET: &str = "test_secret";
const FRONTEND_URL: &str = "http://localhost:5137";

fn make_jwt_secret() -> web::Data<String> {
    web::Data::new(JWT_SECRET.to_string())
}

fn make_frontend_url() -> web::Data<String> {
    web::Data::new(FRONTEND_URL.to_string())
}

fn make_mock_db(db: DatabaseConnection) -> web::Data<DatabaseConnection> {
    web::Data::new(db)
}

fn stub_user_model(id: &str, email: &str, role: Role) -> Model {
    Model {
        id: id.to_string(),
        name: "John".to_string(),
        surname: "Doe".to_string(),
        email: email.to_string(),
        phone: "123".to_string(),
        role,
        status: Status::Active,
        availability_hours: "".to_string(),
    }
}

fn setup_user_service_with_user() -> web::Data<UserService> {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![stub_user_model(
            "user-1",
            "john@example.com",
            Role::Admin,
        )]])
        .into_connection();

    web::Data::new(UserService::new(UserRepository::new(db)))
}

fn setup_user_service_no_user() -> web::Data<UserService> {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();

    web::Data::new(UserService::new(UserRepository::new(db)))
}

// ── /auth/request-link ───────────────────────────────────────────────
//
// request_magic_link extracts web::Data<String> as frontend_url.
// The JWT bearer middleware is NOT applied to /auth routes, so
// the sole web::Data<String> we register here IS the frontend URL.
// We must NOT also register make_jwt_secret() because it would
// collide (same type).

#[actix_web::test]
async fn test_request_magic_link_success() {
    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![magic_entity::Model {
            id: "tok-1".to_string(),
            user_id: "user-1".to_string(),
            token: "generated-token".to_string(),
            expires_at: chrono::Utc::now().to_rfc3339(),
            used: false,
        }]])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let app = test::init_service(
        App::new()
            .app_data(setup_user_service_with_user())
            .app_data(make_frontend_url())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(serde_json::json!({ "email": "john@example.com" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_request_magic_link_response_contains_message() {
    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![magic_entity::Model {
            id: "tok-1".to_string(),
            user_id: "user-1".to_string(),
            token: "generated-token".to_string(),
            expires_at: chrono::Utc::now().to_rfc3339(),
            used: false,
        }]])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let app = test::init_service(
        App::new()
            .app_data(setup_user_service_with_user())
            .app_data(make_frontend_url())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(serde_json::json!({ "email": "john@example.com" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["message"]
            .as_str()
            .unwrap()
            .contains("Magic link sent")
    );
}

#[actix_web::test]
async fn test_request_magic_link_user_not_found() {
    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite).into_connection();

    let app = test::init_service(
        App::new()
            .app_data(setup_user_service_no_user())
            .app_data(make_frontend_url())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(serde_json::json!({ "email": "nobody@example.com" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn test_request_magic_link_db_insert_failure() {
    // User lookup succeeds but the magic token DB insert fails (no exec results)
    let user_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![stub_user_model(
            "user-1",
            "john@example.com",
            Role::Admin,
        )]])
        .into_connection();
    let user_service = web::Data::new(UserService::new(UserRepository::new(user_db)));

    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite).into_connection();

    let app = test::init_service(
        App::new()
            .app_data(user_service)
            .app_data(make_frontend_url())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(serde_json::json!({ "email": "john@example.com" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

// ── /auth/verify ─────────────────────────────────────────────────────
//
// The verify handler extracts web::Data<String> as jwt_secret.
// We must register the JWT secret as the sole web::Data<String>.
// Do NOT also register make_frontend_url() — same type, would collide.
//
// SeaORM's ActiveModel::update() returns a Model, so the mock DB
// needs TWO query results: one for the find, one for the update.

#[actix_web::test]
async fn test_verify_magic_link_success() {
    let future_time = (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();

    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            // find() for the magic token record
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-1".to_string(),
                token: "valid-token-abc".to_string(),
                expires_at: future_time.clone(),
                used: false,
            }],
            // update() returns the updated model
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-1".to_string(),
                token: "valid-token-abc".to_string(),
                expires_at: future_time,
                used: true,
            }],
        ])
        .into_connection();

    let user_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![stub_user_model(
            "user-1",
            "john@example.com",
            Role::Admin,
        )]])
        .into_connection();
    let user_service = web::Data::new(UserService::new(UserRepository::new(user_db)));

    let app = test::init_service(
        App::new()
            .app_data(user_service)
            .app_data(make_jwt_secret())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/verify?token=valid-token-abc")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_verify_magic_link_returns_jwt() {
    let future_time = (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();

    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-1".to_string(),
                token: "valid-token-abc".to_string(),
                expires_at: future_time.clone(),
                used: false,
            }],
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-1".to_string(),
                token: "valid-token-abc".to_string(),
                expires_at: future_time,
                used: true,
            }],
        ])
        .into_connection();

    let user_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![stub_user_model(
            "user-1",
            "john@example.com",
            Role::Admin,
        )]])
        .into_connection();
    let user_service = web::Data::new(UserService::new(UserRepository::new(user_db)));

    let app = test::init_service(
        App::new()
            .app_data(user_service)
            .app_data(make_jwt_secret())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/verify?token=valid-token-abc")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].as_str().is_some());
    assert!(!body["token"].as_str().unwrap().is_empty());
}

#[actix_web::test]
async fn test_verify_magic_link_invalid_token() {
    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<magic_entity::Model>::new()])
        .into_connection();

    let app = test::init_service(
        App::new()
            .app_data(setup_user_service_no_user())
            .app_data(make_jwt_secret())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/verify?token=bogus-token")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_verify_magic_link_expired_token() {
    let past_time = (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();

    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![magic_entity::Model {
            id: "tok-1".to_string(),
            user_id: "user-1".to_string(),
            token: "expired-token".to_string(),
            expires_at: past_time,
            used: false,
        }]])
        .into_connection();

    let app = test::init_service(
        App::new()
            .app_data(setup_user_service_with_user())
            .app_data(make_jwt_secret())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/verify?token=expired-token")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_verify_magic_link_user_not_found_after_verification() {
    let future_time = (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();

    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "ghost-user".to_string(),
                token: "orphan-token".to_string(),
                expires_at: future_time.clone(),
                used: false,
            }],
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "ghost-user".to_string(),
                token: "orphan-token".to_string(),
                expires_at: future_time,
                used: true,
            }],
        ])
        .into_connection();

    // User service returns nothing for the verified user_id
    let user_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();
    let user_service = web::Data::new(UserService::new(UserRepository::new(user_db)));

    let app = test::init_service(
        App::new()
            .app_data(user_service)
            .app_data(make_jwt_secret())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/verify?token=orphan-token")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_web::test]
async fn test_verify_magic_link_db_query_failure() {
    // Empty mock DB — the find query will fail
    let magic_db = MockDatabase::new(DatabaseBackend::Sqlite).into_connection();

    let app = test::init_service(
        App::new()
            .app_data(setup_user_service_with_user())
            .app_data(make_jwt_secret())
            .app_data(make_mock_db(magic_db))
            .configure(auth::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/verify?token=whatever")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── /api/me ──────────────────────────────────────────────────────────
//
// jwt_validator reads web::Data<String> as the JWT secret.
// We MUST NOT register make_frontend_url() here — it is the same
// type and would shadow the JWT secret, making every token fail.
// A bare DatabaseConnection is not extracted by get_current_user,
// so we can skip it entirely for these tests.

#[actix_web::test]
async fn test_get_current_user_with_valid_token() {
    let token_response = generate_jwt("user-1", "admin", JWT_SECRET).await.unwrap();

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
async fn test_get_current_user_response_body() {
    let token_response = generate_jwt("user-1", "admin", JWT_SECRET).await.unwrap();

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

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["sub"], "user-1");
    assert_eq!(body["role"], "admin");
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
    let token_response = generate_jwt("user-1", "admin", "wrong_secret")
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

#[actix_web::test]
async fn test_get_current_user_different_roles() {
    let app = test::init_service(
        App::new()
            .app_data(make_jwt_secret())
            .configure(auth::routes::config),
    )
    .await;

    for role in &["admin", "organizer", "staff", "volunteer"] {
        let token_response = generate_jwt("user-1", role, JWT_SECRET).await.unwrap();

        let req = test::TestRequest::get()
            .uri("/api/me")
            .insert_header(("Authorization", format!("Bearer {}", token_response.token)))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["sub"], "user-1");
        assert_eq!(body["role"], *role);
    }
}
