use std::{future::ready, time::Instant};

use async_trait::async_trait;
use axum::{
    extract::MatchedPath, extract::Request, middleware::Next, response::Response, routing::get,
    Router,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use tokio::net::TcpListener;

/// based on axum/examples/prometheus-metrics/src/main.rs
/// ```rust,no_run
/// use axum::{Router, ServiceExt, routing::get, middleware};
/// use axum_restful::utils::{PrometheusMetrics, track_metrics};
///
/// struct Metrics;
/// impl PrometheusMetrics for Metrics {}
/// tokio::spawn(async {
///     Metrics::start_metrics_server().await;
/// });
///
/// let app = Router::new()
///     .route("/hello", get(|| async {"hello"}))
///     .route_layer(middleware::from_fn(track_metrics));
/// # async {
///     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
///     axum::serve(listener, app).await.unwrap();
/// # };
/// ```
/// and then you can visit http://127.0.0.1:3000/hello to get response from axum server,
/// next the visis metrics is recorded in prometheus metrics,
/// the default prometheus metrics is http://0.0.0.0:3001/metrics.
/// ip and port can modified by [`PrometheusMetrics::get_metrics_addr`]
/// url path can modified by [`PrometheusMetrics::get_metrics_path`]
#[async_trait]
pub trait PrometheusMetrics {
    fn get_exponential_seconds() -> Vec<f64> {
        vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ]
    }

    fn get_prometheus_handle() -> PrometheusHandle {
        PrometheusBuilder::new()
            .set_buckets_for_metric(
                Matcher::Full("http_requests_duration_seconds".to_string()),
                &Self::get_exponential_seconds(),
            )
            .unwrap()
            .install_recorder()
            .unwrap()
    }

    fn get_prometheus_app() -> Router {
        let recorder_handle = Self::get_prometheus_handle();
        Router::new().route(
            Self::get_metrics_path(),
            get(move || ready(recorder_handle.render())),
        )
    }

    fn get_metrics_path() -> &'static str {
        "/metrics"
    }

    fn get_metrics_addr() -> String {
        "0.0.0.0:3001".to_owned()
    }

    async fn start_metrics_server() {
        let addr = Self::get_metrics_addr();
        tracing::debug!("listening on {:?}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, Self::get_prometheus_app().into_make_service())
            .await
            .unwrap();
    }
}

/// a middle record the request info by added into axum middlewares
pub async fn track_metrics(req: Request, next: Next) -> Response {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();
    let response = next.run(req).await;
    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();
    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];
    metrics::increment_counter!("http_requests_total", &labels);
    metrics::histogram!("http_requests_duration_seconds", latency, &labels);
    response
}
