use std::str::FromStr;

use async_trait::async_trait;
use axum::extract::Query;
use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    ModelTrait, PaginatorTrait, PrimaryKeyTrait, TryFromU64,
};
use serde::Serialize;
use serde_json::Value;

use crate::{db, AppError};

#[async_trait]
pub trait ModelView<T>
where
    T: ActiveModelTrait + ActiveModelBehavior + Send + 'static + Sync,
    <T::Entity as EntityTrait>::Model: IntoActiveModel<T> + Serialize + Sync,
    for<'de> <T::Entity as EntityTrait>::Model: serde::de::Deserialize<'de>,
{
    /// get default db connection with default config
    /// you can change this when impl ModelView
    async fn get_db_connection() -> &'static DatabaseConnection {
        db::get_db_connection_pool().await
    }

    /// POST a json body to /api and create a line in database
    /// return http 201 StatusCode::CREATED
    async fn http_create(Json(data): Json<Value>) -> Result<StatusCode, AppError> {
        let active_model = T::from_json(data)?;
        tracing::debug!("active model is {active_model:?}");
        let result = active_model.insert(Self::get_db_connection().await).await?;
        tracing::debug!("create model {result:?}");
        Ok(StatusCode::CREATED)
    }

    /// PUT a json body to /api/:id
    /// change a line in database
    /// return http 200 StatusCode::OK
    async fn http_update(Json(data): Json<Value>) -> Result<StatusCode, AppError> {
        let active_model = T::from_json(data)?;
        tracing::debug!("update active model {active_model:?}");
        let result = active_model.update(Self::get_db_connection().await).await?;
        tracing::debug!("update {result:?}");
        Ok(StatusCode::OK)
    }

    /// PATCH a json body to /api/:id
    /// return http 200 StatusCode::OK if matched, or return 404 if not matched a query
    async fn http_partial_update(Path(id): Path<u64>, Json(data): Json<Value>) -> Response {
        let model = <T::Entity as EntityTrait>::find_by_id(Self::exchange_primary_key(id))
            .one(Self::get_db_connection().await)
            .await
            .unwrap();
        if model.is_none() {
            return StatusCode::NOT_FOUND.into_response();
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
        StatusCode::OK.into_response()
    }

    /// GET list results with /api
    /// you can set page_size and page_num to page results with url like /api?page_size=10 or /api?page_size=10&page_num=1
    /// return results with StatusCode::OK
    // #[]
    async fn http_list(Query(query): Query<Value>) -> Result<Json<Value>, AppError> {
        let db = Self::get_db_connection().await;
        let page_size = Self::get_page_size(&query);
        let results = if !page_size.eq(&0) {
            let paginator = T::Entity::find().into_model().paginate(db, page_size);
            let page_num = match query.get("page_num") {
                Some(page_num) if page_num.is_u64() => page_num.as_u64().unwrap(),
                _ => 1,
            };
            paginator.fetch_page(page_num).await?
        } else {
            <T::Entity as EntityTrait>::find()
                .all(Self::get_db_connection().await)
                .await?
        };
        Ok(Json(serde_json::json!(results)))
    }

    fn get_page_size(query: &Value) -> u64 {
        let page_size = query.get("page_size");
        if let Some(page_size) = page_size {
            if let Some(page_size) = page_size.as_u64() {
                return page_size;
            }
        }
        0
    }

    /// GET a single query result with /api/:id
    /// return http 200 with result or 404 if query not matched
    async fn http_retrieve(Path(id): Path<u64>) -> Response {
        let pk = Self::exchange_primary_key(id);
        let model = <T::Entity as EntityTrait>::find_by_id(pk)
            .one(Self::get_db_connection().await)
            .await;
        if let Ok(Some(value)) = model {
            return Json(serde_json::json!(value)).into_response();
        }
        StatusCode::NOT_FOUND.into_response()
    }

    /// DELETE a instance with /api/:id
    /// return http 204 if success delete or http 404 if not matched or http 500 with error info
    async fn http_delete(Path(id): Path<u64>) -> Response {
        let pk = Self::exchange_primary_key(id);
        let db = Self::get_db_connection().await;
        let model = <T::Entity as EntityTrait>::find_by_id(pk).one(db).await;
        if let Ok(Some(value)) = model {
            let delete = value.delete(db).await;
            if delete.is_ok() {
                StatusCode::NO_CONTENT.into_response()
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", delete)).into_response()
            }
        } else {
            StatusCode::NOT_FOUND.into_response()
        }
    }

    /// change a value u64 into primary key
    fn exchange_primary_key(
        id: u64,
    ) -> <<T::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType {
        <<T::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType::try_from_u64(id)
            .unwrap()
    }

    /// get http routers with full operates
    fn get_http_routes() -> Router
    where
        Self: Send + 'static,
    {
        Router::new()
            .route(
                "/:id",
                get(Self::http_retrieve)
                    .patch(Self::http_partial_update)
                    .put(Self::http_update)
                    .delete(Self::http_delete),
            )
            .route("/", get(Self::http_list).post(Self::http_create))
    }
}
