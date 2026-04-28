use crate::common::{
    OutboxSeed, insert_magic_token, insert_outbox, jwt, seed_user, setup_db, test_config,
    user_service,
};
use actix_web::{App, http::StatusCode, test, web};
use chrono::{Duration, Utc};
use circa_backend::{
    auth::{
        delivery::build_magic_link_delivery, outbox_entity::Entity as OutboxEntity, routes,
        service::GENERIC_MAGIC_LINK_MESSAGE,
    },
    config::AuthDeliveryMode,
};
use sea_orm::EntityTrait;
use serde_json::{Value, json};

async fn app(
    db: &sea_orm::DatabaseConnection,
    mode: AuthDeliveryMode,
) -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let config = test_config(mode);
    let delivery = build_magic_link_delivery(&config);

    test::init_service(
        App::new()
            .app_data(user_service(db))
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(delivery))
            .configure(routes::config),
    )
    .await
}

#[actix_web::test]
async fn request_link_rejects_empty_email() {
    let db = setup_db().await;
    let app = app(&db, AuthDeliveryMode::Outbox).await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(json!({ "email": "   " }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn request_link_returns_generic_response_for_missing_email_without_outbox_write() {
    let db = setup_db().await;
    let app = app(&db, AuthDeliveryMode::Outbox).await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(json!({ "email": "missing@circa.local" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;

    assert_eq!(body["message"], GENERIC_MAGIC_LINK_MESSAGE);
    assert_eq!(OutboxEntity::find().all(&db).await.unwrap().len(), 0);
}

#[actix_web::test]
async fn request_link_returns_generic_response_for_known_email_and_writes_outbox() {
    let db = setup_db().await;
    seed_user(&db, "user-1", "alice@circa.local", "admin", "active").await;
    let app = app(&db, AuthDeliveryMode::Outbox).await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(json!({ "email": "alice@circa.local" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;

    assert_eq!(body["message"], GENERIC_MAGIC_LINK_MESSAGE);
    assert_eq!(OutboxEntity::find().all(&db).await.unwrap().len(), 1);
}

#[actix_web::test]
async fn verify_returns_token_for_valid_magic_link() {
    let db = setup_db().await;
    seed_user(&db, "user-1", "alice@circa.local", "admin", "active").await;
    let app = app(&db, AuthDeliveryMode::Outbox).await;

    let request_link = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(json!({ "email": "alice@circa.local" }))
        .to_request();
    assert_eq!(
        test::call_service(&app, request_link).await.status(),
        StatusCode::OK
    );

    let outbox = OutboxEntity::find().one(&db).await.unwrap().unwrap();
    let token = outbox.magic_link.split("token=").nth(1).unwrap();

    let verify = test::TestRequest::get()
        .uri(&format!("/auth/verify?token={token}"))
        .to_request();
    let resp = test::call_service(&app, verify).await;
    let body: Value = test::read_body_json(resp).await;

    assert!(body["token"].as_str().is_some());
}

#[actix_web::test]
async fn verify_returns_bad_request_for_invalid_magic_link() {
    let db = setup_db().await;
    let app = app(&db, AuthDeliveryMode::Outbox).await;

    let req = test::TestRequest::get()
        .uri("/auth/verify?token=missing")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn verify_returns_internal_error_when_verified_user_disappeared() {
    let db = setup_db().await;
    insert_magic_token(
        &db,
        "token-id",
        "missing-user",
        "valid-token",
        &(Utc::now() + Duration::minutes(15)).to_rfc3339(),
        false,
    )
    .await;

    let app = app(&db, AuthDeliveryMode::Outbox).await;

    let req = test::TestRequest::get()
        .uri("/auth/verify?token=valid-token")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_web::test]
async fn test_inbox_latest_handles_empty_disabled_missing_and_success_states() {
    let db = setup_db().await;
    let future = (Utc::now() + Duration::minutes(15)).to_rfc3339();

    insert_outbox(
        &db,
        OutboxSeed {
            id: "outbox-1",
            email: "alice@circa.local",
            magic_token_id: "token-1",
            link: "http://localhost/latest",
            created_at: "2026-04-20T10:00:00Z",
            expires_at: &future,
            used: false,
        },
    )
    .await;

    let outbox_app = app(&db, AuthDeliveryMode::Outbox).await;

    let empty = test::TestRequest::get()
        .uri("/auth/test-inbox/latest?email=%20%20%20")
        .to_request();
    assert_eq!(
        test::call_service(&outbox_app, empty).await.status(),
        StatusCode::BAD_REQUEST
    );

    let missing = test::TestRequest::get()
        .uri("/auth/test-inbox/latest?email=missing@circa.local")
        .to_request();
    assert_eq!(
        test::call_service(&outbox_app, missing).await.status(),
        StatusCode::NOT_FOUND
    );

    let success = test::TestRequest::get()
        .uri("/auth/test-inbox/latest?email=alice@circa.local")
        .to_request();
    let resp = test::call_service(&outbox_app, success).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["magic_link"], "http://localhost/latest");

    let smtp_app = app(&db, AuthDeliveryMode::Smtp).await;
    let disabled = test::TestRequest::get()
        .uri("/auth/test-inbox/latest?email=alice@circa.local")
        .to_request();
    assert_eq!(
        test::call_service(&smtp_app, disabled).await.status(),
        StatusCode::NOT_FOUND
    );
}

#[actix_web::test]
async fn me_endpoint_requires_valid_bearer_token_and_returns_claims() {
    let db = setup_db().await;
    let app = app(&db, AuthDeliveryMode::Outbox).await;

    let unauthorized = test::TestRequest::get().uri("/api/me").to_request();
    assert_eq!(
        test::call_service(&app, unauthorized).await.status(),
        StatusCode::UNAUTHORIZED
    );

    let invalid = test::TestRequest::get()
        .uri("/api/me")
        .insert_header(("Authorization", "Bearer not-a-jwt"))
        .to_request();
    assert_eq!(
        test::call_service(&app, invalid).await.status(),
        StatusCode::UNAUTHORIZED
    );

    let token = jwt("user-1", "admin").await;
    let valid = test::TestRequest::get()
        .uri("/api/me")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();

    let resp = test::call_service(&app, valid).await;
    let body: Value = test::read_body_json(resp).await;

    assert_eq!(body["sub"], "user-1");
    assert_eq!(body["role"], "admin");
}
