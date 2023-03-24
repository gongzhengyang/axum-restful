use axum::http::StatusCode;
use axum::response::IntoResponse;

/// Handle for not found and return 404
/// # Global handle not found Example
///
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use axum_restful::utils::handle_not_found;
///
/// let app = Router::new()
///     .route("/", get(|| async { "Hello world!"}))
///     .fallback(handle_not_found);
/// # async {
/// # axum::Server::bind(&"".parse().unwrap())
/// #     .serve(app.into_make_service())
/// #     .await
/// #     .unwrap();
/// # };
/// ```
pub async fn handle_not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use axum::{routing::get, Router};

    #[tokio::test]
    async fn handle_404() {
        let app = Router::new()
            .route("/", get(|| async { "Hello" }))
            .fallback(handle_not_found);
        let client = TestClient::new(app);
        let res = client.get("/").send().await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await, "Hello".to_owned());

        let res = client.get("/test").send().await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }
}
