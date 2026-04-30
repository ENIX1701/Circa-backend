use super::{
    entity, membership_entity,
    models::{
        AddEventCollaboratorRequest, CreateEventRequest, CreatePlannerItemRequest,
        CreatePlannerTimelineItemRequest, CreateSocialMediaPostRequest, Event, EventBranding,
        EventCollaborator, EventExport, PlannerItem, PlannerTimelineItem, SocialPost,
        UpdateEventCollaboratorRequest, UpdatePlannerItemRequest, UpdatePlannerTimelineItemRequest,
        UpdateSocialMediaPostRequest, UpsertEventBrandingRequest,
    },
    repository::EventRepository,
};
use crate::error::AppError;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

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

    pub async fn is_slug_available(&self, slug: &str) -> Result<bool, AppError> {
        let slug = slug.trim();

        if slug.is_empty() {
            return Err(AppError::BadRequest(
                "Event slug is required >:c".to_string(),
            ));
        }

        if !is_valid_slug(slug) {
            return Err(AppError::BadRequest(
                "Event slug must contain lowercase letters, numbers and hyphens only!".to_string(),
            ));
        }

        Ok(!self.repository.slug_exists(slug).await?)
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

        let event = self.require_owner(event_id, user_id).await?;
        if event.status != entity::Status::Draft {
            return Err(AppError::BadRequest(
                "Only draft events can be activated".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_status(&event.id, entity::Status::Active, None)
            .await?;

        Ok(Event::from_parts(updated, membership_entity::Role::Owner))
    }

    pub async fn close_event(&self, event_id: &str, user_id: &str) -> Result<Event, AppError> {
        self.require_owner(event_id, user_id).await?;

        let event = self.require_owner(event_id, user_id).await?;
        if event.status != entity::Status::Active {
            return Err(AppError::BadRequest(
                "Only active events can be closed".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_status(&event.id, entity::Status::Closed, None)
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

        let event = self.require_owner(event_id, user_id).await?;
        if event.status != entity::Status::Closed
            && event.status != entity::Status::PendingDestruction
        {
            return Err(AppError::BadRequest(
                "Only pending destruction events can be restored to closed".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_status(&event.id, entity::Status::Closed, None)
            .await?;

        Ok(Event::from_parts(updated, membership_entity::Role::Owner))
    }

    pub async fn archive_event(&self, event_id: &str, user_id: &str) -> Result<Event, AppError> {
        self.require_owner(event_id, user_id).await?;

        let current = self.get_event_for_user(event_id, user_id).await?;
        if !matches!(
            current.status,
            super::models::EventStatus::Closed | super::models::EventStatus::PendingDestruction
        ) {
            return Err(AppError::BadRequest(
                "Only closed or pending destruction events can be archived :c".to_string(),
            ));
        }

        let updated = self
            .repository
            .update_status(event_id, entity::Status::Archived, None)
            .await?;

        Ok(Event::from_parts(updated, membership_entity::Role::Owner))
    }

    pub async fn export_event(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<EventExport, AppError> {
        let event = self.get_event_for_user(event_id, user_id).await?;
        let branding = self.get_event_branding_for_user(event_id, user_id).await?;
        let planner_items = self.get_planner_items_for_user(event_id, user_id).await?;
        let social_posts = self.get_social_posts_for_user(event_id, user_id).await?;
        let planner_timeline_items = self
            .get_planner_timeline_items_for_user(event_id, user_id)
            .await?;

        Ok(EventExport {
            exported_at: Utc::now().to_rfc3339(),
            event,
            branding,
            planner_items,
            social_posts,
            planner_timeline_items,
        })
    }

    pub async fn get_event_collaborators_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Vec<EventCollaborator>, AppError> {
        let (event, _) = self.require_event_access(event_id, user_id).await?;

        let collaborators = self.repository.list_collaborators(&event.id).await?;

        Ok(collaborators
            .into_iter()
            .map(|(membership, user)| EventCollaborator::from_parts(membership, user))
            .collect())
    }

    pub async fn add_event_collaborator(
        &self,
        event_id: &str,
        user_id: &str,
        req: AddEventCollaboratorRequest,
    ) -> Result<EventCollaborator, AppError> {
        self.require_owner(event_id, user_id).await?;

        let email = req.email.trim().to_lowercase();

        if email.is_empty() {
            return Err(AppError::BadRequest("Email is required :c".to_string()));
        }

        let user = self
            .repository
            .find_user_by_email(&email)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found QwQ".to_string()))?;

        let membership = self
            .repository
            .add_membership(event_id, &user.id, req.role.into())
            .await?;

        Ok(EventCollaborator::from_parts(membership, user))
    }

    pub async fn update_event_collaborator(
        &self,
        event_id: &str,
        target_user_id: &str,
        actor_user_id: &str,
        req: UpdateEventCollaboratorRequest,
    ) -> Result<EventCollaborator, AppError> {
        let event = self.require_owner(event_id, actor_user_id).await?;

        if target_user_id == actor_user_id {
            return Err(AppError::BadRequest(
                "You cannot change your own event role :c".to_string(),
            ));
        }

        let current_membership = self
            .repository
            .find_membership(&event.id, target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Collaborator not found".to_string()))?;

        if current_membership.role == membership_entity::Role::Owner
            && self.repository.owner_count(&event.id).await? <= 1
        {
            return Err(AppError::BadRequest(
                "Each event must have at least one owner!!".to_string(),
            ));
        }

        let membership = self
            .repository
            .update_membership_role(&event.id, target_user_id, req.role.into())
            .await?;

        let user = self
            .repository
            .find_user_by_id(target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        Ok(EventCollaborator::from_parts(membership, user))
    }

    pub async fn delete_event_collaborator(
        &self,
        event_id: &str,
        target_user_id: &str,
        actor_user_id: &str,
    ) -> Result<(), AppError> {
        let event = self.require_owner(event_id, actor_user_id).await?;

        if target_user_id == actor_user_id {
            return Err(AppError::BadRequest(
                "You cannot remove yourself from the event >:C".to_string(),
            ));
        }

        let membership = self
            .repository
            .find_membership(&event.id, target_user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Collaborator not found".to_string()))?;

        if membership.role == membership_entity::Role::Owner
            && self.repository.owner_count(&event.id).await? <= 1
        {
            return Err(AppError::BadRequest(
                "Each event must have at least one owner!!".to_string(),
            ));
        }

        self.repository
            .delete_membership(&event.id, target_user_id)
            .await
    }

    pub async fn get_event_branding_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<EventBranding, AppError> {
        let (event, _) = self.require_event_access(event_id, user_id).await?;

        let branding = self.repository.find_event_branding(&event.id).await?;

        Ok(match branding {
            Some(model) => EventBranding::from_model(model),
            None => EventBranding::default_for_event(&event.id),
        })
    }

    pub async fn upsert_event_branding(
        &self,
        event_id: &str,
        user_id: &str,
        req: UpsertEventBrandingRequest,
    ) -> Result<EventBranding, AppError> {
        let event = self.require_event_manager(event_id, user_id).await?;
        self.validate_upsert_event_branding_request(&req)?;

        let branding = self
            .repository
            .upsert_event_branding(&event.id, req)
            .await?;
        Ok(EventBranding::from_model(branding))
    }

    pub async fn get_planner_items_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Vec<PlannerItem>, AppError> {
        let (event, _) = self.require_event_access(event_id, user_id).await?;

        let items = self.repository.list_planner_items(&event.id).await?;
        Ok(items.into_iter().map(PlannerItem::from).collect())
    }

    pub async fn create_planner_item(
        &self,
        event_id: &str,
        user_id: &str,
        req: CreatePlannerItemRequest,
    ) -> Result<PlannerItem, AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.validate_create_planner_item_request(&req)?;

        let item = self.repository.create_planner_item(&event.id, req).await?;
        Ok(item.into())
    }

    pub async fn update_planner_item(
        &self,
        event_id: &str,
        item_id: &str,
        user_id: &str,
        req: UpdatePlannerItemRequest,
    ) -> Result<PlannerItem, AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.validate_update_planner_item_request(&req)?;

        let item = self
            .repository
            .update_planner_item(&event.id, item_id, req)
            .await?;

        Ok(item.into())
    }

    pub async fn delete_planner_item(
        &self,
        event_id: &str,
        item_id: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.repository
            .delete_planner_item(&event.id, item_id)
            .await
    }

    pub async fn get_social_posts_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Vec<SocialPost>, AppError> {
        let (event, _) = self.require_event_access(event_id, user_id).await?;

        let posts = self.repository.list_social_posts(&event.id).await?;
        Ok(posts.into_iter().map(SocialPost::from).collect())
    }

    pub async fn get_planner_timeline_items_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Vec<PlannerTimelineItem>, AppError> {
        let (event, _) = self.require_event_access(event_id, user_id).await?;

        let items = self
            .repository
            .list_planner_timeline_items(&event.id)
            .await?;
        Ok(items.into_iter().map(PlannerTimelineItem::from).collect())
    }

    pub async fn create_planner_timeline_item(
        &self,
        event_id: &str,
        user_id: &str,
        req: CreatePlannerTimelineItemRequest,
    ) -> Result<PlannerTimelineItem, AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.validate_create_planner_timeline_item_request(&req)?;
        self.validate_timeline_assignee(&event.id, req.assigned_user_id.as_deref())
            .await?;

        let item = self
            .repository
            .create_planner_timeline_item(&event.id, req)
            .await?;
        Ok(item.into())
    }

    pub async fn update_planner_timeline_item(
        &self,
        event_id: &str,
        item_id: &str,
        user_id: &str,
        req: UpdatePlannerTimelineItemRequest,
    ) -> Result<PlannerTimelineItem, AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.validate_update_planner_timeline_item_request(&req)?;
        self.validate_timeline_assignee(&event.id, req.assigned_user_id.as_deref())
            .await?;

        let item = self
            .repository
            .update_planner_timeline_item(&event.id, item_id, req)
            .await?;
        Ok(item.into())
    }

    pub async fn delete_planner_timeline_item(
        &self,
        event_id: &str,
        item_id: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.repository
            .delete_planner_timeline_item(&event.id, item_id)
            .await
    }

    pub async fn create_social_post(
        &self,
        event_id: &str,
        user_id: &str,
        req: CreateSocialMediaPostRequest,
    ) -> Result<SocialPost, AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.validate_create_social_post_request(&req)?;

        let post = self.repository.create_social_post(&event.id, req).await?;
        Ok(post.into())
    }

    pub async fn update_social_post(
        &self,
        event_id: &str,
        post_id: &str,
        user_id: &str,
        req: UpdateSocialMediaPostRequest,
    ) -> Result<SocialPost, AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.validate_update_social_post_request(&req)?;

        let post = self
            .repository
            .update_social_post(&event.id, post_id, req)
            .await?;

        Ok(post.into())
    }

    pub async fn delete_social_post(
        &self,
        event_id: &str,
        post_id: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let event = self.require_content_manager(event_id, user_id).await?;
        self.repository.delete_social_post(&event.id, post_id).await
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

        if req.timezone.trim().parse::<Tz>().is_err() {
            return Err(AppError::BadRequest(
                "Timezone must be a valid IANA timezone".to_string(),
            ));
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

        if ends_at.with_timezone(&Utc) <= Utc::now() {
            return Err(AppError::BadRequest(
                "ends_at cannot be in the past".to_string(),
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

        if !req.background_color.is_empty() && !is_valid_hex_color(&req.background_color) {
            return Err(AppError::BadRequest(
                "background_color must be a valid hex color".to_string(),
            ));
        }

        if !req.theme_mode.is_empty() && !matches!(req.theme_mode.as_str(), "dark" | "light") {
            return Err(AppError::BadRequest(
                "theme_mode must be dark or light".to_string(),
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

        if let Some(title) = &req.title
            && title.trim().is_empty()
        {
            return Err(AppError::BadRequest(
                "Planner item title cannot be empty".to_string(),
            ));
        }

        if let Some(position) = req.position
            && position < 0
        {
            return Err(AppError::BadRequest(
                "Planner item position cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_create_planner_timeline_item_request(
        &self,
        req: &CreatePlannerTimelineItemRequest,
    ) -> Result<(), AppError> {
        if req.title.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Timeline title is required".to_string(),
            ));
        }

        if !is_valid_timeline_item_type(&req.item_type) {
            return Err(AppError::BadRequest(
                "Timeline item type must be task, asset or milestone".to_string(),
            ));
        }

        validate_timeline_dates(&req.starts_at, &req.ends_at)?;

        if let Some(status) = &req.status
            && !is_valid_timeline_status(status)
        {
            return Err(AppError::BadRequest(
                "Timeline status must be planned, in_progress, blocked or done".to_string(),
            ));
        }

        if let Some(color) = &req.color
            && !color.is_empty()
            && !is_valid_hex_color(color)
        {
            return Err(AppError::BadRequest(
                "timeline color must be a valid hex color".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_update_planner_timeline_item_request(
        &self,
        req: &UpdatePlannerTimelineItemRequest,
    ) -> Result<(), AppError> {
        if req.title.is_none()
            && req.item_type.is_none()
            && req.starts_at.is_none()
            && req.ends_at.is_none()
            && req.status.is_none()
            && req.owner.is_none()
            && req.notes.is_none()
            && req.color.is_none()
            && req.position.is_none()
            && req.depends_on_item_id.is_none()
            && req.assigned_user_id.is_none()
        {
            return Err(AppError::BadRequest(
                "At least one timeline field must be provided".to_string(),
            ));
        }

        if let Some(title) = &req.title
            && title.trim().is_empty()
        {
            return Err(AppError::BadRequest(
                "Timeline title cannot be empty".to_string(),
            ));
        }

        if let Some(item_type) = &req.item_type
            && !is_valid_timeline_item_type(item_type)
        {
            return Err(AppError::BadRequest(
                "Timeline item type must be task, asset or milestone".to_string(),
            ));
        }

        if let Some(status) = &req.status
            && !is_valid_timeline_status(status)
        {
            return Err(AppError::BadRequest(
                "Timeline status must planned, in_progress, block or done".to_string(),
            ));
        }

        if let Some(starts_at) = &req.starts_at {
            DateTime::parse_from_rfc3339(starts_at).map_err(|_| {
                AppError::BadRequest("starts_at must be a valid RFC3339 datetime".to_string())
            })?;
        }

        if let Some(ends_at) = &req.ends_at {
            DateTime::parse_from_rfc3339(ends_at).map_err(|_| {
                AppError::BadRequest("ends_at must be a valid RFC3339 datetime".to_string())
            })?;
        }

        if let (Some(starts_at), Some(ends_at)) = (&req.starts_at, &req.ends_at) {
            validate_timeline_dates(starts_at, ends_at)?;
        }

        if let Some(color) = &req.color
            && !color.trim().is_empty()
            && !is_valid_hex_color(color)
        {
            return Err(AppError::BadRequest(
                "Timeline color must be a valid hex".to_string(),
            ));
        }

        Ok(())
    }

    async fn validate_timeline_assignee(
        &self,
        event_id: &str,
        assigned_user_id: Option<&str>,
    ) -> Result<(), AppError> {
        let Some(assigned_user_id) = assigned_user_id else {
            return Ok(());
        };

        if assigned_user_id.trim().is_empty() {
            return Ok(());
        }

        if self
            .repository
            .find_membership(event_id, assigned_user_id)
            .await?
            .is_none()
        {
            return Err(AppError::BadRequest(
                "assigned_user_id must belong to an event collaborator >:c".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_create_social_post_request(
        &self,
        req: &CreateSocialMediaPostRequest,
    ) -> Result<(), AppError> {
        if req.platform.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Social post platform is requried".to_string(),
            ));
        }

        if req.title.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Social post title is requried".to_string(),
            ));
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

        if let Some(platform) = &req.platform
            && platform.trim().is_empty()
        {
            return Err(AppError::BadRequest(
                "Social post platform cannot be empty".to_string(),
            ));
        }

        if let Some(title) = &req.title
            && title.trim().is_empty()
        {
            return Err(AppError::BadRequest(
                "Social post title cannot be empty".to_string(),
            ));
        }

        if let Some(status) = &req.status
            && !is_valid_social_post_status(status)
        {
            return Err(AppError::BadRequest(
                "Social post status must be draft, ready or posted".to_string(),
            ));
        }

        if let Some(position) = req.position
            && position < 0
        {
            return Err(AppError::BadRequest(
                "Social post position cannot be negative".to_string(),
            ));
        }

        Ok(())
    }

    async fn require_owner(
        &self,
        event_ref: &str,
        user_id: &str,
    ) -> Result<entity::Model, AppError> {
        let (event, role) = self.require_event_access(event_ref, user_id).await?;

        if role == membership_entity::Role::Owner {
            Ok(event)
        } else {
            Err(AppError::Forbidden)
        }
    }

    async fn require_event_manager(
        &self,
        event_ref: &str,
        user_id: &str,
    ) -> Result<entity::Model, AppError> {
        let (event, role) = self.require_event_access(event_ref, user_id).await?;

        if role == membership_entity::Role::Owner || role == membership_entity::Role::Organizer {
            Ok(event)
        } else {
            Err(AppError::Forbidden)
        }
    }

    async fn require_content_manager(
        &self,
        event_ref: &str,
        user_id: &str,
    ) -> Result<entity::Model, AppError> {
        let (event, role) = self.require_event_access(event_ref, user_id).await?;

        if role == membership_entity::Role::Owner
            || role == membership_entity::Role::Organizer
            || role == membership_entity::Role::Staff
        {
            Ok(event)
        } else {
            Err(AppError::Forbidden)
        }
    }

    async fn require_event_access(
        &self,
        event_ref: &str,
        user_id: &str,
    ) -> Result<(entity::Model, membership_entity::Role), AppError> {
        self.repository
            .find_for_user(event_ref, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Event not found".to_string()))
    }
}

fn is_valid_slug(value: &str) -> bool {
    let value = value.trim();

    !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && !value.contains("--")
        && value
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

fn validate_timeline_dates(starts_at: &str, ends_at: &str) -> Result<(), AppError> {
    let starts_at = DateTime::parse_from_rfc3339(starts_at).map_err(|_| {
        AppError::BadRequest("start_at must be a valid RFC3339 datetime".to_string())
    })?;

    let ends_at = DateTime::parse_from_rfc3339(ends_at).map_err(|_| {
        AppError::BadRequest("ends_at must be a valid RFC3339 datetime".to_string())
    })?;

    if starts_at > ends_at {
        return Err(AppError::BadRequest(
            "ends_at must be the same as or later than starts_at".to_string(),
        ));
    }

    Ok(())
}

fn is_valid_timeline_item_type(value: &str) -> bool {
    matches!(value, "task" | "asset" | "milestone")
}

fn is_valid_timeline_status(value: &str) -> bool {
    matches!(value, "planned" | "in_progress" | "blocked" | "done")
}
