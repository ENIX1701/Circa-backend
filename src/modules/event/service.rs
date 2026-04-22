use super::{
    entity, membership_entity,
    models::{CreateEventRequest, Event},
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

    async fn require_owner(&self, event_id: &str, user_id: &str) -> Result<(), AppError> {
        let membership = self.repository.find_membership(event_id, user_id).await?;

        match membership {
            Some(membership) if membership.role == membership_entity::Role::Owner => Ok(()),
            Some(_) => Err(AppError::Forbidden),
            None => Err(AppError::NotFound("Event not found".to_string())),
        }
    }
}

fn is_valid_slug(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}
