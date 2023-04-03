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
    body::{boxed, Full},
    handler::HandlerWithoutStateExt,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response},
    Extension, Json,
};
use rust_embed::RustEmbed;
use schemars::JsonSchema;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EntityTrait, IntoActiveModel};
use serde::Serialize;

use crate::views::ModelView;

#[async_trait]
pub trait SwaggerGenerator<T>: 'static + ModelView<T>
where
    T: ActiveModelTrait + ActiveModelBehavior + Send + 'static + Sync,
    <T::Entity as EntityTrait>::Model: IntoActiveModel<T> + Serialize + Sync + JsonSchema,
    for<'de> <T::Entity as EntityTrait>::Model: serde::de::Deserialize<'de>,
{
    async fn static_index_handler() -> Response {
        Self::static_file_handler("/index.html".parse::<Uri>().unwrap()).await
    }

    async fn static_file_handler(uri: Uri) -> Response {
        let mut path = uri.path().trim_start_matches('/').to_string();
        if path.starts_with("statics/") {
            path = path.replace("statics/", "");
        }
        StaticFile(path).into_response()
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
                    Redoc::new("/docs/openapi/api.json")
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

    fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
        api.title("Aide axum Open API for axum-restful")
            .summary("axum-restful openapi")
            .description("Axum-restful")
    }

    fn http_retrieve_docs(op: TransformOperation) -> TransformOperation {
        op.summary("get")
            .response::<200, Json<<T::Entity as EntityTrait>::Model>>()
    }

    fn http_partial_update_docs(op: TransformOperation) -> TransformOperation {
        op.summary("partial update")
            .response_with::<200, (), _>(|res| res.description("partial update success"))
    }

    fn http_update_docs(op: TransformOperation) -> TransformOperation {
        op.summary("update")
            .response_with::<200, (), _>(|res| res.description("update success"))
    }

    fn http_delete_docs(op: TransformOperation) -> TransformOperation {
        op.summary("delete")
            .response_with::<204, (), _>(|res| res.description("deleete success"))
    }

    fn http_list_docs(op: TransformOperation) -> TransformOperation {
        op.summary("list the objects")
            .response::<200, Json<Vec<<T::Entity as EntityTrait>::Model>>>()
    }

    fn http_create_docs(op: TransformOperation) -> TransformOperation {
        op.summary("create")
            .response_with::<201, (), _>(|res| res.description("create success"))
    }

    async fn api_server() {
        aide::gen::on_error(|error| {
            tracing::error!("aide gen error: {error}");
        });

        aide::gen::extract_schemas(true);
        let mut api = OpenApi::default();

        let static_router = ApiRouter::new()
            .route_service("/", Self::static_index_handler.into_service())
            .route_service("/*file", Self::static_file_handler.into_service());

        let model_router = ApiRouter::new()
            .api_route(
                "/:id",
                get_with(Self::http_retrieve, Self::http_retrieve_docs)
                    .patch_with(Self::http_partial_update, Self::http_partial_update_docs)
                    .put_with(Self::http_update, Self::http_update_docs)
                    .delete_with(Self::http_delete, Self::http_delete_docs),
            )
            .api_route(
                "/",
                get_with(Self::http_list, Self::http_list_docs)
                    .post_with(Self::http_create, Self::http_create_docs),
            );

        let app = ApiRouter::new()
            .nest_api_service("/api", model_router)
            .nest("/docs/swagger", static_router)
            .nest_service("/docs/openapi", Self::openapi_routes())
            .finish_api_with(&mut api, Self::api_docs)
            .layer(Extension(Arc::new(api)));
        axum::Server::bind(&"0.0.0.0:3002".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

#[derive(RustEmbed)]
#[folder = "statics/swagger"]
struct Asset;

pub struct StaticFile<T>(pub T);

impl<T> IntoResponse for StaticFile<T>
where
    T: Into<String>,
{
    fn into_response(self) -> Response {
        let path = self.0.into();
        match Asset::get(path.as_str()) {
            Some(content) => {
                let body = boxed(Full::from(content.data));
                let mime = mime_guess::from_path(path).first_or_octet_stream();
                Response::builder()
                    .header(header::CONTENT_TYPE, mime.as_ref())
                    .body(body)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(boxed(Full::from("404")))
                .unwrap(),
        }
    }
}
