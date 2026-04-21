use crate::{
    auth::{
        delivery::MagicLinkDelivery,
        middleware::jwt_validator,
        models::{Claims, MagicLinkRequest, TestInboxQuery, VerifyRequest},
        service::{
            create_magic_token, generate_jwt, generic_magic_link_response,
            get_latest_test_inbox_link, verify_magic_token,
        },
    },
    config::Config,
    error::AppError,
    user::service::UserService,
};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/request-link", web::post().to(request_magic_link))
            .route("/verify", web::get().to(verify))
            .route("/test-inbox/latest", web::get().to(test_inbox_latest)),
    );

    let auth_middleware = HttpAuthentication::bearer(jwt_validator);
    cfg.service(
        web::scope("/api")
            .wrap(auth_middleware)
            .route("/me", web::get().to(get_current_user)),
    );
}

async fn request_magic_link(
    body: web::Json<MagicLinkRequest>,
    user_service: web::Data<UserService>,
    db: web::Data<DatabaseConnection>,
    config: web::Data<Config>,
    delivery: web::Data<Arc<dyn MagicLinkDelivery>>,
) -> Result<HttpResponse, AppError> {
    let email = body.email.trim();

    if email.is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }

    let user = match user_service.get_user_by_email(email).await {
        Ok(user) => Some(user),
        Err(AppError::NotFound(_)) => None,
        Err(err) => return Err(err),
    };

    if let Some(user) = user {
        create_magic_token(
            db.get_ref(),
            delivery.get_ref().as_ref(),
            &user.id,
            &user.email,
            &config.frontend_url,
        )
        .await?;
    }

    Ok(HttpResponse::Ok().json(generic_magic_link_response()))
}

async fn verify(
    query: web::Query<VerifyRequest>,
    config: web::Data<Config>,
    user_service: web::Data<UserService>,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let user_id = match verify_magic_token(db.get_ref(), &query.token).await {
        Ok(uid) => uid,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    let user = match user_service.get_user(&user_id).await {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::InternalServerError().body("User not found after verification");
        }
    };

    match generate_jwt(&user.id, user.role.as_str(), &config.jwt_secret).await {
        Ok(token_response) => HttpResponse::Ok().json(token_response),
        Err(_) => HttpResponse::InternalServerError().body("Failed to generate JWT"),
    }
}

async fn test_inbox_latest(
    query: web::Query<TestInboxQuery>,
    config: web::Data<Config>,
    db: web::Data<DatabaseConnection>,
) -> impl Responder {
    let email = query.email.trim();

    if email.is_empty() {
        return HttpResponse::BadRequest().body("Email is required");
    }

    if !config.test_inbox_enabled() {
        return HttpResponse::NotFound().body("Test inbox unavailable");
    }

    match get_latest_test_inbox_link(db.get_ref(), email).await {
        Ok(preview) => HttpResponse::Ok().json(preview),
        Err(AppError::NotFound(message)) => HttpResponse::NotFound().body(message),
        Err(_) => HttpResponse::InternalServerError().body("Failed to load test inbox"),
    }
}

async fn get_current_user(req: HttpRequest) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        HttpResponse::Ok().json(serde_json::json!({"sub": claims.sub, "role": claims.role}))
    } else {
        HttpResponse::Unauthorized().body("Claims not found")
    }
}
