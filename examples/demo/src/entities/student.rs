//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use schemars::JsonSchema;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, JsonSchema, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "student")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub region: String,
    pub age: i16,
    pub create_time: DateTime,
    #[sea_orm(column_type = "Double")]
    pub score: f64,
    pub gender: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
