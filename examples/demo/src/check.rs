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

struct HTTPOperateCheck {
    pub client: TestClient,
    pub path: String,
    pub db: &'static DatabaseConnection
}

impl HTTPOperateCheck {
    #[inline]
    fn path(&self) -> &str {
        &self.path
    }

    async fn test_create(&self) {
        for model in BASIC_MODELS.iter() {
            let body = serde_json::json!(model);
            let res = self.client.post(self.path()).json(&body).send().await;
            assert_eq!(res.status(), StatusCode::CREATED);
            let query_model = student::Entity::find()
                .order_by_desc(student::Column::Id)
                .one(self.db)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(&query_model, model);
        }
    }

    async fn test_list(&self) {
        let res = self.client.get(path).send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let models = res.json::<Vec<student::Model>>().await;
        assert_eq!(models.len(), INSTANCE_LEN);
        let original_models = BASIC_MODELS.iter().rev().collect::<Vec<&student::Model>>();
        for (index, model) in models.iter().enumerate() {
            assert_eq!(model, original_models[index]);
        }
    }

    async fn test_retrive(&self) {
        for model in BASIC_MODELS.iter() {
            self.check_db_model_eq(model).await;
        }
    }

    async fn check_db_model_eq(&self, model: &student::Model) {
        let path = format!("{}/{}",self.path(), model.id);
        let res = self.client.get(&path).send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(model, &res.json::<student::Model>().await);
    }

    async fn reset_all_models(&self) {
        for model in BASIC_MODELS.iter() {
            let detail_path = format!("{}/{}", self.path(), model.id);
            let res = self.client.put(&detail_path).json(model).send().await;
            assert_eq!(res.status(), StatusCode::OK);
            self.check_db_model_eq(model).await;
        }
    }

    async fn test_put(&self){
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
            // check change all the fields
            let detail_path = format!("{path}/{}", model.id);
            let res = self.client.put(&detail_path).json(&put_model).send().await;
            assert_eq!(res.status(), StatusCode::OK);
            put_model.id = model.id;
            self.check_db_model_eq(&put_model).await;
        }
    }

    async fn test_patch(&self) {
        for model in BASIC_MODELS.iter() {
            let name = format!("patch {}", model.name);
            let patch_body = serde_json!({
                "name": name,
            });
            let detail_path = format!("{path}/{}", model.id);
            let res = self.client.patch(&detail_path).json(patch_body).send().await;
            assert_eq!(res.status(), StatusCode::OK);
            let mut patch_model = model;
            patch_model.name = patch_body["name"].as_str().unwrap();
            self.check_db_model_eq(patch_model).await;
        }
    }
}

pub async fn test_curd_operate_correct(app: Router, path: &str, db: &'static DatabaseConnection) {
    let client = TestClient::new(app.clone());

    let c = HTTPOperateCheck {
        client: TestClient::new(app.clone()),
        path: path.to_owned(),
        db: db
    };
    c.test_create().await;
    c.test_list().await;
    c.test_retrive().await;
    c.test_put().await;
    c.test_patch().await;
}