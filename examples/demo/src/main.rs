use axum::http::StatusCode;
use axum::Router;
use sea_orm::*;
use sea_orm::EntityTrait;

use axum_restful::test_helpers::TestClient;
use axum_restful::views::ModelView;
use entities::student;
pub use sea_orm_migration::prelude::MigratorTrait;

mod entities;

#[tokio::main]
async fn main() {
    let db = axum_restful::get_db_connection_pool().await;
    let _ = migration::Migrator::down(db, None).await;
    migration::Migrator::up(db, None).await.unwrap();

    struct StudentView;
    impl ModelView<student::ActiveModel> for StudentView {}
    let path = "/api/student";
    let app = Router::new().nest(path, StudentView::get_http_routes());
    let client = TestClient::new(app);
    let body = serde_json::json!({"name": "test-name", "region": "test-region", "age": "test-age"});
    let res = client.post(path).json(&body).send().await;
    assert_eq!(res.status(), StatusCode::CREATED);
    let post_model = student::Model {
        id: 1,
        name: "test-name".to_owned(),
        region: "test-region".to_owned(),
        age: "test-age".to_owned(),
    };
    let query_model = student::Entity::find().one(db).await.unwrap().unwrap();
    assert_eq!(query_model, post_model);

    let res = client.get(path).send().await;
    assert_eq!(res.status(), StatusCode::OK);
    // detail operate or query
    let detail_path = format!("{path}/1");
    let detail_path_str = detail_path.as_str();

    let body = serde_json::json!({"name": "put-name", "region": "put-region", "age": "put-age"});
    let res = client.put(detail_path_str).json(&body).send().await;
    assert_eq!(res.status(), StatusCode::OK);
    let model = student::Entity::find().one(db).await.unwrap().unwrap();
    assert_eq!(model.name, "patch-name".to_owned());
    let put_model = student::Model {
        id: 1,
        age: "put-age".to_owned(),
        region: "put-region".to_owned(),
        name: "put-name".to_owned(),
    };
    assert_eq!(model, put_model);

    let body = serde_json::json!({"name": "patch-name"});
    let res = client.patch(detail_path_str).json(&body).send().await;
    assert_eq!(res.status(), StatusCode::OK);
    let model = student::Entity::find().one(db).await.unwrap().unwrap();
    let patch_model = student::Model {
        name: "patch-name".to_owned(),
        ..put_model
    };
    assert_eq!(model, patch_model);

    let res = client.get(detail_path_str).json(&body).send().await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = client.delete(detail_path_str).send().await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let results = student::Entity::find().all(db).await.unwrap();
    assert_eq!(results.len(), 0);
}
