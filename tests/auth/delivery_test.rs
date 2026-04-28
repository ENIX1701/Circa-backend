use crate::common::{setup_db, test_config};
use circa_backend::{
    auth::{
        delivery::{
            MagicLinkDelivery, MagicLinkDeliveryPayload, OutboxDelivery, SmtpDelivery,
            build_magic_link_delivery,
        },
        outbox_entity::Entity as OutboxEntity,
    },
    config::AuthDeliveryMode,
};
use sea_orm::EntityTrait;

fn payload() -> MagicLinkDeliveryPayload {
    MagicLinkDeliveryPayload {
        email: "alice@circa.local".to_string(),
        magic_token_id: "token-id".to_string(),
        magic_link: "http://localhost:5173/login?token=abc".to_string(),
        expires_at: "2099-01-01T00:00:00Z".to_string(),
    }
}

#[actix_web::test]
async fn outbox_delivery_writes_a_preview_row() {
    let db = setup_db().await;

    OutboxDelivery.deliver(&db, &payload()).await.unwrap();

    let rows = OutboxEntity::find().all(&db).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].email, "alice@circa.local");
    assert_eq!(rows[0].magic_token_id, "token-id");
    assert_eq!(rows[0].magic_link, "http://localhost:5173/login?token=abc");
    assert!(!rows[0].used);
}

#[actix_web::test]
async fn smtp_delivery_returns_internal_error_placeholder() {
    let db = setup_db().await;

    let err = SmtpDelivery.deliver(&db, &payload()).await.unwrap_err();

    assert_eq!(err.to_string(), "Internal server error");
}

#[test]
fn delivery_factory_accepts_outbox_and_smtp_modes() {
    let outbox = build_magic_link_delivery(&test_config(AuthDeliveryMode::Outbox));
    let smtp = build_magic_link_delivery(&test_config(AuthDeliveryMode::Smtp));

    assert_eq!(std::sync::Arc::strong_count(&outbox), 1);
    assert_eq!(std::sync::Arc::strong_count(&smtp), 1);
}
