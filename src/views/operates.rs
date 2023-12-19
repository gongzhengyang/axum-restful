use std::any::type_name;

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
    Iterable, ModelTrait, PaginatorTrait, PrimaryKeyToColumn, PrimaryKeyTrait, QueryOrder,
    TryFromU64,
};
use serde::Serialize;
use serde_json::Value;
use snafu::{OptionExt, ResultExt};

use crate::error::{OperateDatabaseSnafu, PrimaryKeyNotFoundSnafu};
use crate::{db, error::Result, generate_by_params};

#[async_trait]
pub trait ModelViewExt<T>
where
    T: ActiveModelTrait + ActiveModelBehavior + Send + 'static + Sync,
    <T::Entity as EntityTrait>::Model: IntoActiveModel<T> + Serialize + Sync,
    for<'de> <T::Entity as EntityTrait>::Model: serde::de::Deserialize<'de>,
{
    #[inline]
    fn modle_name() -> String {
        let name = type_name::<Self>().to_lowercase();
        match name.rsplit_once("::") {
            None => name,
            Some((_, last)) => last.to_owned(),
        }
    }

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
        tracing::debug!(
            "[{}] http create: before not set pk {active_model:?}",
            Self::modle_name()
        );
        for key in <T::Entity as EntityTrait>::PrimaryKey::iter() {
            let col = key.into_column();
            active_model.not_set(col);
        }
        tracing::debug!(
            "[{}] http create: active model is {active_model:?}",
            Self::modle_name()
        );
        let result = active_model
            .insert(Self::get_db_connection().await)
            .await
            .context(OperateDatabaseSnafu)?;
        tracing::debug!(
            "[{}] http create: create model {result:?}",
            Self::modle_name()
        );
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
        Path(pk): Path<u64>,
        Json(data): Json<<T::Entity as EntityTrait>::Model>,
    ) -> Result<StatusCode> {
        tracing::debug!("[{}] http update check: {pk}", Self::modle_name());
        Self::check_instance_exists(pk).await?;
        let mut active_model = data.into_active_model().reset_all();
        Self::set_model_primary_key(&mut active_model, pk);
        tracing::debug!(
            "[{}] http update: active pk: {pk} active model: {active_model:?}",
            Self::modle_name()
        );
        let result = active_model
            .update(Self::get_db_connection().await)
            .await
            .context(OperateDatabaseSnafu)?;
        tracing::debug!("[{}] http update: result {result:?}", Self::modle_name());
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

    /// TODO: patch need value trans into active model value
    /// PATCH a json body to /api/:id
    /// return http 200 StatusCode::OK if matched, or return 404 if not matched a query
    // async fn http_partial_update(
    //     Path(pk): Path<u64>,
    //     Json(data): Json<Value>,
    // ) -> Result<StatusCode> {
    //     tracing::debug!("[{}] http patch check: {pk}", Self::modle_name());
    //     let model = Self::check_instance_exists(pk).await?;
    //     let mut active_model = model.into_active_model();
    //     tracing::debug!(
    //         "[{}] http patch: original pk: {pk} active model: {active_model:?}",
    //         Self::modle_name()
    //     );
    //     for (k, v) in data.as_object().unwrap() {
    //         tracing::debug!("[{}] http patch set {}: {}", Self::modle_name(), k, v);
    //         if let Ok(column) = <T::Entity as EntityTrait>::Column::from_str(k) {
    //             // let col_type = column.def().get_column_type()
    //             // active_model.set(column, v);
    //         }
    //     }
    //     tracing::debug!(
    //         "[{}] http patch: ative model pk: {pk} active model: {active_model:?}",
    //         Self::modle_name()
    //     );
    //     let result = active_model.update(Self::get_db_connection().await).await;
    //     tracing::debug!("[{}] http patch: result {result:?}", Self::modle_name());
    //     Ok(StatusCode::OK)
    // }

    fn order_by_desc() -> <T::Entity as EntityTrait>::Column;

    /// GET list results with /api
    /// you can set page_size and page_num to page results with url like /api?page_size=10 or /api?page_size=10&page_num=1
    /// return results with StatusCode::OK
    async fn http_list(Query(query): Query<Value>) -> Result<Json<Value>> {
        let db = Self::get_db_connection().await;
        let page_size = Self::get_page_size(&query);
        let results = if !page_size.eq(&0) {
            T::Entity::find()
                .order_by_desc(Self::order_by_desc())
                .into_model()
                .paginate(db, page_size)
                .fetch_page(Self::get_page_num(&query))
                .await
                .context(OperateDatabaseSnafu)?
        } else {
            tracing::debug!("http list: fetch all");
            <T::Entity as EntityTrait>::find()
                .order_by_desc(Self::order_by_desc())
                .all(Self::get_db_connection().await)
                .await
                .context(OperateDatabaseSnafu)?
        };
        tracing::debug!("http list: fetch results len {}", results.len());
        Ok(Json(serde_json::json!(results)))
    }

    /// GET a single query result with /api/:id
    /// return http 200 with result or 404 if query not matched
    async fn http_retrieve(Path(pk): Path<u64>) -> Result<Response> {
        tracing::debug!("[{}] http retrive: pk: {pk}", Self::modle_name());
        Ok(Json(
            <T::Entity as EntityTrait>::find_by_id(Self::exchange_primary_key(pk))
                .one(Self::get_db_connection().await)
                .await
                .context(OperateDatabaseSnafu)?
                .context(PrimaryKeyNotFoundSnafu { pk })?,
        )
        .into_response())
    }

    /// DELETE a instance with /api/:id
    /// return http 204 if success delete or http 404 if not matched or http 500 with error info
    async fn http_delete(Path(pk): Path<u64>) -> Result<StatusCode> {
        let db = Self::get_db_connection().await;
        tracing::debug!("[{}] http delete: pk: {pk}", Self::modle_name());
        <T::Entity as EntityTrait>::find_by_id(Self::exchange_primary_key(pk))
            .one(db)
            .await
            .context(OperateDatabaseSnafu)?
            .context(PrimaryKeyNotFoundSnafu { pk })?
            .delete(db)
            .await
            .context(OperateDatabaseSnafu)?;
        tracing::debug!("[{}] http delete: success pk: {pk}", Self::modle_name());
        Ok(StatusCode::NO_CONTENT)
    }

    async fn http_delete_all() -> Result<StatusCode> {
        let db = Self::get_db_connection().await;
        tracing::debug!("[{}] http delete all", Self::modle_name());
        <T::Entity as EntityTrait>::delete_many()
            .exec(db)
            .await
            .context(OperateDatabaseSnafu)?;
        tracing::debug!("[{}] http delete all success", Self::modle_name());
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
    generate_by_params! {num, "num", 0, 1}

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
