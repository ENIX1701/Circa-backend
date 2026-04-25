use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "planner_timeline_items")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
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
    pub position: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
