use crate::auth::models::{Claims, TokenResponse};
use jsonwebtoken::{EncodingKey, Header, encode};
use std::time::{SystemTime, UNIX_EPOCH};

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
