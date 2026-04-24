use super::{
    entity, membership_entity,
    models::{
        CreateEventRequest, CreatePlannerItemRequest, CreateSocialMediaPostRequest, Event,
        EventBranding, PlannerItem, SocialPost, UpdatePlannerItemRequest,
        UpdateSocialMediaPostRequest, UpsertEventBrandingRequest, EventExport, 
    },
    repository::EventRepository,
};
use crate::error::AppError;
use chrono::{DateTime, Utc};

pub struct EventService {
    repository: EventRepository,
}

impl EventService {
    pub fn new(repository: EventRepository) -> Self {
        Self { repository }
    }

    pub async fn get_events_for_user(&self, user_id: &str) -> Result<Vec<Event>, AppError> {
        let events = self.repository.list_for_user(user_id).await?;

        Ok(events
            .into_iter()
            .map(|(event, role)| Event::from_parts(event, role))
            .collect())
    }

    pub async fn get_event_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Event, AppError> {
        let (event, role) = self
            .repository
            .find_for_user(event_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Event not found".to_string()))?;

        Ok(Event::from_parts(event, role))
    }

    pub async fn create_event(
        &self,
        req: CreateEventRequest,
        user_id: &str,
    ) -> Result<Event, AppError> {
        self.validate_create_request(&req)?;

        let (event, role) = self.repository.create(req, user_id).await?;
        Ok(Event::from_parts(event, role))
    }

    pub async fn activate_event(&self, event_id: &str, user_id: &str) -> Result<Event, AppError> {
        self.require_owner(event_id, user_id).await?;

        let current = self.get_event_for_user(event_id, user_id).await?;
        if !matches!(current.status, super::models::EventStatus::Draft) {
            return Err(AppError::BadRequest(
                "Only draft events can be activated".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_status(event_id, entity::Status::Active, None)
            .await?;

        Ok(Event::from_parts(updated, membership_entity::Role::Owner))
    }

    pub async fn close_event(&self, event_id: &str, user_id: &str) -> Result<Event, AppError> {
        self.require_owner(event_id, user_id).await?;

        let current = self.get_event_for_user(event_id, user_id).await?;
        if !matches!(current.status, super::models::EventStatus::Active) {
            return Err(AppError::BadRequest(
                "Only active events can be closed".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_status(event_id, entity::Status::Closed, None)
            .await?;

        Ok(Event::from_parts(updated, membership_entity::Role::Owner))
    }

    pub async fn request_destruction(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Event, AppError> {
        self.require_owner(event_id, user_id).await?;

        let current = self.get_event_for_user(event_id, user_id).await?;
        if !matches!(current.status, super::models::EventStatus::Closed) {
            return Err(AppError::BadRequest(
                "Only closed events ca nenter pending destruction".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_status(
                event_id,
                entity::Status::PendingDestruction,
                Some(Utc::now().to_rfc3339()),
            )
            .await?;

        Ok(Event::from_parts(updated, membership_entity::Role::Owner))
    }

    pub async fn cancel_destruction(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Event, AppError> {
        self.require_owner(event_id, user_id).await?;

        let current = self.get_event_for_user(event_id, user_id).await?;
        if !matches!(
            current.status,
            super::models::EventStatus::PendingDestruction
        ) {
            return Err(AppError::BadRequest(
                "Only pending destruction events can be restored to closed".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_status(event_id, entity::Status::Closed, None)
            .await?;

        Ok(Event::from_parts(updated, membership_entity::Role::Owner))
    }

    pub async fn archive_event(&self, event_id: &str, user_id: &str) -> Result<Event, AppError> {
        self.require_owner(event_id, user_id).await?;

        let current = self.get_event_for_user(event_id, user_id).await?;
        if !matches!(current.status, super::models::EventStatus::Closed | super::models::EventStatus::PendingDestruction) {
            return Err(AppError::BadRequest("Only closed or pending destruction events can be archived :c".to_string()));
        }

        let updated = self.repository.update_status(event_id, entity::Status::Archived, None).await?;

        Ok(Event::from_parts(updated, membership_entity::Role::Owner))
    }

    pub async fn export_event(&self, event_id: &str, user_id: &str) -> Result<EventExport, AppError> {
        let event = self.get_event_for_user(event_id, user_id).await?;
        let branding = self.get_event_branding_for_user(event_id, user_id).await?;
        let planner_items = self.get_planner_items_for_user(event_id, user_id).await?;
        let social_posts = self.get_social_posts_for_user(event_id, user_id).await?;

        Ok(EventExport {
            exported_at: Utc::now().to_rfc3339(),
            event,
            branding,
            planner_items,
            social_posts,
        })
    }

    pub async fn get_event_branding_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<EventBranding, AppError> {
        self.require_event_access(event_id, user_id).await?;

        let branding = self.repository.find_event_branding(event_id).await?;

        Ok(match branding {
            Some(model) => EventBranding::from_model(model),
            None => EventBranding::default_for_event(event_id),
        })
    }

    pub async fn upsert_event_branding(
        &self,
        event_id: &str,
        user_id: &str,
        req: UpsertEventBrandingRequest,
    ) -> Result<EventBranding, AppError> {
        self.require_event_access(event_id, user_id).await?;
        self.validate_upsert_event_branding_request(&req)?;

        let branding = self.repository.upsert_event_branding(event_id, req).await?;
        Ok(EventBranding::from_model(branding))
    }

    pub async fn get_planner_items_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Vec<PlannerItem>, AppError> {
        self.require_event_access(event_id, user_id).await?;

        let items = self.repository.list_planner_items(event_id).await?;
        Ok(items.into_iter().map(PlannerItem::from).collect())
    }

    pub async fn create_planner_item(
        &self,
        event_id: &str,
        user_id: &str,
        req: CreatePlannerItemRequest,
    ) -> Result<PlannerItem, AppError> {
        self.require_event_access(event_id, user_id).await?;
        self.validate_create_planner_item_request(&req)?;

        let item = self.repository.create_planner_item(event_id, req).await?;
        Ok(item.into())
    }

    pub async fn update_planner_item(
        &self,
        event_id: &str,
        item_id: &str,
        user_id: &str,
        req: UpdatePlannerItemRequest,
    ) -> Result<PlannerItem, AppError> {
        self.require_event_access(event_id, user_id).await?;
        self.validate_update_planner_item_request(&req)?;

        let item = self
            .repository
            .update_planner_item(event_id, item_id, req)
            .await?;

        Ok(item.into())
    }

    pub async fn delete_planner_item(
        &self,
        event_id: &str,
        item_id: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        self.require_event_access(event_id, user_id).await?;
        self.repository.delete_planner_item(event_id, item_id).await
    }

    pub async fn get_social_posts_for_user(&self, event_id: &str, user_id: &str) -> Result<Vec<SocialPost>, AppError> {
        self.require_event_access(event_id, user_id).await?;

        let posts = self.repository.list_social_posts(event_id).await?;
        Ok(posts.into_iter().map(SocialPost::from).collect())
    }

    pub async fn create_social_post(
        &self,
        event_id: &str,
        user_id: &str,
        req: CreateSocialMediaPostRequest,
    ) -> Result<SocialPost, AppError> {
        self.require_event_access(event_id, user_id).await?;
        self.validate_create_social_post_request(&req)?;

        let post = self.repository.create_social_post(event_id, req).await?;
        Ok(post.into())
    }

    pub async fn update_social_post(
        &self,
        event_id: &str,
        post_id: &str,
        user_id: &str,
        req: UpdateSocialMediaPostRequest,
    ) -> Result<SocialPost, AppError> {
        self.require_event_access(event_id, user_id).await?;
        self.validate_update_social_post_request(&req)?;

        let post = self.repository.update_social_post(event_id, post_id, req).await?;

        Ok(post.into())
    }

    pub async fn delete_social_post(
        &self, event_id: &str, post_id: &str, user_id: &str
    ) -> Result<(), AppError> {
        self.require_event_access(event_id, user_id).await?;
        self.repository.delete_social_post(event_id, post_id).await
    }

    fn validate_create_request(&self, req: &CreateEventRequest) -> Result<(), AppError> {
        if req.name.trim().is_empty() {
            return Err(AppError::BadRequest("Event name is required".to_string()));
        }

        if req.slug.trim().is_empty() {
            return Err(AppError::BadRequest("Event slug is required".to_string()));
        }

        if !is_valid_slug(&req.slug) {
            return Err(AppError::BadRequest(
                "Event slug must contain only lowercase letters, numbers and huphens".to_string(),
            ));
        }

        if req.venue.trim().is_empty() {
            return Err(AppError::BadRequest("Venue is required".to_string()));
        }

        if req.timezone.trim().is_empty() {
            return Err(AppError::BadRequest("Timezone is required".to_string()));
        }

        let starts_at = DateTime::parse_from_rfc3339(&req.starts_at).map_err(|_| {
            AppError::BadRequest("starts_at must be a valid RFC3339 datetime".to_string())
        })?;

        let ends_at = DateTime::parse_from_rfc3339(&req.ends_at).map_err(|_| {
            AppError::BadRequest("ends_at must be a valid RFC3339 datetime".to_string())
        })?;

        if starts_at >= ends_at {
            return Err(AppError::BadRequest(
                "ends_at must be later than starts_at".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_create_planner_item_request(
        &self,
        req: &CreatePlannerItemRequest,
    ) -> Result<(), AppError> {
        if req.title.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Planner item title is required".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_upsert_event_branding_request(
        &self,
        req: &UpsertEventBrandingRequest,
    ) -> Result<(), AppError> {
        if !req.primary_color.is_empty() && !is_valid_hex_color(&req.primary_color) {
            return Err(AppError::BadRequest(
                "primary_color must be a valid hex color".to_string(),
            ));
        }

        if !req.secondary_color.is_empty() && !is_valid_hex_color(&req.secondary_color) {
            return Err(AppError::BadRequest(
                "secondary_color must be a valid hex color".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_update_planner_item_request(
        &self,
        req: &UpdatePlannerItemRequest,
    ) -> Result<(), AppError> {
        if req.title.is_none()
            && req.notes.is_none()
            && req.position.is_none()
            && req.done.is_none()
        {
            return Err(AppError::BadRequest(
                "At least one field must be provided".to_string(),
            ));
        }

        if let Some(title) = &req.title {
            if title.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Planner item title cannot be empty".to_string(),
                ));
            }
        }

        if let Some(position) = req.position {
            if position < 0 {
                return Err(AppError::BadRequest(
                    "Planner item position cannot be negative".to_string(),
                ));
            }
        }

        Ok(())
    }

    fn validate_create_social_post_request(
        &self,
        req: &CreateSocialMediaPostRequest,
    ) -> Result<(), AppError> {
        if req.platform.trim().is_empty() {
            return Err(AppError::BadRequest("Social post platform is requried".to_string()));
        }
        
        if req.title.trim().is_empty() {
            return Err(AppError::BadRequest("Social post title is requried".to_string()));
        }

        Ok(())
    }

    fn validate_update_social_post_request(
        &self,
        req: &UpdateSocialMediaPostRequest,
    ) -> Result<(), AppError> {
        if req.platform.is_none()
            && req.title.is_none()
            && req.body.is_none()
            && req.status.is_none()
            && req.position.is_none()
        {
            return Err(AppError::BadRequest(
                "At least one social post field must be provided".to_string(),
            ));
        }

        if let Some(platform) = &req.platform {
            if platform.trim().is_empty() {
                return Err(AppError::BadRequest("Social post platform cannot be empty".to_string()));
            }
        }

        if let Some(title) = &req.title {
            if title.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "Social post title cannot be empty".to_string(),
                ));
            }
        }

        if let Some(status) = &req.status {
            if !is_valid_social_post_status(status) {
                return Err(AppError::BadRequest("Social post status must be draft, ready or posted".to_string()));
            }
        }

        if let Some(position) = req.position {
            if position < 0 {
                return Err(AppError::BadRequest(
                    "Social post position cannot be negative".to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn require_owner(&self, event_id: &str, user_id: &str) -> Result<(), AppError> {
        let membership = self.repository.find_membership(event_id, user_id).await?;

        match membership {
            Some(membership) if membership.role == membership_entity::Role::Owner => Ok(()),
            Some(_) => Err(AppError::Forbidden),
            None => Err(AppError::NotFound("Event not found".to_string())),
        }
    }

    async fn require_event_access(&self, event_id: &str, user_id: &str) -> Result<(), AppError> {
        let membership = self.repository.find_membership(event_id, user_id).await?;

        match membership {
            Some(_) => Ok(()),
            None => Err(AppError::NotFound("Event not found".to_string())),
        }
    }
}

fn is_valid_slug(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn is_valid_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value.chars().skip(1).all(|c| c.is_ascii_hexdigit())
}

fn is_valid_social_post_status(value: &str) -> bool {
    matches!(value, "draft" | "ready" | "posted")
}
