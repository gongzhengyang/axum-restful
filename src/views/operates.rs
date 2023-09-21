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
    Iterable, ModelTrait, PaginatorTrait, PrimaryKeyToColumn, PrimaryKeyTrait, TryFromU64,
};
use serde::Serialize;
use serde_json::Value;
use snafu::{OptionExt, ResultExt};

use crate::error::{OperateDatabaseSnafu, PrimaryKeyNotFoundSnafu};
use crate::{db, error::Result};

#[macro_export]
macro_rules! checked_response {
    ($value:expr, $status:expr) => {
        match $value {
            Ok(_) => $status.into_response(),
            Err(e) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed with: {e:?}"),
            )
                .into_response(),
        }
    };
}

#[macro_export]
macro_rules! generate_by_params {
    ($key:ident, $key_display:expr, $default:expr) => {
        paste::paste! {
            fn [<get_page_ $key>](query: &Value) -> u64 {
                Self::[<inner_get_page_ $key>](query).unwrap_or(Self::[<default_page_ $key>]())
            }

            fn [<inner_get_page_ $key>](query: &Value) -> Option<u64> {
                let value = query.get(Self::[<page_ $key _param>]())?;
                value.as_u64()
            }

            #[inline]
            fn [<page_ $key _param>]() -> &'static str {
                concat!("page_", $key_display)
            }

            #[inline]
            fn [<default_page_ $key>]() -> u64 {
                $default
            }
        }
    };
}

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
    async fn http_create(
        Json(data): Json<<T::Entity as EntityTrait>::Model>,
    ) -> Result<StatusCode> {
        let mut active_model: T = data.into_active_model();
        tracing::debug!("http create: before not set pk {active_model:?}");
        for key in <T::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            active_model.not_set(col);
        }
        tracing::debug!("active model is {active_model:?}");
        let result = active_model
            .insert(Self::get_db_connection().await)
            .await
            .context(OperateDatabaseSnafu)?;
        tracing::debug!("create model {result:?}");
        Ok(StatusCode::CREATED)
    }

    fn set_model_primary_key(active_model: &mut T, value: u64) {
        if let Some(key) = <T::Entity as EntityTrait>::PrimaryKey::iter().next() {
            let col = key.into_column();
            active_model.set(col, sea_orm::Value::BigInt(Some(value as i64)));
        }
    }

    /// PUT a json body to /api/:id
    /// change a line in database
    /// return http 200 StatusCode::OK
    async fn http_update(
        Path(id): Path<u64>,
        Json(data): Json<<T::Entity as EntityTrait>::Model>,
    ) -> Result<StatusCode> {
        Self::check_instance_exists(id).await?;
        let mut active_model = data.into_active_model().reset_all();
        Self::set_model_primary_key(&mut active_model, id);
        tracing::debug!("update active model {active_model:?}");
        let result = active_model
            .update(Self::get_db_connection().await)
            .await
            .context(OperateDatabaseSnafu)?;
        tracing::debug!("update {result:?}");
        Ok(StatusCode::OK)
    }

    async fn check_instance_exists(pk: u64) -> Result<<T::Entity as EntityTrait>::Model> {
        Ok(
            <T::Entity as EntityTrait>::find_by_id(Self::exchange_primary_key(pk))
                .one(Self::get_db_connection().await)
                .await
                .context(OperateDatabaseSnafu)?
                .context(PrimaryKeyNotFoundSnafu { pk })?,
        )
    }

    /// PATCH a json body to /api/:id
    /// return http 200 StatusCode::OK if matched, or return 404 if not matched a query
    async fn http_partial_update(
        Path(id): Path<u64>,
        Json(data): Json<Value>,
    ) -> Result<StatusCode> {
        let model = Self::check_instance_exists(id).await?;
        let mut active_model = model.into_active_model();
        for (k, v) in data.as_object().unwrap() {
            if let Ok(column) = <T::Entity as EntityTrait>::Column::from_str(k) {
                active_model.set(column, (*v).clone().into());
            }
        }
        let result = active_model.update(Self::get_db_connection().await).await;
        tracing::debug!("patch with result: {result:?}");
        Ok(StatusCode::OK)
    }

    /// GET list results with /api
    /// you can set page_size and page_num to page results with url like /api?page_size=10 or /api?page_size=10&page_num=1
    /// return results with StatusCode::OK
    async fn http_list(Query(query): Query<Value>) -> Result<Json<Value>> {
        let db = Self::get_db_connection().await;
        let page_size = Self::get_page_size(&query);
        let results = if !page_size.eq(&0) {
            T::Entity::find()
                .into_model()
                .paginate(db, page_size)
                .fetch_page(Self::get_page_num(&query))
                .await
                .context(OperateDatabaseSnafu)?
        } else {
            <T::Entity as EntityTrait>::find()
                .all(Self::get_db_connection().await)
                .await
                .context(OperateDatabaseSnafu)?
        };
        Ok(Json(serde_json::json!(results)))
    }

    /// GET a single query result with /api/:id
    /// return http 200 with result or 404 if query not matched
    async fn http_retrieve(Path(id): Path<u64>) -> Result<Response> {
        let pk = Self::exchange_primary_key(id);
        Ok(Json(
            <T::Entity as EntityTrait>::find_by_id(pk)
                .one(Self::get_db_connection().await)
                .await
                .context(OperateDatabaseSnafu)?
                .context(PrimaryKeyNotFoundSnafu { pk: id })?,
        )
        .into_response())
    }

    /// DELETE a instance with /api/:id
    /// return http 204 if success delete or http 404 if not matched or http 500 with error info
    async fn http_delete(Path(id): Path<u64>) -> Result<StatusCode> {
        let pk = Self::exchange_primary_key(id);
        let db = Self::get_db_connection().await;
        <T::Entity as EntityTrait>::find_by_id(pk)
            .one(db)
            .await
            .context(OperateDatabaseSnafu)?
            .context(PrimaryKeyNotFoundSnafu { pk: id })?
            .delete(db)
            .await
            .context(OperateDatabaseSnafu)?;
        Ok(StatusCode::NO_CONTENT)
    }

    async fn http_delete_all() -> Result<StatusCode> {
        let db = Self::get_db_connection().await;
        <T::Entity as EntityTrait>::delete_many()
            .exec(db)
            .await
            .context(OperateDatabaseSnafu)?;
        Ok(StatusCode::NO_CONTENT)
    }

    /// change a value u64 into primary key
    #[inline]
    fn exchange_primary_key(
        id: u64,
    ) -> <<T::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType {
        <<T::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType::try_from_u64(id)
            .unwrap()
    }

    generate_by_params! {size, "size", 20}
    generate_by_params! {num, "num", 1}

    /// get http routers with full operates
    fn http_router(nest_prefix: &'static str) -> Router
    where
        Self: Send + 'static,
    {
        Router::new().nest(
            nest_prefix,
            Router::new()
                .route(
                    "/:id",
                    get(Self::http_retrieve)
                        .patch(Self::http_partial_update)
                        .put(Self::http_update)
                        .delete(Self::http_delete),
                )
                .route(
                    "/",
                    get(Self::http_list)
                        .post(Self::http_create)
                        .delete(Self::http_delete_all),
                ),
        )
    }
}
