//! Prometheus metrics for Perptrix signal engine
//!
//! Provides metrics for HTTP requests, signal evaluations, and system health.

use prometheus::{
    register_counter_with_registry, register_gauge_with_registry, register_histogram_with_registry,
    Counter, Gauge, Histogram, Registry, TextEncoder,
};
use std::sync::Arc;

/// Metrics container for all application metrics
#[derive(Clone)]
pub struct Metrics {
    pub registry: Arc<Registry>,

    // HTTP metrics
    pub http_requests_total: Counter,
    pub http_request_duration_seconds: Histogram,
    pub http_requests_in_flight: Gauge,

    // Signal evaluation metrics
    pub signal_evaluations_total: Counter,
    pub signal_evaluation_duration_seconds: Histogram,
    pub signal_evaluations_active: Gauge,
    pub signal_evaluation_errors_total: Counter,

    // System health metrics
    pub database_connected: Gauge,
    pub cache_connected: Gauge,
    pub websocket_connected: Gauge,
}

impl Metrics {
    /// Create a new metrics instance with all metrics registered
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        // HTTP metrics
        let http_requests_total = register_counter_with_registry!(
            "http_requests_total",
            "Total number of HTTP requests",
            &registry
        )?;

        let http_request_duration_seconds = register_histogram_with_registry!(
            "http_request_duration_seconds",
            "HTTP request latency in seconds",
            &registry
        )?;

        let http_requests_in_flight = register_gauge_with_registry!(
            "http_requests_in_flight",
            "Number of HTTP requests currently being processed",
            &registry
        )?;

        // Signal evaluation metrics
        let signal_evaluations_total = register_counter_with_registry!(
            "signal_evaluations_total",
            "Total number of signal evaluations",
            &registry
        )?;

        let signal_evaluation_duration_seconds = register_histogram_with_registry!(
            "signal_evaluation_duration_seconds",
            "Signal evaluation latency in seconds",
            &registry
        )?;

        let signal_evaluations_active = register_gauge_with_registry!(
            "signal_evaluations_active",
            "Number of signal evaluations currently in progress",
            &registry
        )?;

        let signal_evaluation_errors_total = register_counter_with_registry!(
            "signal_evaluation_errors_total",
            "Total number of signal evaluation errors",
            &registry
        )?;

        // System health metrics
        let database_connected = register_gauge_with_registry!(
            "database_connected",
            "Database connection status (1 = connected, 0 = disconnected)",
            &registry
        )?;

        let cache_connected = register_gauge_with_registry!(
            "cache_connected",
            "Cache connection status (1 = connected, 0 = disconnected)",
            &registry
        )?;

        let websocket_connected = register_gauge_with_registry!(
            "websocket_connected",
            "WebSocket connection status (1 = connected, 0 = disconnected)",
            &registry
        )?;

        Ok(Self {
            registry: Arc::new(registry),
            http_requests_total,
            http_request_duration_seconds,
            http_requests_in_flight,
            signal_evaluations_total,
            signal_evaluation_duration_seconds,
            signal_evaluations_active,
            signal_evaluation_errors_total,
            database_connected,
            cache_connected,
            websocket_connected,
        })
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> Result<String, prometheus::Error> {
        // Use the registry directly to gather metrics
        // This avoids potential issues with cloning or circular references
        let metric_families = self.registry.gather();
        let encoder = TextEncoder::new();
        encoder.encode_to_string(&metric_families)
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics")
    }
}
