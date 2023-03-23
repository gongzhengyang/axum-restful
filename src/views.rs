use async_trait::async_trait;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
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

    async fn http_update() -> Result<StatusCode, Response> {
        Ok(StatusCode::OK)
    }

    async fn http_partial_update() -> Result<StatusCode, Response> {
        Ok(StatusCode::OK)
    }

    async fn http_list() -> Response {
        let results = <T::Entity as EntityTrait>::find()
            .all(Self::get_db_connection().await)
            .await
            .unwrap();
        Json(serde_json::json!(results)).into_response()
    }

    // async fn http_retrieve(Path(pk): Path<<<T::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>) -> Json<Value>
    // {
    //     let model = <T::Entity as EntityTrait>::find_by_id(pk).one(Self::get_db_connection().await).await;
    //     if let Ok(value) = model {
    //         if let Some(v) = value {
    //             return Json(serde_json::json!(v));
    //         }
    //     }
    //     Json(serde_json::json!(Null))
    // }

    async fn get_db_connection() -> &'static DatabaseConnection {
        db::get_db_connection_pool().await
    }

    fn get_http_routes() -> Router
    where
        Self: Send + 'static,
    {
        Router::new()
            .route(
                "/:id",
                // get(Self::http_retrieve).
                get(Self::http_partial_update).put(Self::http_update),
            )
            .route("/", get(Self::http_list).post(Self::http_create))
    }
}
