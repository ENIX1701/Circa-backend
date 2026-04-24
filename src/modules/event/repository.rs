use super::{
    entity::{self, Entity as EventEntity},
    membership_entity::{self, Entity as EventMembershipEntity},
    models::{CreateEventRequest, CreatePlannerItemRequest, UpdatePlannerItemRequest},
    planner_item_entity::{self, Entity as PlannerItemEntity},
};
use crate::error::AppError;
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
}
