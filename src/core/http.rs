//! HTTP endpoint server using Axum

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Json, Response},
    routing::get,
    Router,
};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{info, Level};

use crate::metrics::Metrics;

#[derive(Clone)]
pub struct AppState {
    pub health: Arc<RwLock<HealthStatus>>,
    pub metrics: Arc<Metrics>,
    pub start_time: Arc<Instant>,
}

#[derive(Clone, Debug)]
pub struct HealthStatus {
    pub status: String,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
        }
    }
}

pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let health = state.health.read().await;
    let uptime_seconds = state.start_time.elapsed().as_secs();
    Ok(Json(json!({
        "status": health.status,
        "uptime_seconds": uptime_seconds,
        "service": "perptrix-signal-engine"
    })))
}

pub async fn metrics_handler(State(state): State<AppState>) -> Result<String, StatusCode> {
    state
        .metrics
        .export()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Middleware to track HTTP request metrics
async fn metrics_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // Increment in-flight requests
    state.metrics.http_requests_in_flight.inc();

    // Process request
    let response = next.run(request).await;
    let status = response.status();
    let duration = start.elapsed();

    // Decrement in-flight requests
    state.metrics.http_requests_in_flight.dec();

    // Record metrics
    state.metrics.http_requests_total.inc();
    state
        .metrics
        .http_request_duration_seconds
        .observe(duration.as_secs_f64());

    // Log if error status
    if status.is_server_error() {
        tracing::error!(
            method = %method,
            path = %path,
            status = %status,
            duration_ms = duration.as_millis(),
            "HTTP request error"
        );
    }

    response
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::DEBUG))
                        .on_request(DefaultOnRequest::new().level(Level::DEBUG))
                        .on_response(DefaultOnResponse::new().level(Level::DEBUG)),
                )
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    metrics_middleware,
                ))
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let metrics = Arc::new(Metrics::new()?);
    let start_time = Arc::new(Instant::now());
    let state = AppState {
        health: Arc::new(RwLock::new(HealthStatus::default())),
        metrics: metrics.clone(),
        start_time: start_time.clone(),
    };
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(port = port, "HTTP server listening on port {}", port);
    info!(
        "Metrics endpoint available at http://0.0.0.0:{}/metrics",
        port
    );
    axum::serve(listener, app).await?;

    Ok(())
}
