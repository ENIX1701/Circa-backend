use super::entity;
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    #[display("Admin")]
    Admin,
    #[display("Organizer")]
    Organizer,
    #[display("Staff")]
    Staff,
    #[display("Volunteer")]
    Volunteer,
}

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    #[display("Active")]
    Active,
    #[display("Inactive")]
    Inactive,
}

impl From<UserRole> for entity::Role {
    fn from(item: UserRole) -> Self {
        match item {
            UserRole::Admin => entity::Role::Admin,
            UserRole::Organizer => entity::Role::Organizer,
            UserRole::Staff => entity::Role::Staff,
            UserRole::Volunteer => entity::Role::Volunteer,
        }
    }
}

impl From<UserStatus> for entity::Status {
    fn from(item: UserStatus) -> Self {
        match item {
            UserStatus::Active => entity::Status::Active,
            UserStatus::Inactive => entity::Status::Inactive,
        }
    }
}

impl From<entity::Role> for UserRole {
    fn from(item: entity::Role) -> Self {
        match item {
            entity::Role::Admin => UserRole::Admin,
            entity::Role::Organizer => UserRole::Organizer,
            entity::Role::Staff => UserRole::Staff,
            entity::Role::Volunteer => UserRole::Volunteer,
        }
    }
}

impl From<entity::Status> for UserStatus {
    fn from(item: entity::Status) -> Self {
        match item {
            entity::Status::Active => UserStatus::Active,
            entity::Status::Inactive => UserStatus::Inactive,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub phone: String,
    pub role: UserRole,
    pub status: UserStatus,
}

impl From<entity::Model> for User {
    fn from(model: entity::Model) -> Self {
        Self {
            id: model.id,
            name: model.name,
            surname: model.surname,
            email: model.email,
            phone: model.phone,
            role: model.role.into(),
            status: model.status.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub surname: String,
    pub email: String,
    pub phone: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub surname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
}
