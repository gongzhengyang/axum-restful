use axum::http::StatusCode;
use axum::Router;
use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, EntityTrait};
use sea_orm_migration::prelude::MigratorTrait;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum_restful::swagger::SwaggerGenerator;
use axum_restful::test_helpers::TestClient;
use axum_restful::views::ModelView;
use entities::student;

use crate::entities::student::ActiveModel;

mod entities;

async fn test_curd_operate_correct(app: Router, path: &str, db: &DatabaseConnection) {
    let client = TestClient::new(app.clone());

    // test POST create a instance
    let body = serde_json::json!({"name": "test-name", "region": "test-region", "age": 11});
    let res = client.post(path).json(&body).send().await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let post_model = student::Model {
        id: 1,
        name: "test-name".to_owned(),
        region: "test-region".to_owned(),
        age: 11,
    };
    let query_model = student::Entity::find().one(db).await.unwrap().unwrap();
    assert_eq!(query_model, post_model);

    // test GET list correct
    let res = client.get(path).send().await;
    assert_eq!(res.status(), StatusCode::OK);

    // detail operate or query
    let detail_path = format!("{path}/1");
    let detail_path_str = detail_path.as_str();

    // test PUT correct
    let body = serde_json::json!({"id": 1, "name": "put-name", "region": "put-region", "age": 11});
    let res = client.put(detail_path_str).json(&body).send().await;
    assert_eq!(res.status(), StatusCode::OK);
    let model = student::Entity::find().one(db).await.unwrap().unwrap();
    let put_model = student::Model {
        id: 1,
        age: 11,
        region: "put-region".to_owned(),
        name: "put-name".to_owned(),
    };
    assert_eq!(model, put_model);

    // test PATCH correct
    let body = serde_json::json!({"name": "patch-name"});
    let res = client.patch(detail_path_str).json(&body).send().await;
    assert_eq!(res.status(), StatusCode::OK);
    let model = student::Entity::find().one(db).await.unwrap().unwrap();
    let patch_model = student::Model {
        name: "patch-name".to_owned(),
        ..put_model
    };
    assert_eq!(model, patch_model);

    // test GET a single instance
    let res = client.get(detail_path_str).json(&body).send().await;
    assert_eq!(res.status(), StatusCode::OK);

    // test delete a instance
    let res = client.delete(detail_path_str).send().await;
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    let results = student::Entity::find().all(db).await.unwrap();
    assert_eq!(results.len(), 0);

    tracing::info!("all tests success");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let db = axum_restful::get_db_connection_pool().await;
    let _ = migration::Migrator::down(db, None).await;
    migration::Migrator::up(db, None).await.unwrap();
    tracing::info!("migrate success");

    /// student
    #[derive(JsonSchema)]
    struct StudentView;
    impl ModelView<ActiveModel> for StudentView {}

    let path = "/api/student";
    let app = StudentView::http_router(path);
    test_curd_operate_correct(app.clone(), path, db).await;

    // if you want to generate swagger docs
    // impl OperationInput and SwaggerGenerator and change app into http_routers_with_swagger
    impl aide::operation::OperationInput for student::Model {}
    impl axum_restful::swagger::SwaggerGenerator<student::ActiveModel> for StudentView {}
    let app = StudentView::http_router_with_swagger(path, StudentView::model_api_router());

    let addr = "0.0.0.0:3000";
    tracing::info!("listen at {addr}");
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}
