use super::{
    entity::{self, Entity as EventEntity},
    event_branding_entity::{self, Entity as EventBrandingEntity},
    membership_entity::{self, Entity as EventMembershipEntity},
    models::{
        CreateEventRequest, CreatePlannerItemRequest, CreatePlannerTimelineItemRequest,
        CreateSocialMediaPostRequest, UpdatePlannerItemRequest, UpdatePlannerTimelineItemRequest,
        UpdateSocialMediaPostRequest, UpsertEventBrandingRequest,
    },
    planner_item_entity::{self, Entity as PlannerItemEntity},
    planner_timeline_item_entity::{self, Entity as PlannerTimelineItemEntity},
    social_post_entity::{self, Entity as SocialPostEntity},
};
use crate::{
    error::AppError,
    user::entity::{self as user_entity, Entity as UserEntity},
};
use chrono::Utc;
use sea_orm::*;
use std::collections::HashMap;

pub struct EventRepository {
    db: DatabaseConnection,
}

impl EventRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn list_for_user(
        &self,
        user_id: &str,
    ) -> Result<Vec<(entity::Model, membership_entity::Role)>, AppError> {
        let memberships = EventMembershipEntity::find()
            .filter(membership_entity::Column::UserId.eq(user_id))
            .all(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if memberships.is_empty() {
            return Ok(vec![]);
        }

        let event_ids: Vec<String> = memberships.iter().map(|m| m.event_id.clone()).collect();
        let role_by_event_id: HashMap<String, membership_entity::Role> = memberships
            .into_iter()
            .map(|m| (m.event_id, m.role))
            .collect();

        let events = EventEntity::find()
            .filter(entity::Column::Id.is_in(event_ids))
            .order_by_desc(entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(events
            .into_iter()
            .filter_map(|event| {
                role_by_event_id
                    .get(&event.id)
                    .cloned()
                    .map(|role| (event, role))
            })
            .collect())
    }

    pub async fn find_membership(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Option<membership_entity::Model>, AppError> {
        EventMembershipEntity::find()
            .filter(membership_entity::Column::EventId.eq(event_id))
            .filter(membership_entity::Column::UserId.eq(user_id))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn list_collaborators(
        &self,
        event_id: &str,
    ) -> Result<Vec<(membership_entity::Model, user_entity::Model)>, AppError> {
        let memberships = EventMembershipEntity::find()
            .filter(membership_entity::Column::EventId.eq(event_id))
            .order_by_asc(membership_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let user_ids: Vec<String> = memberships.iter().map(|m| m.user_id.clone()).collect();

        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        let users = UserEntity::find()
            .filter(user_entity::Column::Id.is_in(user_ids))
            .all(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let users_by_id: HashMap<String, user_entity::Model> = users
            .into_iter()
            .map(|user| (user.id.clone(), user))
            .collect();

        Ok(memberships
            .into_iter()
            .filter_map(|membership| {
                users_by_id
                    .get(&membership.user_id)
                    .cloned()
                    .map(|user| (membership, user))
            })
            .collect())
    }

    pub async fn find_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<user_entity::Model>, AppError> {
        UserEntity::find()
            .filter(user_entity::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn find_user_by_id(
        &self,
        user_id: &str,
    ) -> Result<Option<user_entity::Model>, AppError> {
        UserEntity::find_by_id(user_id.to_string())
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn add_membership(
        &self,
        event_id: &str,
        user_id: &str,
        role: membership_entity::Role,
    ) -> Result<membership_entity::Model, AppError> {
        let membership = membership_entity::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            event_id: Set(event_id.to_string()),
            user_id: Set(user_id.to_string()),
            role: Set(role),
            created_at: Set(Utc::now().to_rfc3339()),
        };

        membership
            .insert(&self.db)
            .await
            .map_err(|_| AppError::BadRequest("User is already a collaborator! :o".to_string()))
    }

    pub async fn update_membership_role(
        &self,
        event_id: &str,
        user_id: &str,
        role: membership_entity::Role,
    ) -> Result<membership_entity::Model, AppError> {
        let membership = self
            .find_membership(event_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Collaborator not found :c".to_string()))?;

        let mut active_model: membership_entity::ActiveModel = membership.into();
        active_model.role = Set(role);

        active_model
            .update(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn delete_membership(&self, event_id: &str, user_id: &str) -> Result<(), AppError> {
        let membership = self
            .find_membership(event_id, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Collaborator not found :c".to_string()))?;

        membership
            .delete(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    pub async fn owner_count(&self, event_id: &str) -> Result<u64, AppError> {
        EventMembershipEntity::find()
            .filter(membership_entity::Column::EventId.eq(event_id))
            .filter(membership_entity::Column::Role.eq(membership_entity::Role::Owner))
            .count(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn find_for_user(
        &self,
        event_id: &str,
        user_id: &str,
    ) -> Result<Option<(entity::Model, membership_entity::Role)>, AppError> {
        let membership = self.find_membership(event_id, user_id).await?;

        let Some(membership) = membership else {
            return Ok(None);
        };

        let event = EventEntity::find_by_id(event_id.to_string())
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(event.map(|event| (event, membership.role)))
    }

    pub async fn slug_exists(&self, slug: &str) -> Result<bool, AppError> {
        let existing = EventEntity::find()
            .filter(entity::Column::Slug.eq(slug))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(existing.is_some())
    }

    pub async fn create(
        &self,
        dto: CreateEventRequest,
        creator_user_id: &str,
    ) -> Result<(entity::Model, membership_entity::Role), AppError> {
        let now = Utc::now().to_rfc3339();
        let event_id = uuid::Uuid::now_v7().to_string();

        let event = entity::ActiveModel {
            id: Set(event_id.clone()),
            name: Set(dto.name),
            slug: Set(dto.slug),
            description: Set(dto.description.unwrap_or_default()),
            venue: Set(dto.venue),
            timezone: Set(dto.timezone),
            starts_at: Set(dto.starts_at),
            ends_at: Set(dto.ends_at),
            status: Set(entity::Status::Draft),
            created_by_user_id: Set(creator_user_id.to_string()),
            destruction_requested_at: Set(None),
            created_at: Set(now.clone()),
            updated_at: Set(now.clone()),
        };

        EventEntity::insert(event)
            .exec(&self.db)
            .await
            .map_err(|_| AppError::BadRequest("Event slug already exists".to_string()))?;

        let membership = membership_entity::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            event_id: Set(event_id.clone()),
            user_id: Set(creator_user_id.to_string()),
            role: Set(membership_entity::Role::Owner),
            created_at: Set(now),
        };

        EventMembershipEntity::insert(membership)
            .exec(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        self.find_for_user(&event_id, creator_user_id)
            .await?
            .ok_or(AppError::InternalServerError)
    }

    pub async fn update_status(
        &self,
        event_id: &str,
        status: entity::Status,
        destruction_requested_at: Option<String>,
    ) -> Result<entity::Model, AppError> {
        let event = EventEntity::find_by_id(event_id.to_string())
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Event not found".to_string()))?;

        let mut active_model: entity::ActiveModel = event.into();
        active_model.status = Set(status);
        active_model.destruction_requested_at = Set(destruction_requested_at);
        active_model.updated_at = Set(Utc::now().to_rfc3339());

        active_model
            .update(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn find_event_branding(
        &self,
        event_id: &str,
    ) -> Result<Option<event_branding_entity::Model>, AppError> {
        EventBrandingEntity::find()
            .filter(event_branding_entity::Column::EventId.eq(event_id))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn upsert_event_branding(
        &self,
        event_id: &str,
        dto: UpsertEventBrandingRequest,
    ) -> Result<event_branding_entity::Model, AppError> {
        let now = Utc::now().to_rfc3339();

        let theme_mode = if dto.theme_mode.trim().is_empty() {
            "dark".to_string()
        } else {
            dto.theme_mode
        };

        let background_color = dto.background_color;

        if let Some(existing) = self.find_event_branding(event_id).await? {
            let mut active_model: event_branding_entity::ActiveModel = existing.into();
            active_model.event_name_override = Set(dto.event_name_override);
            active_model.tagline = Set(dto.tagline);
            active_model.primary_color = Set(dto.primary_color);
            active_model.secondary_color = Set(dto.secondary_color);
            active_model.theme_mode = Set(theme_mode);
            active_model.background_color = Set(background_color);
            active_model.notes = Set(dto.notes);
            active_model.updated_at = Set(now);

            active_model
                .update(&self.db)
                .await
                .map_err(|_| AppError::InternalServerError)
        } else {
            let branding = event_branding_entity::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                event_id: Set(event_id.to_string()),
                event_name_override: Set(dto.event_name_override),
                tagline: Set(dto.tagline),
                primary_color: Set(dto.primary_color),
                secondary_color: Set(dto.secondary_color),
                theme_mode: Set(theme_mode),
                background_color: Set(background_color),
                notes: Set(dto.notes),
                created_at: Set(now.clone()),
                updated_at: Set(now),
            };

            branding
                .insert(&self.db)
                .await
                .map_err(|_| AppError::InternalServerError)
        }
    }

    pub async fn list_planner_items(
        &self,
        event_id: &str,
    ) -> Result<Vec<planner_item_entity::Model>, AppError> {
        PlannerItemEntity::find()
            .filter(planner_item_entity::Column::EventId.eq(event_id))
            .order_by_asc(planner_item_entity::Column::Position)
            .order_by_asc(planner_item_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn create_planner_item(
        &self,
        event_id: &str,
        dto: CreatePlannerItemRequest,
    ) -> Result<planner_item_entity::Model, AppError> {
        let now = Utc::now().to_rfc3339();

        let next_position = PlannerItemEntity::find()
            .filter(planner_item_entity::Column::EventId.eq(event_id))
            .order_by_desc(planner_item_entity::Column::Position)
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .map(|item| item.position + 1)
            .unwrap_or(0);

        let planner_item = planner_item_entity::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            event_id: Set(event_id.to_string()),
            title: Set(dto.title),
            notes: Set(dto.notes.unwrap_or_default()),
            position: Set(next_position),
            done: Set(false),
            created_at: Set(now.clone()),
            updated_at: Set(now),
        };

        planner_item
            .insert(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn update_planner_item(
        &self,
        event_id: &str,
        item_id: &str,
        dto: UpdatePlannerItemRequest,
    ) -> Result<planner_item_entity::Model, AppError> {
        let planner_item = PlannerItemEntity::find()
            .filter(planner_item_entity::Column::EventId.eq(event_id))
            .filter(planner_item_entity::Column::Id.eq(item_id))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Planner item not found".to_string()))?;

        let mut active_model: planner_item_entity::ActiveModel = planner_item.into();

        if let Some(title) = dto.title {
            active_model.title = Set(title);
        }

        if let Some(notes) = dto.notes {
            active_model.notes = Set(notes);
        }

        if let Some(position) = dto.position {
            active_model.position = Set(position);
        }

        if let Some(done) = dto.done {
            active_model.done = Set(done);
        }

        active_model.updated_at = Set(Utc::now().to_rfc3339());

        active_model
            .update(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn delete_planner_item(&self, event_id: &str, item_id: &str) -> Result<(), AppError> {
        let planner_item = PlannerItemEntity::find()
            .filter(planner_item_entity::Column::EventId.eq(event_id))
            .filter(planner_item_entity::Column::Id.eq(item_id))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Planner item not found".to_string()))?;

        planner_item
            .delete(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    pub async fn list_planner_timeline_items(
        &self,
        event_id: &str,
    ) -> Result<Vec<planner_timeline_item_entity::Model>, AppError> {
        PlannerTimelineItemEntity::find()
            .filter(planner_timeline_item_entity::Column::EventId.eq(event_id))
            .order_by_asc(planner_timeline_item_entity::Column::Position)
            .order_by_asc(planner_timeline_item_entity::Column::StartsAt)
            .all(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn create_planner_timeline_item(
        &self,
        event_id: &str,
        dto: CreatePlannerTimelineItemRequest,
    ) -> Result<planner_timeline_item_entity::Model, AppError> {
        let now = Utc::now().to_rfc3339();

        let next_position = PlannerTimelineItemEntity::find()
            .filter(planner_timeline_item_entity::Column::EventId.eq(event_id))
            .order_by_desc(planner_timeline_item_entity::Column::Position)
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .map(|post| post.position + 1)
            .unwrap_or(0);

        let timeline_item = planner_timeline_item_entity::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            event_id: Set(event_id.to_string()),
            title: Set(dto.title),
            item_type: Set(dto.item_type),
            starts_at: Set(dto.starts_at),
            ends_at: Set(dto.ends_at),
            status: Set(dto.status.unwrap_or_else(|| "planned".to_string())),
            owner: Set(dto.owner.unwrap_or_default()),
            notes: Set(dto.notes.unwrap_or_default()),
            color: Set(dto.color.unwrap_or_default()),
            position: Set(next_position),
            created_at: Set(now.clone()),
            updated_at: Set(now),
            depends_on_item_id: Set(dto.depends_on_item_id.unwrap_or_default()),
            assigned_user_id: Set(dto.assigned_user_id.unwrap_or_default()),
        };

        timeline_item
            .insert(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn update_planner_timeline_item(
        &self,
        event_id: &str,
        item_id: &str,
        dto: UpdatePlannerTimelineItemRequest,
    ) -> Result<planner_timeline_item_entity::Model, AppError> {
        let timeline_item = PlannerTimelineItemEntity::find()
            .filter(planner_timeline_item_entity::Column::EventId.eq(event_id))
            .filter(planner_timeline_item_entity::Column::Id.eq(item_id))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Planner timeline item not found".to_string()))?;

        let mut active_model: planner_timeline_item_entity::ActiveModel = timeline_item.into();

        if let Some(title) = dto.title {
            active_model.title = Set(title);
        }
        if let Some(item_type) = dto.item_type {
            active_model.item_type = Set(item_type);
        }
        if let Some(starts_at) = dto.starts_at {
            active_model.starts_at = Set(starts_at);
        }
        if let Some(ends_at) = dto.ends_at {
            active_model.ends_at = Set(ends_at);
        }
        if let Some(status) = dto.status {
            active_model.status = Set(status);
        }
        if let Some(owner) = dto.owner {
            active_model.owner = Set(owner);
        }
        if let Some(notes) = dto.notes {
            active_model.notes = Set(notes);
        }
        if let Some(color) = dto.color {
            active_model.color = Set(color);
        }
        if let Some(position) = dto.position {
            active_model.position = Set(position);
        }
        if let Some(depends_on_item_id) = dto.depends_on_item_id {
            active_model.depends_on_item_id = Set(depends_on_item_id);
        }
        if let Some(assigned_user_id) = dto.assigned_user_id {
            active_model.assigned_user_id = Set(assigned_user_id);
        }

        active_model.updated_at = Set(Utc::now().to_rfc3339());

        active_model
            .update(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn delete_planner_timeline_item(
        &self,
        event_id: &str,
        item_id: &str,
    ) -> Result<(), AppError> {
        let timeline_item = PlannerTimelineItemEntity::find()
            .filter(planner_timeline_item_entity::Column::EventId.eq(event_id))
            .filter(planner_timeline_item_entity::Column::Id.eq(item_id))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Planner timeline item not found".to_string()))?;

        timeline_item
            .delete(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    pub async fn list_social_posts(
        &self,
        event_id: &str,
    ) -> Result<Vec<social_post_entity::Model>, AppError> {
        SocialPostEntity::find()
            .filter(social_post_entity::Column::EventId.eq(event_id))
            .order_by_asc(social_post_entity::Column::Position)
            .order_by_asc(social_post_entity::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn create_social_post(
        &self,
        event_id: &str,
        dto: CreateSocialMediaPostRequest,
    ) -> Result<social_post_entity::Model, AppError> {
        let now = Utc::now().to_rfc3339();

        let next_position = SocialPostEntity::find()
            .filter(social_post_entity::Column::EventId.eq(event_id))
            .order_by_desc(social_post_entity::Column::Position)
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .map(|post| post.position + 1)
            .unwrap_or(0);

        let social_post = social_post_entity::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            event_id: Set(event_id.to_string()),
            platform: Set(dto.platform),
            title: Set(dto.title),
            body: Set(dto.body.unwrap_or_default()),
            status: Set("draft".to_string()),
            position: Set(next_position),
            created_at: Set(now.clone()),
            updated_at: Set(now),
        };

        social_post
            .insert(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn update_social_post(
        &self,
        event_id: &str,
        post_id: &str,
        dto: UpdateSocialMediaPostRequest,
    ) -> Result<social_post_entity::Model, AppError> {
        let social_post = SocialPostEntity::find()
            .filter(social_post_entity::Column::EventId.eq(event_id))
            .filter(social_post_entity::Column::Id.eq(post_id))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Social post not found".to_string()))?;

        let mut active_model: social_post_entity::ActiveModel = social_post.into();

        if let Some(platform) = dto.platform {
            active_model.platform = Set(platform);
        }

        if let Some(title) = dto.title {
            active_model.title = Set(title);
        }

        if let Some(body) = dto.body {
            active_model.body = Set(body);
        }

        if let Some(status) = dto.status {
            active_model.status = Set(status);
        }

        if let Some(position) = dto.position {
            active_model.position = Set(position);
        }

        active_model.updated_at = Set(Utc::now().to_rfc3339());

        active_model
            .update(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)
    }

    pub async fn delete_social_post(&self, event_id: &str, post_id: &str) -> Result<(), AppError> {
        let social_post = SocialPostEntity::find()
            .filter(social_post_entity::Column::EventId.eq(event_id))
            .filter(social_post_entity::Column::Id.eq(post_id))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or_else(|| AppError::NotFound("Social post not found".to_string()))?;

        social_post
            .delete(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }
}
