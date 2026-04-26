use super::{
    entity, event_branding_entity, membership_entity, planner_item_entity, social_post_entity, planner_timeline_item_entity,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use crate::user::entity as user_entity;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventCollaborator {
    pub user_id: String,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub phone: String,
    pub role: EventMembershipRole,
    pub created_at: String,
}

impl EventCollaborator {
    pub fn from_parts(membership: membership_entity::Model, user: user_entity::Model) -> Self {
        Self {
            user_id: user.id,
            name: user.name,
            surname: user.surname,
            email: user.email,
            phone: user.phone,
            role: membership.role.into(),
            created_at: membership.created_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AddEventCollaboratorRequest {
    pub email: String,
    pub role: EventMembershipRole,
}


#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateEventCollaboratorRequest {
    pub role: EventMembershipRole,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlannerItem {
    pub id: String,
    pub event_id: String,
    pub title: String,
    pub notes: String,
    pub position: i32,
    pub done: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<planner_item_entity::Model> for PlannerItem {
    fn from(model: planner_item_entity::Model) -> Self {
        Self {
            id: model.id,
            event_id: model.event_id,
            title: model.title,
            notes: model.notes,
            position: model.position,
            done: model.done,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreatePlannerItemRequest {
    pub title: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdatePlannerItemRequest {
    pub title: Option<String>,
    pub notes: Option<String>,
    pub position: Option<i32>,
    pub done: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PlannerTimelineItem {
    pub id: String,
    pub event_id: String,
    pub title: String,
    pub item_type: String,
    pub starts_at: String,
    pub ends_at: String,
    pub status: String,
    pub owner: String,
    pub notes: String,
    pub color: String,
    pub depends_on_item_id: String,
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<planner_timeline_item_entity::Model> for PlannerTimelineItem {
    fn from(model: planner_timeline_item_entity::Model) -> Self {
        Self {
            id: model.id,
            event_id: model.event_id,
            title: model.title,
            item_type: model.item_type,
            starts_at: model.starts_at,
            ends_at: model.ends_at,
            status: model.status,
            owner: model.owner,
            notes: model.notes,
            color: model.color,
            depends_on_item_id: model.depends_on_item_id,
            position: model.position,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreatePlannerTimelineItemRequest {
    pub title: String,
    pub item_type: String,
    pub starts_at: String,
    pub ends_at: String,
    pub status: Option<String>,
    pub owner: Option<String>,
    pub notes: Option<String>,
    pub color: Option<String>,
    pub depends_on_item_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UpdatePlannerTimelineItemRequest {
    pub title: Option<String>,
    pub item_type: Option<String>,
    pub starts_at: Option<String>,
    pub ends_at: Option<String>,
    pub status: Option<String>,
    pub owner: Option<String>,
    pub notes: Option<String>,
    pub color: Option<String>,
    pub depends_on_item_id: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventBranding {
    pub id: Option<String>,
    pub event_id: String,
    pub event_name_override: String,
    pub tagline: String,
    pub primary_color: String, // TODO: how to store this more efficiently?
    pub secondary_color: String,
    pub theme_mode: String,
    pub background_color: String,
    pub notes: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl EventBranding {
    pub fn from_model(model: event_branding_entity::Model) -> Self {
        Self {
            id: Some(model.id),
            event_id: model.event_id,
            event_name_override: model.event_name_override,
            tagline: model.tagline,
            primary_color: model.primary_color,
            secondary_color: model.secondary_color,
            theme_mode: model.theme_mode,
            background_color: model.background_color,
            notes: model.notes,
            created_at: Some(model.created_at),
            updated_at: Some(model.updated_at),
        }
    }

    pub fn default_for_event(event_id: &str) -> Self {
        Self {
            id: None,
            event_id: event_id.to_string(),
            event_name_override: String::new(),
            tagline: String::new(),
            primary_color: String::new(),
            secondary_color: String::new(),
            theme_mode: "dark".to_string(),
            background_color: String::new(),
            notes: String::new(),
            created_at: None,
            updated_at: None,
        }
    }
}

// this uses a cool new word I've learned
// upsert
// it basically means update + insert
// a single endpoint for both :D
#[derive(Debug, Deserialize, Serialize)]
pub struct UpsertEventBrandingRequest {
    #[serde(default)]
    pub event_name_override: String,
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub primary_color: String,
    #[serde(default)]
    pub secondary_color: String,
    #[serde(default)]
    pub theme_mode: String,
    #[serde(default)]
    pub background_color: String,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SocialPost {
    pub id: String,
    pub event_id: String,
    pub platform: String,
    pub title: String,
    pub body: String,
    pub status: String,
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<social_post_entity::Model> for SocialPost {
    fn from(model: social_post_entity::Model) -> Self {
        Self {
            id: model.id,
            event_id: model.event_id,
            platform: model.platform,
            title: model.title,
            body: model.body,
            status: model.status,
            position: model.position,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateSocialMediaPostRequest {
    pub platform: String,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateSocialMediaPostRequest {
    pub platform: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventExport {
    pub exported_at: String,
    pub event: Event,
    pub branding: EventBranding,
    pub planner_items: Vec<PlannerItem>,
    pub social_posts: Vec<SocialPost>,
    pub planner_timeline_items: Vec<PlannerTimelineItem>,
}
