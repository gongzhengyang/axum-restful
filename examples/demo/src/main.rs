use axum::http::StatusCode;
use axum::Router;
use chrono::SubsecRound;
use schemars::JsonSchema;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};
use sea_orm_migration::prelude::MigratorTrait;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum_restful::swagger::SwaggerGenerator;
use axum_restful::test_helpers::TestClient;
use axum_restful::views::ModelView;
use entities::student;
use once_cell::sync::Lazy;

use crate::entities::student::ActiveModel;

mod entities;

const INSTANCE_LEN: usize = 10;

static BASIC_MODELS: Lazy<Vec<student::Model>> = Lazy::new(|| {
    let mut models = vec![];
    for i in 1..INSTANCE_LEN + 1 {
        models.push(student::Model {
            id: i as i64,
            /// keep id is 1
            name: format!("test name {i}"),
            region: format!("test region {i}"),
            age: 1 + (i as i16),
            create_time: chrono::Local::now().naive_local().trunc_subsecs(3),
            score: i as f64 / 2.0,
            gender: (i as u64 % 2).eq(&0),
        });
    }
    models
});

async fn test_create(client: &TestClient, path: &str, db: &DatabaseConnection) {
    for (_, model) in BASIC_MODELS.iter().enumerate() {
        let body = serde_json::json!(model);
        let res = client.post(path).json(&body).send().await;
        assert_eq!(res.status(), StatusCode::CREATED);
        let query_model = student::Entity::find()
            .order_by_desc(student::Column::Id)
            .one(db)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(&query_model, model);
    }
}

async fn test_curd_operate_correct(app: Router, path: &str, db: &DatabaseConnection) {
    let client = TestClient::new(app.clone());

    test_create(&client, path, db).await;

    // test GET list correct
    let res = client.get(path).send().await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res
        .json::<Vec<student::Model>>()
        .await
        .len()
        .eq(&INSTANCE_LEN))

    // // detail operate or query
    // let detail_path = format!("{path}/1");
    // let detail_path_str = detail_path.as_str();
    //
    // // test PUT correct
    // let body = serde_json::json!({"id": 1, "name": "put-name", "region": "put-region", "age": 11});
    // let res = client.put(detail_path_str).json(&body).send().await;
    // assert_eq!(res.status(), StatusCode::OK);
    // let model = student::Entity::find_by_id(1)
    //     .one(db)
    //     .await
    //     .unwrap()
    //     .unwrap();
    // let put_model = student::Model {
    //     id: 1,
    //     age: 11,
    //     region: "put-region".to_owned(),
    //     name: "put-name".to_owned(),
    // };
    // assert_eq!(model, put_model);
    //
    // // test PATCH correct
    // let body = serde_json::json!({"name": "patch-name"});
    // let res = client.patch(detail_path_str).json(&body).send().await;
    // assert_eq!(res.status(), StatusCode::OK);
    // let model = student::Entity::find().one(db).await.unwrap().unwrap();
    // let patch_model = student::Model {
    //     name: "patch-name".to_owned(),
    //     ..put_model
    // };
    // assert_eq!(model, patch_model);
    //
    // // test GET a single instance
    // let res = client.get(detail_path_str).json(&body).send().await;
    // assert_eq!(res.status(), StatusCode::OK);
    //
    // // test delete a instance
    // let res = client.delete(detail_path_str).send().await;
    // assert_eq!(res.status(), StatusCode::NO_CONTENT);
    // let results = student::Entity::find().all(db).await.unwrap();
    // assert_eq!(results.len(), 0);
    //
    // tracing::info!("all tests success");
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
