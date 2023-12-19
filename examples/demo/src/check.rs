use axum::http::StatusCode;
use axum::Router;
use chrono::SubsecRound;
use once_cell::sync::Lazy;
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};

use axum_restful::test_helpers::TestClient;

use crate::entities::student;

const INSTANCE_LEN: usize = 10;

static BASIC_MODELS: Lazy<Vec<student::Model>> = Lazy::new(|| {
    let mut models = vec![];
    for i in 1..INSTANCE_LEN + 1 {
        models.push(student::Model {
            id: i as i64,
            // keep id is 1
            name: format!("check name {i}"),
            region: format!("check region {i}"),
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
    pub db: &'static DatabaseConnection,
}

impl HTTPOperateCheck {
    #[inline]
    fn path(&self) -> &str {
        &self.path
    }

    async fn check_create(&self, check_equal: bool) {
        tracing::info!("http ceate check");
        for model in BASIC_MODELS.iter() {
            let body = serde_json::json!(model);
            let res = self.client.post(self.path()).json(&body).send().await;
            assert_eq!(res.status(), StatusCode::CREATED);
            if check_equal {
                let query_model = student::Entity::find()
                    .order_by_desc(student::Column::Id)
                    .one(self.db)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(&query_model, model);
            }
        }
    }

    async fn check_list(&self) {
        tracing::info!("http list check");
        let res = self.client.get(self.path()).send().await;
        assert_eq!(res.status(), StatusCode::OK);
        let models = res.json::<Vec<student::Model>>().await;
        assert_eq!(models.len(), INSTANCE_LEN);
        let original_models = BASIC_MODELS.iter().rev().collect::<Vec<&student::Model>>();
        for (index, model) in models.iter().enumerate() {
            assert_eq!(model, original_models[index]);
        }
        let page_size = 3;
        for page_num in 1..3 {
            let resp = self
                .client
                .get(&format!(
                    "{}?page_size={page_size}&page_num={page_num}",
                    self.path()
                ))
                .send()
                .await;
            assert_eq!(resp.status(), StatusCode::OK);
            let results = resp.json::<Vec<student::Model>>().await;
            assert_eq!(results.len(), page_size);
            let start = (page_num - 1) * page_size;
            assert_eq!(
                results.iter().collect::<Vec<&student::Model>>()[..],
                original_models[start..start + page_size]
            );
        }
    }

    async fn check_retrive(&self) {
        tracing::info!("http retrive check");
        for model in BASIC_MODELS.iter() {
            self.check_db_model_eq(model).await;
        }
    }

    async fn check_db_model_eq(&self, model: &student::Model) {
        let path = format!("{}/{}", self.path(), model.id);
        let res = self.client.get(&path).send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(model, &res.json::<student::Model>().await);
    }

    async fn reset_all_models(&self) {
        tracing::info!("reset all models");
        for model in BASIC_MODELS.iter() {
            let detail_path = format!("{}/{}", self.path(), model.id);
            let res = self.client.put(&detail_path).json(model).send().await;
            assert_eq!(res.status(), StatusCode::OK);
            self.check_db_model_eq(model).await;
        }
    }

    async fn check_put(&self) {
        tracing::info!("http put check");
        for model in BASIC_MODELS.iter() {
            let mut put_model = model.clone();
            // check wrong body id
            put_model.id *= 10;
            tracing::info!("check wrong id {}", put_model.id);
            put_model.name = format!("changed {}", put_model.name);
            put_model.region = format!("changed {} region", put_model.region);
            put_model.age *= 2;
            put_model.create_time = chrono::Local::now().naive_local().trunc_subsecs(3);
            put_model.score += 99.0;
            put_model.gender = !put_model.gender;
            assert_ne!(model, &put_model);
            // check change all the fields
            let detail_path = format!("{}/{}", self.path(), model.id);
            let res = self.client.put(&detail_path).json(&put_model).send().await;
            assert_eq!(res.status(), StatusCode::OK);
            put_model.id = model.id;
            tracing::info!("put {}", serde_json::json!(put_model));
            self.check_db_model_eq(&put_model).await;
        }
        let resp = self
            .client
            .put(&format!("{}/{}", self.path(), u16::MAX))
            .json(BASIC_MODELS.iter().next().unwrap())
            .send()
            .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    async fn check_delete(&self) {
        tracing::info!("http delete single check");
        for model in BASIC_MODELS.iter() {
            let detail_path = format!("{}/{}", self.path(), model.id);
            let resp = self.client.delete(&detail_path).send().await;
            assert_eq!(resp.status(), StatusCode::NO_CONTENT);
            let resp = self.client.get(&detail_path).send().await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }
        self.check_is_empty().await;
        let resp = self
            .client
            .delete(&format!("{}/{}", self.path(), u16::MAX))
            .json(BASIC_MODELS.iter().next().unwrap())
            .send()
            .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        tracing::info!("recreate all");
        self.check_create(false).await;
        let resp = self.client.delete(self.path()).send().await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
        self.check_is_empty().await;
    }

    async fn check_is_empty(&self) {
        let resp = self
            .client
            .get(self.path())
            .send()
            .await
            .json::<Vec<student::Model>>()
            .await;
        assert!(resp.is_empty());
    }
}

pub async fn check_curd_operate_correct(app: Router, path: &str, db: &'static DatabaseConnection) {
    let c = HTTPOperateCheck {
        client: TestClient::new(app.clone()),
        path: path.to_owned(),
        db,
    };
    tracing::warn!("check for curd, this will generate some error level `PrimaryKeyNotFound`, it's for check, please ignore it.");
    c.check_create(true).await;
    c.check_list().await;
    c.check_retrive().await;
    c.check_put().await;
    c.reset_all_models().await;
    c.check_delete().await;
}
