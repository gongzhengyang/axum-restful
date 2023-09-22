use std::os::fd::AsFd;
use axum::http::StatusCode;
use axum::Router;
use chrono::SubsecRound;
use once_cell::sync::Lazy;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};
use sea_orm_migration::prelude::MigratorTrait;
use serde_json::de::Read;

use axum_restful::test_helpers::TestClient;

use crate::entities::student;

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
    for model in BASIC_MODELS.iter() {
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

async fn test_list(client: &TestClient, path: &str, db: &DatabaseConnection) {
    let res = client.get(path).send().await;
    assert_eq!(res.status(), StatusCode::OK);
    let models = res.json::<Vec<student::Model>>().await;
    assert_eq!(models.len(), INSTANCE_LEN);
    let original_models = BASIC_MODELS.iter().rev().collect::<Vec<&student::Model>>();
    for (index, model) in models.iter().enumerate() {
       assert_eq!(model, original_models[index]);
    }
}

async fn test_retrive(client: &TestClient, path: &str, db: &DatabaseConnection) {
    for model in BASIC_MODELS.iter() {
        check_db_model_eq(model, path, client).await;
    }
}

async fn check_db_model_eq(model: &student::Model, path: &str, client: &TestClient) {
    let path = format!("{path}/{}", model.id);
    let res = client.get(&path).send().await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_eq!(model, &res.json::<student::Model>().await);
}

async fn test_put(client: &TestClient, path: &str, db: &DatabaseConnection){
    for model in BASIC_MODELS.iter() {
        let mut put_model = model.clone();
        // check wrong body id
        put_model.id *= 10;
        put_model.name = format!("changed {}", put_model.name);
        put_model.region = format!("changed {} region", put_model.region);
        put_model.age *= 2;
        put_model.create_time = chrono::Local::now().naive_local().trunc_subsecs(3);
        put_model.score += 99.0;
        put_model.gender = !put_model.gender;
        assert_ne!(model, &put_model);
        let res = client.put(&format!("{path}/{}", model.id)).json(&put_model).send().await;
        assert_eq!(res.status(), StatusCode::OK);
        put_model.id = model.id;
        check_db_model_eq(&put_model, path, client).await;
    }
}

pub async fn test_curd_operate_correct(app: Router, path: &str, db: &DatabaseConnection) {
    let client = TestClient::new(app.clone());

    test_create(&client, path, db).await;
    test_list(&client, path, db).await;
    test_retrive(&client, path, db).await;
    test_put(&client, path, db).await;

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