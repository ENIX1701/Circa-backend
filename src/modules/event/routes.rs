use actix_web::HttpMessage;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::{
    auth::{middleware::jwt_validator, models::Claims},
    error::AppError,
    event::{models::CreateEventRequest, service::EventService},
};

pub fn config(cfg: &mut web::ServiceConfig) {
    let auth_middleware = HttpAuthentication::bearer(jwt_validator);

    cfg.service(
        web::scope("/events")
            .wrap(auth_middleware)
            .route("", web::get().to(get_events))
            .route("", web::post().to(create_event))
            .route("/{id}", web::get().to(get_event))
            .route("/{id}/activate", web::post().to(activate_event))
            .route("/{id}/close", web::post().to(close_event))
            .route(
                "/{id}/request-destruction",
                web::post().to(request_destruction),
            )
            .route(
                "/{id}/cancel-destruction",
                web::post().to(cancel_destruction),
            ),
    );
}

async fn get_events(
    req: HttpRequest,
    service: web::Data<EventService>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let events = service.get_events_for_user(&claims.sub).await?;
    Ok(HttpResponse::Ok().json(events))
}

async fn create_event(
    req: HttpRequest,
    service: web::Data<EventService>,
    body: web::Json<CreateEventRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let event = service.create_event(body.into_inner(), &claims.sub).await?;
    Ok(HttpResponse::Ok().json(event))
}

async fn get_event(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let event = service
        .get_event_for_user(&path.into_inner(), &claims.sub)
        .await?;
    Ok(HttpResponse::Ok().json(event))
}

async fn activate_event(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let event = service
        .activate_event(&path.into_inner(), &claims.sub)
        .await?;
    Ok(HttpResponse::Ok().json(event))
}

async fn close_event(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let event = service.close_event(&path.into_inner(), &claims.sub).await?;
    Ok(HttpResponse::Ok().json(event))
}

async fn request_destruction(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let event = service
        .request_destruction(&path.into_inner(), &claims.sub)
        .await?;
    Ok(HttpResponse::Ok().json(event))
}

async fn cancel_destruction(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let event = service
        .cancel_destruction(&path.into_inner(), &claims.sub)
        .await?;
    Ok(HttpResponse::Ok().json(event))
}
