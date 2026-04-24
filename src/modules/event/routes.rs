use actix_web::HttpMessage;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_httpauth::middleware::HttpAuthentication;

use crate::{
    auth::{middleware::jwt_validator, models::Claims},
    error::AppError,
    event::{
        models::{
            CreateEventRequest, CreatePlannerItemRequest, CreateSocialMediaPostRequest,
            UpdatePlannerItemRequest, UpdateSocialMediaPostRequest, UpsertEventBrandingRequest,
        },
        service::EventService,
    },
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
            )
            .route("/{id}/branding", web::get().to(get_event_branding))
            .route("/{id}/branding", web::put().to(put_event_branding))
            .route("/{id}/social-posts", web::get().to(get_social_posts))
            .route("/{id}/social-posts", web::post().to(create_social_post))
            .route("/{id}/social-posts/{post_id}", web::patch().to(update_social_post))
            .route("/{id}/social-posts/{post_id}", web::delete().to(delete_social_post))
            .route("/{id}/planner-items", web::get().to(get_planner_items))
            .route("/{id}/planner-items", web::post().to(create_planner_item))
            .route(
                "/{id}/planner-items/{item_id}",
                web::patch().to(update_planner_item),
            )
            .route(
                "/{id}/planner-items/{item_id}",
                web::delete().to(delete_planner_item),
            )
            .route("/{id}/archive", web::post().to(archive_event))
            .route("/{id}/export", web::get().to(export_event)),
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

async fn get_event_branding(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let branding = service
        .get_event_branding_for_user(&path.into_inner(), &claims.sub)
        .await?;

    Ok(HttpResponse::Ok().json(branding))
}

async fn put_event_branding(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
    body: web::Json<UpsertEventBrandingRequest>
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let branding = service
        .upsert_event_branding(&path.into_inner(), &claims.sub, body.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(branding))
}

async fn get_social_posts(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let posts = service.get_social_posts_for_user(&path.into_inner(), &claims.sub).await?;

    Ok(HttpResponse::Ok().json(posts))
}

async fn create_social_post(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
    body: web::Json<CreateSocialMediaPostRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let post = service.create_social_post(&path.into_inner(), &claims.sub, body.into_inner()).await?;

    Ok(HttpResponse::Ok().json(post))
}

async fn update_social_post(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<(String, String)>,
    body: web::Json<UpdateSocialMediaPostRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let (event_id, post_id) = path.into_inner();

    let post = service.update_social_post(&event_id, &post_id, &claims.sub, body.into_inner()).await?;

    Ok(HttpResponse::Ok().json(post))
}

async fn delete_social_post(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let (event_id, post_id) = path.into_inner();

    service
        .delete_social_post(&event_id, &post_id, &claims.sub)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

async fn get_planner_items(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let items = service
        .get_planner_items_for_user(&path.into_inner(), &claims.sub)
        .await?;

    Ok(HttpResponse::Ok().json(items))
}

async fn create_planner_item(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
    body: web::Json<CreatePlannerItemRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let item = service
        .create_planner_item(&path.into_inner(), &claims.sub, body.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(item))
}

async fn update_planner_item(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<(String, String)>,
    body: web::Json<UpdatePlannerItemRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let (event_id, item_id) = path.into_inner();

    let item = service
        .update_planner_item(&event_id, &item_id, &claims.sub, body.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(item))
}

async fn delete_planner_item(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let (event_id, item_id) = path.into_inner();

    service
        .delete_planner_item(&event_id, &item_id, &claims.sub)
        .await?;

    Ok(HttpResponse::NoContent().finish())
}

async fn archive_event(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let event = service.archive_event(&path.into_inner(), &claims.sub).await?;

    Ok(HttpResponse::Ok().json(event))
}

async fn export_event(
    req: HttpRequest,
    service: web::Data<EventService>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let claims = req
        .extensions()
        .get::<Claims>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    let export = service.export_event(&path.into_inner(), &claims.sub).await?;

    Ok(HttpResponse::Ok().json(export))
}
