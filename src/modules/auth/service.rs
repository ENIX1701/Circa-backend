use crate::{
    auth::{
        delivery::{MagicLinkDelivery, MagicLinkDeliveryPayload},
        models::{Claims, MagicLinkResponse, TestInboxLinkPreview, TokenResponse},
    },
    error::AppError,
};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::RngExt;
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    entity::{self as magic_entity, Entity as MagicTokenEntity},
    outbox_entity::{self, Entity as OutboxEntity},
};

pub const GENERIC_MAGIC_LINK_MESSAGE: &str =
    "If that email is registered, a magic link has been sent";

// for security we'll just return a generic message
// this isn't *required* now, but it'll be handy in the future
pub fn generic_magic_link_response() -> MagicLinkResponse {
    MagicLinkResponse {
        message: GENERIC_MAGIC_LINK_MESSAGE.to_string(),
    }
}

pub async fn generate_jwt(
    user_id: &str,
    role: &str,
    secret: &str,
) -> Result<TokenResponse, jsonwebtoken::errors::Error> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time bent backwards @-@")
        .as_secs() as usize
        + 60 * 60 * 24;

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;

    Ok(TokenResponse { token })
}

pub async fn create_magic_token(
    db: &DatabaseConnection,
    delivery: &dyn MagicLinkDelivery,
    user_id: &str,
    email: &str,
    frontend_url: &str,
) -> Result<MagicLinkResponse, AppError> {
    let token: String = rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect();

    let id = uuid::Uuid::now_v7().to_string();
    let expires_at = (Utc::now() + chrono::Duration::minutes(15)).to_rfc3339();

    let record = magic_entity::ActiveModel {
        id: Set(id.clone()),
        user_id: Set(user_id.to_string()),
        token: Set(token.clone()),
        expires_at: Set(expires_at.clone()),
        used: Set(false),
    };

    MagicTokenEntity::insert(record)
        .exec(db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let payload = MagicLinkDeliveryPayload {
        email: email.to_string(),
        magic_token_id: id.clone(),
        magic_link: format!("{}/login?token={}", frontend_url, token),
        expires_at,
    };

    if let Err(err) = delivery.deliver(db, &payload).await {
        let _ = MagicTokenEntity::delete_by_id(id).exec(db).await;
        return Err(err);
    }

    Ok(generic_magic_link_response())
}

pub async fn verify_magic_token(db: &DatabaseConnection, token: &str) -> Result<String, AppError> {
    let record = MagicTokenEntity::find()
        .filter(magic_entity::Column::Token.eq(token))
        .filter(magic_entity::Column::Used.eq(false))
        .one(db)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or_else(|| AppError::BadRequest("Invalid or expired magic link".to_string()))?;

    let expires_at = chrono::DateTime::parse_from_rfc3339(&record.expires_at)
        .map_err(|_| AppError::InternalServerError)?;

    if Utc::now() > expires_at.with_timezone(&Utc) {
        return Err(AppError::BadRequest("Magic link has expired".to_string()));
    }

    MagicTokenEntity::update_many()
        .col_expr(magic_entity::Column::Used, Expr::value(true))
        .filter(magic_entity::Column::Id.eq(record.id.clone()))
        .exec(db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    OutboxEntity::update_many()
        .col_expr(outbox_entity::Column::Used, Expr::value(true))
        .filter(outbox_entity::Column::MagicTokenId.eq(record.id.clone()))
        .exec(db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    Ok(record.user_id)
}

pub async fn get_latest_test_inbox_link(
    db: &DatabaseConnection,
    email: &str,
) -> Result<TestInboxLinkPreview, AppError> {
    let record = OutboxEntity::find()
        .filter(outbox_entity::Column::Email.eq(email.to_string()))
        .filter(outbox_entity::Column::Used.eq(false))
        .filter(outbox_entity::Column::ExpiresAt.gt(Utc::now().to_rfc3339()))
        .order_by_desc(outbox_entity::Column::CreatedAt)
        .one(db)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or_else(|| AppError::NotFound("No valid magic link found".to_string()))?;

    Ok(TestInboxLinkPreview {
        email: record.email,
        magic_link: record.magic_link,
        requested_at: record.created_at,
        expires_at: record.expires_at,
    })
}
