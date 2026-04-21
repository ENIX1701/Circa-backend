use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait};
use std::sync::Arc;

use crate::{
    config::{AuthDeliveryMode, Config},
    error::AppError,
};

use super::outbox_entity::{self, Entity as OutboxEntity};

#[derive(Debug, Clone)]
pub struct MagicLinkDeliveryPayload {
    pub email: String,
    pub magic_token_id: String,
    pub magic_link: String,
    pub expires_at: String,
}

#[async_trait]
pub trait MagicLinkDelivery: Send + Sync {
    async fn deliver(
        &self,
        db: &DatabaseConnection,
        payload: &MagicLinkDeliveryPayload,
    ) -> Result<(), AppError>;
}

pub struct OutboxDelivery;
pub struct SmtpDelivery;

#[async_trait]
impl MagicLinkDelivery for OutboxDelivery {
    async fn deliver(
        &self,
        db: &DatabaseConnection,
        payload: &MagicLinkDeliveryPayload,
    ) -> Result<(), AppError> {
        let record = outbox_entity::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            email: Set(payload.email.clone()),
            magic_token_id: Set(payload.magic_token_id.clone()),
            magic_link: Set(payload.magic_link.clone()),
            created_at: Set(Utc::now().to_rfc3339()),
            expires_at: Set(payload.expires_at.clone()),
            used: Set(false),
        };

        OutboxEntity::insert(record)
            .exec(db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }
}

#[async_trait]
impl MagicLinkDelivery for SmtpDelivery {
    async fn deliver(
        &self,
        db: &DatabaseConnection,
        payload: &MagicLinkDeliveryPayload,
    ) -> Result<(), AppError> {
        Err(AppError::InternalServerError)
    }
}

pub fn build_magic_link_delivery(config: &Config) -> Arc<dyn MagicLinkDelivery> {
    match config.auth_delivery_mode {
        AuthDeliveryMode::Outbox => Arc::new(OutboxDelivery),
        AuthDeliveryMode::Smtp => Arc::new(SmtpDelivery),
    }
}
