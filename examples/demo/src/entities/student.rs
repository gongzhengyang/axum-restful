//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.0

use sea_orm::entity::prelude::*;
use sea_orm::strum::IntoEnumIterator;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "student")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub name: String,
    pub region: String,
    pub age: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// #[derive(Copy, Clone, Debug, EnumIter)]
// pub enum PrimaryKey {
//     Id,
// }
//
// impl PrimaryKeyTrait for PrimaryKey {
//     type ValueType = i64;
//
//     fn auto_increment() -> bool {
//         true
//     }
// }
