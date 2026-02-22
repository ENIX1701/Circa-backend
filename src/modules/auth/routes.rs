use crate::{
    auth::{middleware::jwt_validator, models::Claims, service::generate_jwt},
    modules::auth::models::LoginRequest,
    user::service::UserService,
};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, web};
use actix_web_httpauth::middleware::HttpAuthentication;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").route("/login", web::post().to(login)));

    let auth_middleware = HttpAuthentication::bearer(jwt_validator);
    cfg.service(
        web::scope("/api")
            .wrap(auth_middleware)
            .route("/me", web::get().to(get_current_user)),
    );
}

// TODO: fix? x3
async fn login(
    body: web::Json<LoginRequest>,
    jwt_secret: web::Data<String>,
    user_service: web::Data<UserService>,
) -> impl Responder {
    let user = match user_service.get_user_by_email(&body.email).await {
        Ok(user) => user,
        Err(_) => return HttpResponse::Unauthorized().body("User not found"),
    };

    match generate_jwt(&body.email, user.role.as_str(), &jwt_secret).await {
        Ok(token_response) => HttpResponse::Ok().json(token_response),
        Err(_) => HttpResponse::InternalServerError().body("Failed to generate JWT"),
    }
}

async fn get_current_user(req: HttpRequest) -> impl Responder {
    if let Some(claims) = req.extensions().get::<Claims>() {
        HttpResponse::Ok().body(format!("Hello {}! Your token is valid :3", claims.sub))
    } else {
        HttpResponse::Unauthorized().body("Claims not found")
    }
}
