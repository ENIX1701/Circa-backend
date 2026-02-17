use derive_more::Display;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::StringLen;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Role {
    #[sea_orm(string_value = "event_director")]
    EventDirector,
    #[sea_orm(string_value = "booth_owner")]
    BoothOwner,
    #[sea_orm(string_value = "clown")]
    Clown,
}

#[derive(Debug, Serialize, Deserialize, Display, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum Status {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub surname: String,
    pub email: String,
    pub phone: String,
    pub role: Role,
    pub status: Status,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
