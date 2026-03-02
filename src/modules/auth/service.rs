use crate::{
    auth::models::{Claims, MagicLinkResponse, TokenResponse},
    error::AppError,
};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::RngExt;
use sea_orm::*;
use std::time::{SystemTime, UNIX_EPOCH};

use super::entity::{self as magic_entity, Entity as MagicTokenEntity};

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
    user_id: &str,
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
        id: Set(id),
        user_id: Set(user_id.to_string()),
        token: Set(token.clone()),
        expires_at: Set(expires_at),
        used: Set(false),
    };

    record
        .insert(db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let link = format!("{}/login?token={}", frontend_url, token);
    println!("[MAGIC LINK] {}", link);

    Ok(MagicLinkResponse {
        message: "Magic link sent :3 check your inbox! (server console for now...)".to_string(),
    })
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

    if Utc::now() > expires_at {
        return Err(AppError::BadRequest("Magic link has expired".to_string()));
    }

    let mut active: magic_entity::ActiveModel = record.clone().into();
    active.used = Set(true);
    active
        .update(db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    Ok(record.user_id)
}
