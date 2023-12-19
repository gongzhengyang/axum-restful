#![cfg_attr(nightly_error_messages, feature(rustc_attrs))]
//! axum A restful framework based on `axum` and `sea-orm`. Inspired by `django-rest-framework`.
//! The goal of the project is to build an enterprise-level production framework.
pub mod db;
pub mod error;
pub mod swagger;
pub mod test_helpers;
pub mod utils;
pub mod views;

pub use db::get_db_connection_pool;
pub use error::AppError;
pub use views::ModelViewExt;
