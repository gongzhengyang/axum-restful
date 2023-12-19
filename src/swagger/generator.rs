use std::sync::Arc;

use aide::{
    axum::{
        routing::{get, get_with},
        ApiRouter,
    },
    openapi::OpenApi,
    transform::{TransformOpenApi, TransformOperation},
};
use async_trait::async_trait;
use axum::{
    response::{IntoResponse, Response},
    Extension, Json, Router,
};
use schemars::{gen, JsonSchema};
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde::Serialize;
use tower_http::services::ServeDir;

use crate::views::ModelViewExt;

/// generate swagger docs for service
/// when the service is up
/// you can visit below
/// ```http
/// // swagger doc
/// http://{ipaddress}:{port}/docs/swagger/
/// // openapi doc
/// http://{ipaddress}:{port}/docs/openapi
/// ```
#[async_trait]
pub trait SwaggerGeneratorExt<T>: 'static + ModelViewExt<T>
where
    Self: Send + JsonSchema,
    T: ActiveModelTrait + ActiveModelBehavior + Send + 'static + Sync,
    <T::Entity as EntityTrait>::Model: IntoActiveModel<T> + Serialize + Sync + JsonSchema,
    for<'de> <T::Entity as EntityTrait>::Model: serde::de::Deserialize<'de>,
{
    fn modle_schema_description() -> String {
        let mut gen = gen::SchemaGenerator::default();
        let value = serde_json::json!(Self::json_schema(&mut gen));
        if let Some(description) = value["description"].as_str() {
            description.to_owned()
        } else {
            Self::modle_name()
        }
    }

    #[inline]
    fn serve_dir_path() -> &'static str {
        awesome_operates::embed::EXTRACT_SWAGGER_DIR_PATH
    }

    #[inline]
    fn redoc_openapi_json_url() -> &'static str {
        "/docs/openapi/api.json"
    }

    async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> Response {
        Json(serde_json::json!(*api)).into_response()
    }

    fn api_docs_head_config(api: TransformOpenApi) -> TransformOpenApi {
        api.title("Aide axum Open API for axum-restful")
            .summary("axum-restful openapi")
    }

    fn http_retrieve_summary() -> String {
        format!("fetch an instance: {}", Self::modle_schema_description())
    }

    fn http_retrieve_docs(op: TransformOperation) -> TransformOperation {
        op.summary(&Self::http_retrieve_summary())
            .response::<200, Json<<T::Entity as EntityTrait>::Model>>()
    }

    fn http_update_summary() -> String {
        format!("update an instance {}", Self::modle_schema_description())
    }

    fn http_update_docs(op: TransformOperation) -> TransformOperation {
        op.summary(&Self::http_update_summary())
            .response::<200, ()>()
    }

    fn http_delete_summary() -> String {
        format!("delete an instance {}", Self::modle_schema_description())
    }

    fn http_delete_docs(op: TransformOperation) -> TransformOperation {
        op.summary(&Self::http_delete_summary())
            .response::<204, ()>()
    }

    fn http_delete_all_summary() -> String {
        format!("delete all instances {}", Self::modle_schema_description())
    }

    fn http_delete_all_docs(op: TransformOperation) -> TransformOperation {
        op.summary(&Self::http_delete_all_summary())
            .response::<204, ()>()
    }

    fn http_list_summary() -> String {
        format!("list all instances {}", Self::modle_schema_description())
    }

    fn http_list_docs(op: TransformOperation) -> TransformOperation {
        op.summary(&Self::http_list_summary())
            .response::<200, Json<Vec<<T::Entity as EntityTrait>::Model>>>()
    }

    fn http_create_summary() -> String {
        format!("create an instance {}", Self::modle_schema_description())
    }

    fn http_create_docs(op: TransformOperation) -> TransformOperation {
        op.summary(&Self::http_create_summary())
            // .input::<Json<<T::Entity as EntityTrait>::Model>>()
            .response::<201, ()>()
    }

    fn model_api_router() -> ApiRouter {
        ApiRouter::new()
            .api_route(
                "/:id",
                get_with(Self::http_retrieve, Self::http_retrieve_docs)
                    .put_with(Self::http_update, Self::http_update_docs)
                    .delete_with(Self::http_delete, Self::http_delete_docs),
            )
            .api_route(
                "/",
                get_with(Self::http_list, Self::http_list_docs)
                    .post_with(Self::http_create, Self::http_create_docs)
                    .delete_with(Self::http_delete_all, Self::http_delete_all_docs),
            )
    }

    async fn http_router_with_swagger(
        nest_prefix: &'static str,
        model_api_router: ApiRouter,
    ) -> anyhow::Result<Router>
    where
        Self: Send + 'static,
    {
        let mut api = OpenApi::default();

        awesome_operates::extract_all_files!(awesome_operates::embed::Asset);
        awesome_operates::swagger::InitSwagger::new(
            awesome_operates::embed::EXTRACT_SWAGGER_DIR_PATH,
            "swagger-init.js",
            "index.html",
            "../api.json",
        )
        .build()
        .await?;
        Ok(ApiRouter::new()
            .nest_api_service(nest_prefix, model_api_router)
            .nest_service("/swagger", ServeDir::new(Self::serve_dir_path()))
            .route("/api.json", get(Self::serve_docs))
            .finish_api_with(&mut api, Self::api_docs_head_config)
            .layer(Extension(Arc::new(api))))
    }
}
