use actix_web::{Error, HttpMessage, dev::ServiceRequest, error::ErrorUnauthorized};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::auth::models::Claims;

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    let validation = Validation::default();

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(b"TODO: put actual key here x3"),
        &validation,
    ) {
        Ok(token_data) => {
            req.extensions_mut().insert(token_data.claims);
            Ok(req)
        }
        Err(_) => Err((ErrorUnauthorized("Invalid or expired token"), req)),
    }
}
