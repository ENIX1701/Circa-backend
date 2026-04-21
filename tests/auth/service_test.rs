use chrono::{Duration, Utc};
use circa_backend::auth::{
    delivery::OutboxDelivery,
    entity::{self as magic_entity, Entity as MagicTokenEntity},
    outbox_entity::{self, Entity as OutboxEntity},
    service::{
        GENERIC_MAGIC_LINK_MESSAGE, create_magic_token, get_latest_test_inbox_link,
        verify_magic_token,
    },
};
use sea_orm::{
    ActiveValue::Set, ConnectOptions, ConnectionTrait, Database, DatabaseConnection, EntityTrait,
};

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
            first_name TEXT NOT NULL,
            last_name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            role TEXT NOT NULL,
            status TEXT NOT NULL
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
async fn create_magic_token_returns_generic_message_and_writes_outbox() {
    let db = setup_db().await;

    let response = create_magic_token(
        &db,
        &OutboxDelivery,
        "user-123",
        "alice@circa.local",
        "http://localhost:5173",
    )
    .await
    .unwrap();

    assert_eq!(response.message, GENERIC_MAGIC_LINK_MESSAGE);

    let tokens = MagicTokenEntity::find().all(&db).await.unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].user_id, "user-123");
    assert!(!tokens[0].used);

    let outbox_rows = OutboxEntity::find().all(&db).await.unwrap();
    assert_eq!(outbox_rows.len(), 1);
    assert_eq!(outbox_rows[0].email, "alice@circa.local");
    assert_eq!(outbox_rows[0].magic_token_id, tokens[0].id);
    assert!(
        outbox_rows[0]
            .magic_link
            .starts_with("http://localhost:5173/login?token=")
    );
    assert!(!outbox_rows[0].used);
}

#[actix_web::test]
async fn get_latest_test_inbox_link_returns_newest_valid_row_only() {
    let db = setup_db().await;
    let now = Utc::now();

    insert_outbox_row(
        &db,
        "expired-row",
        "alice@circa.local",
        "token-expired",
        "http://localhost:5173/login?token=expired",
        (now - Duration::minutes(30)).to_rfc3339(),
        (now - Duration::minutes(5)).to_rfc3339(),
        false,
    )
    .await;

    insert_outbox_row(
        &db,
        "used-row",
        "alice@circa.local",
        "token-used",
        "http://localhost:5173/login?token=used",
        (now - Duration::minutes(20)).to_rfc3339(),
        (now + Duration::minutes(10)).to_rfc3339(),
        true,
    )
    .await;

    insert_outbox_row(
        &db,
        "older-valid-row",
        "alice@circa.local",
        "token-old",
        "http://localhost:5173/login?token=old",
        (now - Duration::minutes(10)).to_rfc3339(),
        (now + Duration::minutes(10)).to_rfc3339(),
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

    let preview = get_latest_test_inbox_link(&db, "alice@circa.local")
        .await
        .unwrap();

    assert_eq!(preview.email, "alice@circa.local");
    assert_eq!(
        preview.magic_link,
        "http://localhost:5173/login?token=latest"
    );
}

#[actix_web::test]
async fn verify_magic_token_marks_token_and_outbox_row_used() {
    let db = setup_db().await;

    MagicTokenEntity::insert(magic_entity::ActiveModel {
        id: Set("magic-token-id".to_string()),
        user_id: Set("user-123".to_string()),
        token: Set("valid-token".to_string()),
        expires_at: Set((Utc::now() + Duration::minutes(15)).to_rfc3339()),
        used: Set(false),
    })
    .exec(&db)
    .await
    .unwrap();

    insert_outbox_row(
        &db,
        "outbox-row",
        "alice@circa.local",
        "magic-token-id",
        "http://localhost:5173/login?token=valid-token",
        Utc::now().to_rfc3339(),
        (Utc::now() + Duration::minutes(15)).to_rfc3339(),
        false,
    )
    .await;

    let user_id = verify_magic_token(&db, "valid-token").await.unwrap();
    assert_eq!(user_id, "user-123");

    let token = MagicTokenEntity::find_by_id("magic-token-id")
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert!(token.used);

    let outbox_row = OutboxEntity::find_by_id("outbox-row")
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert!(outbox_row.used);
}
