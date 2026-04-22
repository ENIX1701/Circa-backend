use super::{entity, membership_entity};
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    #[display("Draft")]
    Draft,
    #[display("Active")]
    Active,
    #[display("Closed")]
    Closed,
    #[display("Archived")]
    Archived,
    #[display("Pending destruction")]
    PendingDestruction,
}

impl From<EventStatus> for entity::Status {
    fn from(item: EventStatus) -> Self {
        match item {
            EventStatus::Draft => entity::Status::Draft,
            EventStatus::Active => entity::Status::Active,
            EventStatus::Closed => entity::Status::Closed,
            EventStatus::Archived => entity::Status::Archived,
            EventStatus::PendingDestruction => entity::Status::PendingDestruction,
        }
    }
}

impl From<entity::Status> for EventStatus {
    fn from(item: entity::Status) -> Self {
        match item {
            entity::Status::Draft => EventStatus::Draft,
            entity::Status::Active => EventStatus::Active,
            entity::Status::Closed => EventStatus::Closed,
            entity::Status::Archived => EventStatus::Archived,
            entity::Status::PendingDestruction => EventStatus::PendingDestruction,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventMembershipRole {
    #[display("Owner")]
    Owner,
    #[display("Organizer")]
    Organizer,
    #[display("Staff")]
    Staff,
    #[display("Volunteer")]
    Volunteer,
}

impl From<EventMembershipRole> for membership_entity::Role {
    fn from(item: EventMembershipRole) -> Self {
        match item {
            EventMembershipRole::Owner => membership_entity::Role::Owner,
            EventMembershipRole::Organizer => membership_entity::Role::Organizer,
            EventMembershipRole::Staff => membership_entity::Role::Staff,
            EventMembershipRole::Volunteer => membership_entity::Role::Volunteer,
        }
    }
}

impl From<membership_entity::Role> for EventMembershipRole {
    fn from(item: membership_entity::Role) -> Self {
        match item {
            membership_entity::Role::Owner => EventMembershipRole::Owner,
            membership_entity::Role::Organizer => EventMembershipRole::Organizer,
            membership_entity::Role::Staff => EventMembershipRole::Staff,
            membership_entity::Role::Volunteer => EventMembershipRole::Volunteer,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub venue: String,
    pub timezone: String,
    pub starts_at: String,
    pub ends_at: String,
    pub status: EventStatus,
    pub created_by_user_id: String,
    pub current_user_role: EventMembershipRole,
    pub destruction_requested_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Event {
    pub fn from_parts(model: entity::Model, current_user_role: membership_entity::Role) -> Self {
        Self {
            id: model.id,
            name: model.name,
            slug: model.slug,
            description: model.description,
            venue: model.venue,
            timezone: model.timezone,
            starts_at: model.starts_at,
            ends_at: model.ends_at,
            status: model.status.into(),
            created_by_user_id: model.created_by_user_id,
            current_user_role: current_user_role.into(),
            destruction_requested_at: model.destruction_requested_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEventRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub venue: String,
    pub timezone: String,
    pub starts_at: String,
    pub ends_at: String,
}
