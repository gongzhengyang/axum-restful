use std::net::SocketAddr;

use axum::{Router, routing::get};

use axum_restful::utils::{GenerateCertKey, redirect_http_to_https};

#[tokio::main]
async fn main() {
    struct GenerateAppCertKey;
    impl GenerateCertKey for GenerateAppCertKey {}
    let http_port = 3000;
    let https_port = 3001;
    let ip = "0.0.0.0";
    tokio::spawn(async move {
        redirect_http_to_https(http_port, https_port, ip).await;
    });
    let app = Router::new().route("/hello", get(|| async { "Hello, world!" }));
    let tls_config = GenerateAppCertKey::get_rustls_config(true).await.unwrap();
    let addr: SocketAddr = format!("{}:{}", ip, https_port).as_str().parse().unwrap();
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
