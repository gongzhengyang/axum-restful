use std::fmt::Debug;
use std::str::FromStr;

use async_trait::async_trait;
use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    response::{IntoResponse, Response},
    Router, routing::get,
};
use axum::extract::Query;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, DatabaseConnection, EntityTrait, IntoActiveModel, ModelTrait, PaginatorTrait, PrimaryKeyTrait, TryFromU64};
use serde::Serialize;
use serde_json::Value;

use crate::{AppError, db};

#[async_trait]
pub trait ModelView<T>
    where
        T: ActiveModelTrait + ActiveModelBehavior + Send + 'static,
        <T::Entity as EntityTrait>::Model: IntoActiveModel<T> + Serialize,
        for<'de> <T::Entity as EntityTrait>::Model: serde::de::Deserialize<'de>,
{
    async fn get_db_connection() -> &'static DatabaseConnection {
        db::get_db_connection_pool().await
    }

    /// POST a json body to /api and create a line in database
    /// return http 201 StatusCode::CREATED
    async fn http_create(Json(data): Json<Value>) -> Result<StatusCode, AppError> {
        let active_model = T::from_json(data)?;
        let result = active_model.insert(Self::get_db_connection().await).await?;
        tracing::debug!("create model {result:?}");
        Ok(StatusCode::CREATED)
    }

    /// PUT a json body to /api/:id
    /// change a line in database
    /// return http 200 StatusCode::OK
    async fn http_update(Json(data): Json<Value>) -> Result<StatusCode, AppError> {
        let active_model = T::from_json(data)?;
        let result = active_model.update(Self::get_db_connection().await).await?;
        tracing::debug!("update {result:?}");
        Ok(StatusCode::OK)
    }

    /// PATCH a json body to /api/:id
    /// return http 200 StatusCode::OK if matched, or return 404 if not matched a query
    async fn http_partial_update(
        Path(id): Path<u64>,
        Json(data): Json<Value>,
    ) -> Result<StatusCode, AppError> {
        let model = <T::Entity as EntityTrait>::find_by_id(Self::exchange_primary_key(id))
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

    /// GET list results with /api
    /// you can set page_size and page_num to page results with url like /api?page_size=10 or /api?page_size=10&page_num=1
    /// return results with StatusCode::OK
    async fn http_list(Query(query): Query<Value>) -> Result<Json<Value>, AppError> {
        let query = query.as_object().unwrap();
        let selector = <T::Entity as EntityTrait>::find();
        let db = Self::get_db_connection().await;
        let page_size = query.get("page_size");
        let results = if page_size.is_some() {
            let paginator = selector.paginate(db, page_size.unwrap().into());
            paginator.fetch_page(query.get("page_num").unwrap_or(&serde_json::json!(1)).into()).await?
        } else {
            <T::Entity as EntityTrait>::find()
                .all(Self::get_db_connection().await)
                .await?
        };
        Ok(Json(serde_json::json!(results)))
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
        let model = <T::Entity as EntityTrait>::find_by_id(pk)
            .one(db)
            .await;
        if let Ok(Some(value)) = model {
            let delete = value.delete(db).await;
            if delete.is_ok() {
                StatusCode::NO_CONTENT.into_response()
            }
            else {
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
