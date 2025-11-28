//! Logging initialization with environment-based formatters
//!
//! - Production: Structured JSON logs for cloud monitoring
//! - Sandbox: Colorful, human-readable logs for development
//!
//! Also initializes OpenTelemetry tracing that exports to Grafana Tempo via OTLP.

use crate::config::get_environment;
use opentelemetry::global;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use std::env;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// Initialize OpenTelemetry tracing that exports to Tempo via OTLP
fn init_tracing() -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
    // Get configuration from environment
    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4318".to_string());
    let service_name =
        env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "perptrix-signal-engine".to_string());

    // Create resource with service name
    let resource = Resource::new(vec![SERVICE_NAME.string(service_name.clone())]);

    // Create OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .http()
        .with_endpoint(otlp_endpoint);

    // Create tracer provider using pipeline
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace::config().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    // Set global propagator
    global::set_text_map_propagator(TraceContextPropagator::new());

    Ok(tracer)
}

/// Initialize logging based on the environment
///
/// - Production: JSON structured logs (suitable for log aggregation systems)
/// - Sandbox/Development: Colorful, human-readable logs
///
/// Also initializes OpenTelemetry tracing if OTEL_EXPORTER_OTLP_ENDPOINT is set.
pub fn init_logging() {
    let env = get_environment();
    // Build filter from environment or default to "info"
    // Suppress verbose PostgreSQL debug logs by setting tokio_postgres to warn level
    let mut env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Add directive to suppress tokio_postgres debug logs
    if let Ok(directive) = "tokio_postgres=warn".parse() {
        env_filter = env_filter.add_directive(directive);
    }

    let is_production = matches!(env.as_str(), "production" | "prod");

    // Try to initialize OpenTelemetry tracing
    // If it fails (e.g., Tempo not available), continue without tracing
    let otel_layer = match init_tracing() {
        Ok(_tracer) => {
            // Note: We can't log here because tracing isn't initialized yet
            // The tracer is set globally, so tracing_opentelemetry will use it
            Some(tracing_opentelemetry::layer())
        }
        Err(_e) => {
            // Can't log error here since tracing isn't initialized
            None
        }
    };

    if let Some(otel) = otel_layer {
        if is_production {
            Registry::default()
                .with(env_filter.clone())
                .with(otel)
                .with(
                    fmt::layer()
                        .json()
                        .with_target(true)
                        .with_file(true)
                        .with_line_number(true)
                        .with_writer(std::io::stdout),
                )
                .init();
        } else {
            Registry::default()
                .with(env_filter.clone())
                .with(otel)
                .with(
                    fmt::layer()
                        .with_target(true)
                        .with_file(true)
                        .with_line_number(true)
                        .with_ansi(true)
                        .with_writer(std::io::stdout),
                )
                .init();
        }
    } else if is_production {
        Registry::default()
            .with(env_filter.clone())
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_writer(std::io::stdout),
            )
            .init();
    } else {
        Registry::default()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(true)
                    .with_writer(std::io::stdout),
            )
            .init();
    }
}
