use tokio::signal;

/// copy from axum/examples/graceful-shutdown/src/main.rs
/// receive Ctrl + C and graceful shutdown server
/// # graceful shutdown Example
/// ```rust,no_run
/// use axum::{Router, routing::get};
/// use axum_restful::utils::shutdown_signal;
///
/// // let app = Router::new().route("/", get(|| async {"hello"}));
/// # async {
/// //axum::Server::bind(&"".parse().unwrap())
///  //    .serve(app.into_make_service())
///  //    .with_graceful_shutdown(shutdown_signal())
///  //    .await
///  //    .unwrap()
/// # };
/// ```
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}
