use std::sync::Arc;

use aide::{
    axum::{
        routing::{get, get_with},
        ApiRouter,
    },
    openapi::OpenApi,
    redoc::Redoc,
    transform::{TransformOpenApi, TransformOperation},
};
use async_trait::async_trait;
use axum::{
    handler::HandlerWithoutStateExt,
    http::Uri,
    response::{IntoResponse, Response},
    Extension, Json, Router,
};
use schemars::{gen, JsonSchema};
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde::Serialize;

use crate::views::ModelViewExt;

use super::asset::StaticFile;

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
    fn index_uri() -> &'static str {
        "/index.html"
    }

    async fn static_index_handler() -> Response {
        Self::static_file_handler(Self::index_uri().parse::<Uri>().unwrap()).await
    }

    #[inline]
    fn trim_static_prefix_pattern() -> &'static str {
        "statics/"
    }

    async fn static_file_handler(uri: Uri) -> Response {
        let mut path = uri.path().trim_start_matches('/').to_string();
        if path.starts_with(Self::trim_static_prefix_pattern()) {
            path = path.replace(Self::trim_static_prefix_pattern(), "");
        }
        StaticFile(path).into_response()
    }

    #[inline]
    fn redoc_openapi_json_url() -> &'static str {
        "/docs/openapi/api.json"
    }

    /// provide an openapi json for swagger/openapi to use with a ptah /docs/openapi/api.json
    /// if you want to change the OPENAPI json path, you must change `swagger-initializer.js`
    /// ```javascript
    ///   window.ui = SwaggerUIBundle({
    ///     url: "/docs/openapi/api.json",
    ///   });
    /// ```
    fn openapi_routes() -> ApiRouter {
        aide::gen::infer_responses(true);
        let router = ApiRouter::new()
            .api_route_with(
                "/",
                get_with(
                    Redoc::new(Self::redoc_openapi_json_url())
                        .with_title("Axum restful")
                        .axum_handler(),
                    |op| op.description("This documentation page."),
                ),
                |p| p.security_requirement("ApiKey"),
            )
            .route("/api.json", get(Self::serve_docs));
        aide::gen::infer_responses(false);

        router
    }

    async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> Response {
        Json(api).into_response()
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

    // fn http_partial_update_summary() -> String {
    //     format!(
    //         "partial update an instance: {}",
    //         Self::modle_schema_description()
    //     )
    // }
    //
    // fn http_partial_update_docs(op: TransformOperation) -> TransformOperation {
    //     op.summary(&Self::http_partial_update_summary())
    //         .input::<Json<<T::Entity as EntityTrait>::Model>>()
    //         .response_with::<200, (), _>(|res| res.description("partial update success"))
    // }

    fn http_update_summary() -> String {
        format!("update an instance {}", Self::modle_schema_description())
    }

    fn http_update_docs(op: TransformOperation) -> TransformOperation {
        op.summary(&Self::http_update_summary())
            .input::<Json<<T::Entity as EntityTrait>::Model>>()
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
            .input::<Json<<T::Entity as EntityTrait>::Model>>()
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

    fn http_router_with_swagger(nest_prefix: &'static str, model_api_router: ApiRouter) -> Router
    where
        Self: Send + 'static,
    {
        aide::gen::on_error(|error| {
            tracing::error!("swagger api gen error: {error}");
        });
        aide::gen::extract_schemas(true);

        let mut api = OpenApi::default();

        let static_router = ApiRouter::new()
            .route_service("/", Self::static_index_handler.into_service())
            .route_service("/*file", Self::static_file_handler.into_service());
        ApiRouter::new()
            .nest_api_service(nest_prefix, model_api_router)
            .nest("/docs/swagger/", static_router)
            .nest_service("/docs/openapi", Self::openapi_routes())
            .finish_api_with(&mut api, Self::api_docs_head_config)
            .layer(Extension(Arc::new(api)))
    }
}
