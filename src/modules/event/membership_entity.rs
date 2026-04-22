use derive_more::Display;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Role {
    #[sea_orm(string_value = "owner")]
    Owner,
    #[sea_orm(string_value = "organizer")]
    Organizer,
    #[sea_orm(string_value = "staff")]
    Staff,
    #[sea_orm(string_value = "volunteer")]
    Volunteer,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "event_memberships")]
pub struct Model {
    #[sea_orm(primary_key, auto_increament = false)]
    pub id: String,
    pub event_id: String,
    pub user_id: String,
    pub role: Role,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
