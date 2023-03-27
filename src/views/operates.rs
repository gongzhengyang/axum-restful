use std::str::FromStr;

use async_trait::async_trait;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PrimaryKeyTrait, TryFromU64,
};
use serde::Serialize;
use serde_json::Value;

use crate::{db, AppError};

#[async_trait]
pub trait ModelView<T>
    where
        T: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <T::Entity as EntityTrait>::Model: IntoActiveModel<T> + Serialize,
        for<'de> <T::Entity as EntityTrait>::Model: serde::de::Deserialize<'de>,
{
    async fn http_create(Json(data): Json<Value>) -> Result<StatusCode, AppError> {
        let active_model = T::from_json(data)?;
        let result = active_model.insert(Self::get_db_connection().await).await?;
        tracing::debug!("create model {result:?}");
        Ok(StatusCode::CREATED)
    }

    async fn http_update(Json(data): Json<Value>) -> Result<StatusCode, AppError> {
        let active_model = T::from_json(data)?;
        let result = active_model.update(Self::get_db_connection().await).await?;
        tracing::debug!("update {result:?}");
        Ok(StatusCode::OK)
    }

    async fn http_partial_update(
        Path(pk): Path<u64>,
        Json(data): Json<Value>,
    ) -> Result<StatusCode, AppError> {
        let model = <T::Entity as EntityTrait>::find_by_id(Self::exchange_primary_key(pk))
            .one(Self::get_db_connection().await)
            .await?;
        if model.is_none() {
            return Ok(StatusCode::NOT_FOUND);
        }
        let model = model.unwrap();
        let mut active_model = model.into_active_model();
        for (k, v) in data.as_object().unwrap() {
            if let Ok(column) = <T::Entity as EntityTrait>::Column::from_str(k) {
                active_model.set(column, v.as_str().unwrap().into());
            }
        }
        let result = active_model.update(Self::get_db_connection().await).await;
        tracing::debug!("patch with result: {result:?}");
        Ok(StatusCode::OK)
    }

    async fn http_list() -> Result<Json<Value>, AppError> {
        let results = <T::Entity as EntityTrait>::find()
            .all(Self::get_db_connection().await)
            .await?;
        Ok(Json(serde_json::json!(results)))
    }

    async fn http_retrieve(Path(pk): Path<u64>) -> Response {
        let pk = Self::exchange_primary_key(pk);
        let model = <T::Entity as EntityTrait>::find_by_id(pk)
            .one(Self::get_db_connection().await)
            .await;
        if let Ok(Some(value)) = model {
            return Json(serde_json::json!(value)).into_response();
        }
        StatusCode::NOT_FOUND.into_response()
    }

    async fn get_db_connection() -> &'static DatabaseConnection {
        db::get_db_connection_pool().await
    }

    fn exchange_primary_key(
        pk: u64,
    ) -> <<T::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType {
        <<T::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType::try_from_u64(pk)
            .unwrap()
    }

    fn get_http_routes() -> Router
        where
            Self: Send + 'static,
    {
        Router::new()
            .route(
                "/:pk",
                get(Self::http_retrieve)
                    .patch(Self::http_partial_update)
                    .put(Self::http_update),
            )
            .route("/", get(Self::http_list).post(Self::http_create))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::tests_cfg::*;
    use sea_orm::{DbBackend, MockDatabase};
    use sea_orm::ActiveValue::Set;

    async fn get_db() -> DatabaseConnection {
        let db = MockDatabase::new(DbBackend::Postgres)
            .into_connection();
        for i in 0..10 {
            let cake_active_model = cake::ActiveModel {
                id: Set(i),
                name: Set(format!("cake-{i}"))
            };
            cake_active_model.save(&db);
        }
        db
    }

    #[tokio::test]
    async fn http_create() {
        let results: Vec<cake::Model> = cake::Entity::find().all(&db).await.unwrap();
        println!("{:?}",results);
    }
}
