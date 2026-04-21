use actix_web::{App, http::StatusCode, test, web};
use chrono::{Duration, Utc};
use circa_backend::{
    auth::{
        delivery::build_magic_link_delivery,
        entity::Entity as MagicTokenEntity,
        outbox_entity::{self, Entity as OutboxEntity},
        routes,
        service::GENERIC_MAGIC_LINK_MESSAGE,
    },
    config::{AppEnvironment, AuthDeliveryMode, Config},
    user::{repository::UserRepository, service::UserService},
};
use sea_orm::{
    ActiveValue::Set, ConnectOptions, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
};
use serde_json::{Value, json};

async fn setup_db() -> DatabaseConnection {
    let mut options = ConnectOptions::new("sqlite::memory:");
    options
        .max_connections(1)
        .min_connections(1)
        .sqlx_logging(false);

    let db = Database::connect(options).await.unwrap();

    db.execute_unprepared(
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            surname TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            phone TEXT NOT NULL,
            role TEXT NOT NULL,
            status TEXT NOT NULL,
            availability_hours TEXT NOT NULL
        );
        "#,
    )
    .await
    .unwrap();

    db.execute_unprepared(
        r#"
        CREATE TABLE magic_tokens (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            token TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            used BOOLEAN NOT NULL DEFAULT 0
        );
        "#,
    )
    .await
    .unwrap();

    db.execute_unprepared(
        r#"
        CREATE TABLE magic_link_outbox (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            magic_token_id TEXT NOT NULL,
            magic_link TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            used BOOLEAN NOT NULL DEFAULT 0
        );
        "#,
    )
    .await
    .unwrap();

    db
}

fn make_config(auth_delivery_mode: AuthDeliveryMode) -> Config {
    Config {
        database_url: "sqlite::memory:".to_string(),
        jwt_secret: "test-secret".to_string(),
        frontend_url: "http://localhost:5173".to_string(),
        app_env: AppEnvironment::Development,
        auth_delivery_mode,
    }
}

async fn seed_user(db: &DatabaseConnection, id: &str, email: &str, role: &str) {
    let sql = format!(
        "INSERT INTO users (id, name, surname, email, phone, role, status, availability_hours) \
         VALUES ('{}', 'Alice', 'Tester', '{}', '+48123456789', '{}', 'active', '[]');",
        id, email, role
    );

    db.execute_unprepared(&sql).await.unwrap();
}

async fn insert_outbox_row(
    db: &DatabaseConnection,
    id: &str,
    email: &str,
    magic_token_id: &str,
    magic_link: &str,
    created_at: String,
    expires_at: String,
    used: bool,
) {
    OutboxEntity::insert(outbox_entity::ActiveModel {
        id: Set(id.to_string()),
        email: Set(email.to_string()),
        magic_token_id: Set(magic_token_id.to_string()),
        magic_link: Set(magic_link.to_string()),
        created_at: Set(created_at),
        expires_at: Set(expires_at),
        used: Set(used),
    })
    .exec(db)
    .await
    .unwrap();
}

#[actix_web::test]
async fn request_link_returns_generic_success_for_missing_email_without_writing_outbox() {
    let db = setup_db().await;
    let config = make_config(AuthDeliveryMode::Outbox);
    let delivery = build_magic_link_delivery(&config);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(UserService::new(UserRepository::new(
                db.clone(),
            ))))
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(delivery))
            .app_data(web::Data::new(db.clone()))
            .configure(routes::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(json!({ "email": "missing@circa.local" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(
        body.get("message").and_then(Value::as_str),
        Some(GENERIC_MAGIC_LINK_MESSAGE)
    );

    let outbox_rows = OutboxEntity::find().all(&db).await.unwrap();
    assert_eq!(outbox_rows.len(), 0);
}

#[actix_web::test]
async fn request_link_returns_generic_success_for_known_email_and_writes_outbox() {
    let db = setup_db().await;
    seed_user(&db, "user-123", "alice@circa.local", "admin").await;

    let config = make_config(AuthDeliveryMode::Outbox);
    let delivery = build_magic_link_delivery(&config);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(UserService::new(UserRepository::new(
                db.clone(),
            ))))
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(delivery))
            .app_data(web::Data::new(db.clone()))
            .configure(routes::config),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(json!({ "email": "alice@circa.local" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(
        body.get("message").and_then(Value::as_str),
        Some(GENERIC_MAGIC_LINK_MESSAGE)
    );

    let outbox_rows = OutboxEntity::find().all(&db).await.unwrap();
    assert_eq!(outbox_rows.len(), 1);
    assert_eq!(outbox_rows[0].email, "alice@circa.local");
}

#[actix_web::test]
async fn test_inbox_latest_returns_the_newest_valid_link() {
    let db = setup_db().await;
    let now = Utc::now();

    insert_outbox_row(
        &db,
        "expired-row",
        "alice@circa.local",
        "token-expired",
        "http://localhost:5173/login?token=expired",
        (now - Duration::minutes(20)).to_rfc3339(),
        (now - Duration::minutes(5)).to_rfc3339(),
        false,
    )
    .await;

    insert_outbox_row(
        &db,
        "latest-valid-row",
        "alice@circa.local",
        "token-latest",
        "http://localhost:5173/login?token=latest",
        now.to_rfc3339(),
        (now + Duration::minutes(10)).to_rfc3339(),
        false,
    )
    .await;

    let config = make_config(AuthDeliveryMode::Outbox);
    let delivery = build_magic_link_delivery(&config);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(UserService::new(UserRepository::new(
                db.clone(),
            ))))
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(delivery))
            .app_data(web::Data::new(db.clone()))
            .configure(routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/test-inbox/latest?email=alice@circa.local")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(
        body.get("email").and_then(Value::as_str),
        Some("alice@circa.local")
    );
    assert_eq!(
        body.get("magic_link").and_then(Value::as_str),
        Some("http://localhost:5173/login?token=latest")
    );
}

#[actix_web::test]
async fn test_inbox_latest_returns_404_when_disabled() {
    let db = setup_db().await;
    let config = make_config(AuthDeliveryMode::Smtp);
    let delivery = build_magic_link_delivery(&config);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(UserService::new(UserRepository::new(
                db.clone(),
            ))))
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(delivery))
            .app_data(web::Data::new(db))
            .configure(routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/auth/test-inbox/latest?email=alice@circa.local")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn verify_marks_token_and_outbox_row_used() {
    let db = setup_db().await;
    seed_user(&db, "user-123", "alice@circa.local", "admin").await;

    let config = make_config(AuthDeliveryMode::Outbox);
    let delivery = build_magic_link_delivery(&config);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(UserService::new(UserRepository::new(
                db.clone(),
            ))))
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(delivery))
            .app_data(web::Data::new(db.clone()))
            .configure(routes::config),
    )
    .await;

    let request_link_req = test::TestRequest::post()
        .uri("/auth/request-link")
        .set_json(json!({ "email": "alice@circa.local" }))
        .to_request();

    let request_link_resp = test::call_service(&app, request_link_req).await;
    assert_eq!(request_link_resp.status(), StatusCode::OK);

    let outbox_row = OutboxEntity::find().one(&db).await.unwrap().unwrap();
    let token = outbox_row
        .magic_link
        .split("token=")
        .nth(1)
        .expect("magic link should contain token")
        .to_string();

    let verify_req = test::TestRequest::get()
        .uri(&format!("/auth/verify?token={}", token))
        .to_request();

    let verify_resp = test::call_service(&app, verify_req).await;
    assert_eq!(verify_resp.status(), StatusCode::OK);

    let verify_body: Value = test::read_body_json(verify_resp).await;
    assert!(verify_body.get("token").and_then(Value::as_str).is_some());

    let outbox_after = OutboxEntity::find_by_id(outbox_row.id.clone())
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert!(outbox_after.used);

    let magic_token_after = MagicTokenEntity::find_by_id(outbox_row.magic_token_id.clone())
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert!(magic_token_after.used);
}
