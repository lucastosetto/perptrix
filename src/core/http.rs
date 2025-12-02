//! HTTP endpoint server using Axum

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info, Level};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::db::QuestDatabase;
use crate::metrics::Metrics;
use crate::models::strategy::{Strategy, StrategyConfig};

#[derive(Clone)]
pub struct AppState {
    pub health: Arc<RwLock<HealthStatus>>,
    pub metrics: Arc<Metrics>,
    pub start_time: Arc<Instant>,
    pub database: Option<Arc<QuestDatabase>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthStatus {
    pub status: String,
}

#[derive(Serialize, Deserialize, utoipa::ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub service: String,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
        }
    }
}

/// Health check endpoint
///
/// Returns the health status and uptime of the service
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, StatusCode> {
    let health = state.health.read().await;
    let uptime_seconds = state.start_time.elapsed().as_secs();
    Ok(Json(HealthResponse {
        status: health.status.clone(),
        uptime_seconds,
        service: "perptrix-signal-engine".to_string(),
    }))
}

/// Prometheus metrics endpoint
///
/// Returns metrics in Prometheus format
#[utoipa::path(
    get,
    path = "/metrics",
    tag = "Metrics",
    responses(
        (status = 200, description = "Metrics in Prometheus format", content_type = "text/plain")
    )
)]
pub async fn metrics_handler(State(state): State<AppState>) -> Result<String, StatusCode> {
    state
        .metrics
        .export()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}


#[derive(Debug, Deserialize, utoipa::IntoParams, utoipa::ToSchema)]
#[into_params(parameter_in = Query)]
struct StrategyQuery {
    /// Filter strategies by symbol
    symbol: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
struct CreateStrategyRequest {
    /// Strategy name
    name: String,
    /// Trading symbol (e.g., "BTC-USD")
    symbol: String,
    /// Strategy configuration
    config: StrategyConfig,
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema)]
struct UpdateStrategyRequest {
    /// Strategy name (optional)
    name: Option<String>,
    /// Trading symbol (optional)
    symbol: Option<String>,
    /// Strategy configuration (optional)
    config: Option<StrategyConfig>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
struct StrategyResponse {
    /// Strategy ID
    id: i64,
    /// Strategy name
    name: String,
    /// Trading symbol
    symbol: String,
    /// Strategy configuration
    config: StrategyConfig,
    /// Creation timestamp
    created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Strategy> for StrategyResponse {
    fn from(strategy: Strategy) -> Self {
        Self {
            id: strategy.id.unwrap_or(0),
            name: strategy.name,
            symbol: strategy.symbol,
            config: strategy.config,
            created_at: strategy.created_at,
            updated_at: strategy.updated_at,
        }
    }
}

/// List all strategies, optionally filtered by symbol
#[utoipa::path(
    get,
    path = "/api/strategies",
    tag = "Strategies",
    params(StrategyQuery),
    responses(
        (status = 200, description = "List of strategies", body = Vec<StrategyResponse>),
        (status = 503, description = "Database unavailable")
    )
)]
async fn list_strategies(
    State(state): State<AppState>,
    Query(params): Query<StrategyQuery>,
) -> Result<Json<Vec<StrategyResponse>>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let strategies = db
        .get_strategies(params.symbol.as_deref())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to load strategies");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let responses: Vec<StrategyResponse> = strategies.into_iter().map(Into::into).collect();
    Ok(Json(responses))
}

/// Get a strategy by ID
#[utoipa::path(
    get,
    path = "/api/strategies/{id}",
    tag = "Strategies",
    params(
        ("id" = i64, Path, description = "Strategy ID")
    ),
    responses(
        (status = 200, description = "Strategy found", body = StrategyResponse),
        (status = 404, description = "Strategy not found"),
        (status = 503, description = "Database unavailable")
    )
)]
async fn get_strategy(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<StrategyResponse>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let strategy = db.get_strategy(id).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to load strategy");
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(strategy.into()))
}

/// Create a new strategy
#[utoipa::path(
    post,
    path = "/api/strategies",
    tag = "Strategies",
    request_body = CreateStrategyRequest,
    responses(
        (status = 200, description = "Strategy created", body = StrategyResponse),
        (status = 503, description = "Database unavailable")
    )
)]
async fn create_strategy(
    State(state): State<AppState>,
    Json(request): Json<CreateStrategyRequest>,
) -> Result<Json<StrategyResponse>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let now = chrono::Utc::now();
    let strategy = Strategy {
        id: None,
        name: request.name,
        symbol: request.symbol,
        config: request.config,
        created_at: now,
        updated_at: now,
    };

    let id = db.create_strategy(&strategy).await.map_err(|e| {
        error!(error = %e, "Failed to create strategy");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let created_strategy = db.get_strategy(id).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to load created strategy");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(created_strategy.into()))
}

/// Update a strategy
#[utoipa::path(
    put,
    path = "/api/strategies/{id}",
    tag = "Strategies",
    params(
        ("id" = i64, Path, description = "Strategy ID")
    ),
    request_body = UpdateStrategyRequest,
    responses(
        (status = 200, description = "Strategy updated", body = StrategyResponse),
        (status = 404, description = "Strategy not found"),
        (status = 503, description = "Database unavailable")
    )
)]
async fn update_strategy(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(request): Json<UpdateStrategyRequest>,
) -> Result<Json<StrategyResponse>, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let mut strategy = db.get_strategy(id).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to load strategy");
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Update fields if provided
    if let Some(name) = request.name {
        strategy.name = name;
    }
    if let Some(symbol) = request.symbol {
        strategy.symbol = symbol;
    }
    if let Some(config) = request.config {
        strategy.config = config;
    }
    strategy.updated_at = chrono::Utc::now();

    db.update_strategy(id, &strategy).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to update strategy");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(strategy.into()))
}

/// Delete a strategy
#[utoipa::path(
    delete,
    path = "/api/strategies/{id}",
    tag = "Strategies",
    params(
        ("id" = i64, Path, description = "Strategy ID")
    ),
    responses(
        (status = 204, description = "Strategy deleted"),
        (status = 404, description = "Strategy not found"),
        (status = 503, description = "Database unavailable")
    )
)]
async fn delete_strategy(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let db = state
        .database
        .as_ref()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    db.delete_strategy(id).await.map_err(|e| {
        error!(error = %e, strategy_id = id, "Failed to delete strategy");
        if e.to_string().contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        metrics_handler,
        list_strategies,
        get_strategy,
        create_strategy,
        update_strategy,
        delete_strategy
    ),
    components(schemas(
        HealthResponse,
        StrategyResponse,
        CreateStrategyRequest,
        UpdateStrategyRequest,
        StrategyConfig,
        StrategyQuery,
        crate::models::strategy::Rule,
        crate::models::strategy::RuleType,
        crate::models::strategy::Condition,
        crate::models::strategy::IndicatorType,
        crate::models::strategy::Comparison,
        crate::models::strategy::LogicalOperator,
        crate::models::strategy::AggregationConfig,
        crate::models::strategy::AggregationMethod,
        crate::models::strategy::SignalThresholds
    )),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Metrics", description = "Metrics endpoints"),
        (name = "Strategies", description = "Strategy management endpoints")
    ),
    info(
        title = "Perptrix API",
        description = "API for the Perptrix signal engine - a trading strategy evaluation system",
        version = "0.1.0"
    )
)]
struct ApiDoc;

/// Middleware to track HTTP request metrics
async fn metrics_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // Skip metrics tracking for the /metrics endpoint to avoid potential recursion issues
    let path = request.uri().path();
    if path == "/metrics" {
        return next.run(request).await;
    }

    let start = Instant::now();
    let method = request.method().clone();
    let path = path.to_string();

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
        .merge(
            SwaggerUi::new("/docs")
                .url("/docs/openapi.json", ApiDoc::openapi())
        )
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        .route("/api/strategies", get(list_strategies))
        .route("/api/strategies", post(create_strategy))
        .route("/api/strategies/{id}", get(get_strategy))
        .route("/api/strategies/{id}", put(update_strategy))
        .route("/api/strategies/{id}", delete(delete_strategy))
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
    
    // Initialize database connection (optional - API works without it but strategy endpoints won't)
    let database = match crate::db::QuestDatabase::new().await {
        Ok(db) => {
            info!("QuestDB connected for API server");
            Some(Arc::new(db))
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to connect to QuestDB for API server - strategy endpoints will be unavailable");
            None
        }
    };
    
    let state = AppState {
        health: Arc::new(RwLock::new(HealthStatus::default())),
        metrics: metrics.clone(),
        start_time: start_time.clone(),
        database,
    };
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!(port = port, "HTTP server listening on port {}", port);
    info!(
        "Metrics endpoint available at http://0.0.0.0:{}/metrics",
        port
    );
    info!(
        "API documentation available at http://0.0.0.0:{}/docs",
        port
    );
    axum::serve(listener, app).await?;

    Ok(())
}
