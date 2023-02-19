use std::fmt::Debug;

use axum::{
    http::StatusCode,
    response::Response
};
use async_trait::async_trait;

#[async_trait]
pub trait ModelView {
    async fn post() -> Result<StatusCode, Response> {
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
    T: Debug
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