use std::net::SocketAddr;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use axum_prometheus::PrometheusMetricLayer;
use serde_json::json;
use tracing::info;

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"message": "ok"})))
}

pub async fn metrics(addr: SocketAddr) -> Result<(), anyhow::Error> {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let router = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(prometheus_layer);

    info!("starting http server: {addr}");

    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
