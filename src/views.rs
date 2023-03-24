use std::str::FromStr;

use async_trait::async_trait;
use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
    Router, routing::get,
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    PrimaryKeyTrait, TryFromU64,
};
use serde::Serialize;
use serde_json::Value;

use crate::db;

#[async_trait]
pub trait ModelView<T>
where
    T: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
    <T::Entity as EntityTrait>::Model: IntoActiveModel<T> + Serialize,
    for<'de> <T::Entity as EntityTrait>::Model: serde::de::Deserialize<'de>,
{
    async fn http_create(Json(data): Json<Value>) -> Response {
        let active_model = T::from_json(data);
        match active_model {
            Ok(active_model) => {
                let result = active_model.insert(Self::get_db_connection().await).await;
                match result {
                    Ok(_) => StatusCode::CREATED.into_response(),
                    Err(e) => format!("{e:?}").into_response(),
                }
            }
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{e:?}")).into_response(),
        }
    }

    async fn http_update(Json(data): Json<Value>) -> Response {
        let active_model = T::from_json(data).unwrap();
        let result = active_model.update(Self::get_db_connection().await).await;
        println!("update {result:?}");
        Self::ok_response()
    }

    async fn http_partial_update(Path(pk): Path<u64>, Json(data): Json<Value>) -> Response {
        let model = <T::Entity as EntityTrait>::find_by_id(Self::exchange_primary_key(pk))
            .one(Self::get_db_connection().await)
            .await;
        match model {
            Ok(model) => match model {
                Some(model) => {
                    println!("{model:?}");
                    let mut active_model = model.into_active_model();
                    for (k, v) in data.as_object().unwrap() {
                        if let Ok(column) = <T::Entity as EntityTrait>::Column::from_str(k) {
                            active_model.set(column, v.as_str().unwrap().into());
                        }
                    }
                    println!("changed active model to {active_model:?}");
                    let result = active_model.update(Self::get_db_connection().await).await;
                    println!("patch with {result:?}");
                    Self::ok_response()
                }
                None => Self::not_found(),
            },
            Err(err) => Self::internal_erorr(err),
        }
    }

    async fn http_list() -> Response {
        let results = <T::Entity as EntityTrait>::find()
            .all(Self::get_db_connection().await)
            .await
            .unwrap();
        Json(serde_json::json!(results)).into_response()
    }

    async fn http_retrieve(Path(pk): Path<u64>) -> Response {
        let pk = Self::exchange_primary_key(pk);
        let model = <T::Entity as EntityTrait>::find_by_id(pk)
            .one(Self::get_db_connection().await)
            .await;
        if let Ok(Some(value)) = model {
            return Json(serde_json::json!(value)).into_response();
        }
        Self::not_found()
    }

    fn not_found() -> Response {
        StatusCode::NOT_FOUND.into_response()
    }

    fn internal_erorr(error: impl std::error::Error) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{error:?}")).into_response()
    }

    fn ok_response() -> Response {
        StatusCode::OK.into_response()
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
