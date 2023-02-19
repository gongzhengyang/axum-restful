use std::fmt::Debug;

use sea_orm::{
    ModelTrait,
    ActiveModelTrait
};

use axum::{http::StatusCode, Json, response::Response};
use async_trait::async_trait;

#[async_trait]
pub trait ModelView {
    type ActiveModel: ActiveModelTrait;
    type Model: ModelTrait;

    async fn post(Json(data): Json<serde_json::Value>) -> Result<StatusCode, Response> {
        Ok(StatusCode::CREATED)
    }

    async fn patch() -> Result<StatusCode, Response> {
        Ok(StatusCode::OK)
    }

    async fn put() -> Result<StatusCode, Response> {
        Ok(StatusCode::OK)
    }

    async fn get() -> Result<StatusCode, Response> {
        Ok(StatusCode::OK)
    }
}

#[async_trait]
impl<T> ModelView for T
where
    T: Debug,
    T::ActiveModel: ActiveModelTrait,
    T::Model: ModelTrait
{

}


#[derive(Debug)]
struct Model {

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_view() {
        let result = Model::post().await;
        assert_eq!(result.unwrap(), StatusCode::CREATED);
    }
}