use actix_web::{Error, HttpMessage, dev::ServiceRequest, error::ErrorUnauthorized, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::{auth::models::Claims, config::Config};

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    let validation = Validation::default();

    let config = req
        .app_data::<web::Data<Config>>()
        .expect("Config not found in app state");

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(_) => Err((ErrorUnauthorized("Invalid or expired token"), req)),
    }
}
