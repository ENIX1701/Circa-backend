use crate::error::AppError;
use crate::modules::user::models::{
    CreateUserRequest, UpdateUserRequest, User, UserRole, UserStatus,
};
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

// get all
async fn get_users() -> Result<HttpResponse, AppError> {
    // TODO: fetch from db
    // mock for now
    let users = vec![
        User {
            id: "1".to_string(),
            name: "John".to_string(),
            surname: "Smith".to_string(),
            role: UserRole::EventDirector,
            email: "john@event.com".to_string(),
            phone: "+1 234 567 890".to_string(),
            status: UserStatus::Active,
        },
        User {
            id: "2".to_string(),
            name: "Michael".to_string(),
            surname: "Brown".to_string(),
            role: UserRole::BoothOwner,
            email: "michael@event.com".to_string(),
            phone: "+22 987 654 321".to_string(),
            status: UserStatus::Active,
        },
        User {
            id: "3".to_string(),
            name: "Teressa".to_string(),
            surname: "Birkenstock".to_string(),
            role: UserRole::Clown,
            email: "clown@event.com".to_string(),
            phone: "+123 33 44 112".to_string(),
            status: UserStatus::Inactive,
        },
    ];

    Ok(HttpResponse::Ok().json(users))
}

// post new
async fn create_user(body: web::Json<CreateUserRequest>) -> Result<HttpResponse, AppError> {
    if body.email.is_empty() {
        return Err(AppError::BadRequest("Email is required".to_string()));
    }

    // TODO: save to db
    let user = User {
        id: "1".to_string(),
        name: body.name.clone(),
        surname: body.surname.clone(),
        role: body.role.clone(),
        email: body.email.clone(),
        phone: body.phone.clone(),
        status: UserStatus::Active,
    };

    Ok(HttpResponse::Ok().json(user))
}

// get {id}
async fn get_user(path: web::Path<String>) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    // TODO: fetch user from db

    Err(AppError::NotFound(
        "User not found! (We don't have a db QwQ)".to_string(),
    ))
}

// patch
async fn update_user(
    path: web::Path<String>,
    body: web::Json<UpdateUserRequest>,
) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    Ok(HttpResponse::Ok().body("updated UwU (not really)"))
}

// delete
async fn delete_user(path: web::Path<String>) -> Result<HttpResponse, AppError> {
    let user_id = path.into_inner();
    Ok(HttpResponse::Ok().body("deleted QwQ (not really)"))
}
