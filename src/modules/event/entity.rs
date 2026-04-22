use derive_more::Display;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Status {
    #[sea_orm(string_value = "draft")]
    Draft,
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "closed")]
    Closed,
    #[sea_orm(string_value = "archived")]
    Archived,
    #[sea_orm(string_value = "pending_destruction")]
    PendingDestruction,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "events")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub venue: String,
    pub timezone: String,
    pub starts_at: String,
    pub ends_at: String,
    pub status: Status,
    pub created_by_user_id: String,
    pub destruction_requested_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
