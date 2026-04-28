use circa_backend::{
    auth::{entity::Entity as MagicTokenEntity, outbox_entity::Entity as OutboxEntity},
    config::{AppEnvironment, AuthDeliveryMode, Config},
    db,
    event::{
        entity::Entity as EventEntity, event_branding_entity::Entity as EventBrandingEntity,
        membership_entity::Entity as EventMembershipEntity,
        planner_item_entity::Entity as PlannerItemEntity,
        planner_timeline_item_entity::Entity as PlannerTimelineItemEntity,
        social_post_entity::Entity as SocialPostEntity,
    },
    user::entity::Entity as UserEntity,
};
use sea_orm::EntityTrait;

#[test]
fn config_struct_reports_test_inbox_availability_by_delivery_mode() {
    let outbox = Config {
        database_url: "sqlite::memory:".to_string(),
        jwt_secret: "secret".to_string(),
        frontend_url: "http://localhost:5173".to_string(),
        app_env: AppEnvironment::Development,
        auth_delivery_mode: AuthDeliveryMode::Outbox,
    };

    let smtp = Config {
        auth_delivery_mode: AuthDeliveryMode::Smtp,
        ..outbox.clone()
    };

    assert!(outbox.test_inbox_enabled());
    assert!(!smtp.test_inbox_enabled());
}

#[actix_web::test]
async fn establish_connection_initializes_runtime_sqlite_schema() {
    let db = db::establish_connection("sqlite::memory:").await.unwrap();

    assert!(UserEntity::find().all(&db).await.is_ok());
    assert!(MagicTokenEntity::find().all(&db).await.is_ok());
    assert!(OutboxEntity::find().all(&db).await.is_ok());
    assert!(EventEntity::find().all(&db).await.is_ok());
    assert!(EventMembershipEntity::find().all(&db).await.is_ok());
    assert!(EventBrandingEntity::find().all(&db).await.is_ok());
    assert!(PlannerItemEntity::find().all(&db).await.is_ok());
    assert!(PlannerTimelineItemEntity::find().all(&db).await.is_ok());
    assert!(SocialPostEntity::find().all(&db).await.is_ok());
}
