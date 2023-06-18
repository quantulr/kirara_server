//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "images")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,
    #[sea_orm(unique)]
    pub file_path: String,
    pub file_name: String,
    pub size: u64,
    pub width: u64,
    pub height: u64,
    pub upload_time: u64,
    #[serde(skip_serializing)]
    pub uid: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
