use std::sync::Arc;

use aide::{
    axum::{
        ApiRouter,
        routing::{get, get_with},
    },
    openapi::{OpenApi, Tag},
    redoc::Redoc,
    transform::TransformOpenApi,
};
use async_trait::async_trait;
use axum::{
    body::{boxed, Full}, Extension,
    http::{header, StatusCode, Uri}, Json,
    response::{IntoResponse, Response},
    handler::HandlerWithoutStateExt,
    Router, ServiceExt};
use rust_embed::RustEmbed;

#[async_trait]
pub trait SwaggerGenerator {
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

    async fn docs_routes() -> ApiRouter {
        // We infer the return types for these routes
        // as an example.
        //
        // As a result, the `serve_redoc` route will
        // have the `text/html` content-type correctly set
        // with a 200 status.
        aide::gen::infer_responses(true);

        let router = ApiRouter::new()
            .api_route_with(
                "/",
                get_with(
                    Redoc::new("/docs/private/api.json")
                        .with_title("Axum restful")
                        .axum_handler(),
                    |op| op.description("This documentation page."),
                ),
                |p| p.security_requirement("ApiKey"),
            )
            .route("/private/api.json", get(Self::serve_docs));

        // Afterwards we disable response inference because
        // it might be incorrect for other routes.
        aide::gen::infer_responses(false);

        router
    }

    async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> Response {
        Json(api).into_response()
    }

    async fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
        api.title("Aide axum Open API")
            .summary("An example Todo application")
            .description("Axum-restful")
            .tag(Tag {
                name: "todo".into(),
                description: Some("Todo Management".into()),
                ..Default::default()
            })
            .security_scheme(
                "ApiKey",
                aide::openapi::SecurityScheme::ApiKey {
                    location: aide::openapi::ApiKeyLocation::Header,
                    name: "X-Auth-Key".into(),
                    description: Some("A key that is ignored.".into()),
                    extensions: Default::default(),
                },
            )
    }

    async fn api_server() {
        aide::gen::on_error(|error| {
            println!("{error}");
        });

        aide::gen::extract_schemas(true);
        let mut api = OpenApi::default();

        let static_router = ApiRouter::new()
            .route("/", get(Self::static_index_handler))
            .route_service("/*file", Self::static_file_handler.into_service());

        let app = ApiRouter::new()
            .nest_api_service("/docs", Self::docs_routes)
            .nest("/docs/swaggger/", static_router)
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
