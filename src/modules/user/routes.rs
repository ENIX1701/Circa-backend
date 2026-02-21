use crate::error::AppError;
use crate::modules::user::models::{CreateUserRequest, UpdateUserRequest};
use crate::modules::user::service::UserService;
use actix_web::{HttpResponse, web};

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("", web::get().to(get_users))
            .route("", web::post().to(create_user))
            .route("/{id}", web::get().to(get_user))
            .route("/{id}", web::patch().to(update_user))
            .route("/{id}", web::delete().to(delete_user)),
    );
}

async fn get_users(service: web::Data<UserService>) -> Result<HttpResponse, AppError> {
    let users = service.get_users().await?;
    Ok(HttpResponse::Ok().json(users))
}

async fn create_user(
    service: web::Data<UserService>,
    body: web::Json<CreateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let user = service.create_user(body.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}

async fn get_user(
    service: web::Data<UserService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let user = service.get_user(&path.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}

async fn update_user(
    service: web::Data<UserService>,
    path: web::Path<String>,
    body: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let user = service
        .update_user(&path.into_inner(), body.into_inner())
        .await?;
    Ok(HttpResponse::Ok().json(user))
}

async fn delete_user(
    service: web::Data<UserService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    service.delete_user(&path.into_inner()).await?;
    Ok(HttpResponse::Ok().body("User deleted successfully"))
}
