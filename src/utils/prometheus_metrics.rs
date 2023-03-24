use std::{future::ready, net::SocketAddr, time::Instant};

use async_trait::async_trait;
use axum::{
    extract::MatchedPath, http::Request, middleware::Next, response::Response, routing::get, Router,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

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
/// let app = Router::new().route("/hello", get(|| async {"hello"})).route_layer(middleware::from_fn(track_metrics));
/// # async {
/// #     axum::Server::bind(&"0.0.0.0:3000".parse().unwrap()).serve(app.into_make_service()).await.unwrap()
/// # };
/// ```
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
        Router::new().route("/metrics", get(move || ready(recorder_handle.render())))
    }

    fn get_metrics_addr() -> SocketAddr {
        "0.0.0.0:3001".parse().unwrap()
    }

    async fn start_metrics_server() {
        let addr = Self::get_metrics_addr();
        tracing::debug!("listening on {:?}", addr);
        axum::Server::bind(&addr)
            .serve(Self::get_prometheus_app().into_make_service())
            .await
            .unwrap()
    }
}

pub async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> Response {
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
