use crate::{
    auth::{
        middleware::jwt_validator,
        models::{Claims, MagicLinkRequest, VerifyRequest},
        service::{create_magic_token, generate_jwt, verify_magic_token},
    },
    user::service::UserService,
};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use actix_web_httpauth::middleware::HttpAuthentication;
use sea_orm::DatabaseConnection;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/request-link", web::post().to(request_magic_link))
            .route("/verify", web::get().to(verify)),
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
    frontend_url: web::Data<String>,
) -> impl Responder {
    let user = match user_service.get_user_by_email(&body.email).await {
        Ok(u) => u,
        Err(_) => {
            // for now it's okay to verbosely return a 404, but
            // TODO: change to always return 200 with "if email exists, the link has been sent there"
            // to prevent account/email enumeration
            return HttpResponse::NotFound().json(serde_json::json!({"message": "Not found :c"}));
        }
    };

    match create_magic_token(db.get_ref(), &user.id, &frontend_url).await {
        Ok(resp) => HttpResponse::Ok().json(resp),
        Err(_) => HttpResponse::InternalServerError().body("Failed to create a magic link QwQ"),
    }
}

async fn verify(
    query: web::Query<VerifyRequest>,
    jwt_secret: web::Data<String>,
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

    match generate_jwt(&user.id, user.role.as_str(), &jwt_secret).await {
        Ok(token_response) => HttpResponse::Ok().json(token_response),
        Err(_) => HttpResponse::InternalServerError().body("Failed to generate JWT"),
    }
}

async fn get_current_user(req: HttpRequest) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        HttpResponse::Ok().json(serde_json::json!({"sub": claims.sub, "role": claims.role}))
    } else {
        HttpResponse::Unauthorized().body("Claims not found")
    }
}
