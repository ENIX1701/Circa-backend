use crate::common::{
    JWT_SECRET, OutboxSeed, insert_magic_token, insert_outbox, setup_db, test_config,
};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use circa_backend::{
    auth::{
        delivery::{MagicLinkDelivery, MagicLinkDeliveryPayload, OutboxDelivery},
        entity::Entity as MagicTokenEntity,
        models::Claims,
        outbox_entity::Entity as OutboxEntity,
        service::{
            GENERIC_MAGIC_LINK_MESSAGE, create_magic_token, generate_jwt,
            generic_magic_link_response, get_latest_test_inbox_link, verify_magic_token,
        },
    },
    config::AuthDeliveryMode,
    error::AppError,
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::EntityTrait;

struct FailingDelivery;

#[async_trait]
impl MagicLinkDelivery for FailingDelivery {
    async fn deliver(
        &self,
        _db: &sea_orm::DatabaseConnection,
        _payload: &MagicLinkDeliveryPayload,
    ) -> Result<(), AppError> {
        Err(AppError::InternalServerError)
    }
}

#[test]
fn generic_magic_link_response_uses_security_preserving_message() {
    let response = generic_magic_link_response();

    assert_eq!(response.message, GENERIC_MAGIC_LINK_MESSAGE);
}

#[actix_web::test]
async fn generate_jwt_encodes_subject_role_and_future_expiration() {
    let token = generate_jwt("user-1", "admin", JWT_SECRET)
        .await
        .unwrap()
        .token;

    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET.as_bytes()),
        &Validation::default(),
    )
    .unwrap();

    assert_eq!(decoded.claims.sub, "user-1");
    assert_eq!(decoded.claims.role, "admin");
    assert!(decoded.claims.exp > Utc::now().timestamp() as usize);
}

#[actix_web::test]
async fn create_magic_token_returns_generic_message_and_writes_outbox() {
    let db = setup_db().await;

    let response = create_magic_token(
        &db,
        &OutboxDelivery,
        "user-1",
        "alice@circa.local",
        "http://localhost:5173",
    )
    .await
    .unwrap();

    assert_eq!(response.message, GENERIC_MAGIC_LINK_MESSAGE);

    let tokens = MagicTokenEntity::find().all(&db).await.unwrap();
    let outbox = OutboxEntity::find().all(&db).await.unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].user_id, "user-1");
    assert!(!tokens[0].used);

    assert_eq!(outbox.len(), 1);
    assert_eq!(outbox[0].email, "alice@circa.local");
    assert_eq!(outbox[0].magic_token_id, tokens[0].id);
    assert!(
        outbox[0]
            .magic_link
            .starts_with("http://localhost:5173/login?token=")
    );
    assert!(!outbox[0].used);
}

#[actix_web::test]
async fn create_magic_token_removes_token_when_delivery_fails() {
    let db = setup_db().await;

    let err = create_magic_token(
        &db,
        &FailingDelivery,
        "user-1",
        "alice@circa.local",
        "http://localhost:5173",
    )
    .await
    .unwrap_err();

    assert_eq!(err.to_string(), "Internal server error");
    assert_eq!(MagicTokenEntity::find().all(&db).await.unwrap().len(), 0);
}

#[actix_web::test]
async fn verify_magic_token_marks_token_and_outbox_row_used() {
    let db = setup_db().await;
    let token_expires_at = (Utc::now() + Duration::minutes(15)).to_rfc3339();
    let outbox_expires_at = (Utc::now() + Duration::minutes(15)).to_rfc3339();

    insert_magic_token(
        &db,
        "magic-token-id",
        "user-1",
        "valid-token",
        &token_expires_at,
        false,
    )
    .await;

    insert_outbox(
        &db,
        OutboxSeed {
            id: "outbox-row",
            email: "alice@circa.local",
            magic_token_id: "magic-token-id",
            link: "http://localhost:5173/login?token=valid-token",
            created_at: "2026-04-20T10:00:00Z",
            expires_at: &outbox_expires_at,
            used: false,
        },
    )
    .await;

    let user_id = verify_magic_token(&db, "valid-token").await.unwrap();

    assert_eq!(user_id, "user-1");
    assert!(
        MagicTokenEntity::find_by_id("magic-token-id")
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .used
    );
    assert!(
        OutboxEntity::find_by_id("outbox-row")
            .one(&db)
            .await
            .unwrap()
            .unwrap()
            .used
    );
}

#[actix_web::test]
async fn verify_magic_token_rejects_missing_used_expired_and_malformed_tokens() {
    let db = setup_db().await;

    insert_magic_token(
        &db,
        "used-id",
        "user-1",
        "used-token",
        &(Utc::now() + Duration::minutes(15)).to_rfc3339(),
        true,
    )
    .await;
    insert_magic_token(
        &db,
        "expired-id",
        "user-1",
        "expired-token",
        &(Utc::now() - Duration::minutes(1)).to_rfc3339(),
        false,
    )
    .await;
    insert_magic_token(
        &db,
        "bad-date-id",
        "user-1",
        "bad-date-token",
        "not-a-date",
        false,
    )
    .await;

    assert_eq!(
        verify_magic_token(&db, "missing")
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: Invalid or expired magic link"
    );
    assert_eq!(
        verify_magic_token(&db, "used-token")
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: Invalid or expired magic link"
    );
    assert_eq!(
        verify_magic_token(&db, "expired-token")
            .await
            .unwrap_err()
            .to_string(),
        "Bad request: Magic link has expired"
    );
    assert_eq!(
        verify_magic_token(&db, "bad-date-token")
            .await
            .unwrap_err()
            .to_string(),
        "Internal server error"
    );
}

#[actix_web::test]
async fn get_latest_test_inbox_link_returns_newest_unused_unexpired_row() {
    let db = setup_db().await;
    let future = (Utc::now() + Duration::minutes(15)).to_rfc3339();
    let past = (Utc::now() - Duration::minutes(1)).to_rfc3339();

    insert_outbox(
        &db,
        OutboxSeed {
            id: "expired",
            email: "alice@circa.local",
            magic_token_id: "expired-token",
            link: "http://localhost/expired",
            created_at: "2026-04-20T10:00:00Z",
            expires_at: &past,
            used: false,
        },
    )
    .await;

    insert_outbox(
        &db,
        OutboxSeed {
            id: "used",
            email: "alice@circa.local",
            magic_token_id: "used-token",
            link: "http://localhost/used",
            created_at: "2026-04-20T10:01:00Z",
            expires_at: &future,
            used: true,
        },
    )
    .await;

    insert_outbox(
        &db,
        OutboxSeed {
            id: "older",
            email: "alice@circa.local",
            magic_token_id: "older-token",
            link: "http://localhost/older",
            created_at: "2026-04-20T10:02:00Z",
            expires_at: &future,
            used: false,
        },
    )
    .await;

    insert_outbox(
        &db,
        OutboxSeed {
            id: "latest",
            email: "alice@circa.local",
            magic_token_id: "latest-token",
            link: "http://localhost/latest",
            created_at: "2026-04-20T10:03:00Z",
            expires_at: &future,
            used: false,
        },
    )
    .await;

    let preview = get_latest_test_inbox_link(&db, "alice@circa.local")
        .await
        .unwrap();

    assert_eq!(preview.email, "alice@circa.local");
    assert_eq!(preview.magic_link, "http://localhost/latest");
}

#[actix_web::test]
async fn get_latest_test_inbox_link_returns_not_found_when_no_valid_row_exists() {
    let db = setup_db().await;

    let err = get_latest_test_inbox_link(&db, "missing@circa.local")
        .await
        .unwrap_err();

    assert_eq!(err.to_string(), "Not found: No valid magic link found");
}

#[actix_web::test]
async fn auth_config_helper_keeps_test_secret_consistent() {
    let config = test_config(AuthDeliveryMode::Outbox);

    assert_eq!(config.jwt_secret, JWT_SECRET);
    assert!(config.test_inbox_enabled());
}
