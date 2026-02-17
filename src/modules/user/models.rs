use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    #[display("Event director")]
    EventDirector,
    #[display("Booth owner")]
    BoothOwner,
    #[display("Clown")]
    Clown,
}

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    #[display("Active")]
    Active,
    #[display("Inactive")]
    Inactive,
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

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub surname: String,
    pub email: String,
    pub phone: String,
    pub role: UserRole,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub surname: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role: Option<UserRole>,
    pub status: Option<UserStatus>,
}
