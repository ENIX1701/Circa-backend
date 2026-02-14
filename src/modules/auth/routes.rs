use crate::modules::auth::models::{LoginRequest, RegisterRequest};
use actix_web::{HttpResponse, Responder, web};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/login", web::post.to(login))
            .route("/register", web::post.to(register))
            .route("/logout", web::post.to(logout)),
    )
}

async fn login(body: web::Json<LoginRequest>, data: web::Data<AppState>) -> impl Responder {
    //
}
