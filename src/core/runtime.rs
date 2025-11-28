//! Periodic task runner for continuous signal evaluation

use crate::metrics::Metrics;
use crate::services::market_data::{MarketDataProvider, PlaceholderMarketDataProvider};
use crate::signals::engine::SignalEngine;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

pub struct RuntimeConfig {
    pub evaluation_interval_seconds: u64,
    pub symbols: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            evaluation_interval_seconds: 60,
            symbols: vec!["BTC-PERP".to_string()],
        }
    }
}

pub struct SignalRuntime {
    config: RuntimeConfig,
    data_provider: Arc<dyn MarketDataProvider + Send + Sync>,
    database: Option<Arc<crate::db::QuestDatabase>>,
    metrics: Option<Arc<Metrics>>,
}

impl SignalRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            data_provider: Arc::new(PlaceholderMarketDataProvider),
            database: None,
            metrics: None,
        }
    }

    pub fn with_provider<P: MarketDataProvider + Send + Sync + 'static>(
        config: RuntimeConfig,
        provider: P,
    ) -> Self {
        Self {
            config,
            data_provider: Arc::new(provider),
            database: None,
            metrics: None,
        }
    }

    pub fn with_database(mut self, database: Arc<crate::db::QuestDatabase>) -> Self {
        self.database = Some(database);
        self
    }

    pub fn with_metrics(mut self, metrics: Arc<Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval_timer =
            interval(Duration::from_secs(self.config.evaluation_interval_seconds));

        info!(
            interval = self.config.evaluation_interval_seconds,
            "Signal runtime started. Evaluating signals every {} seconds",
            self.config.evaluation_interval_seconds
        );

        loop {
            interval_timer.tick().await;

            for symbol in &self.config.symbols {
                match self.evaluate_signal(symbol).await {
                    Ok(Some(signal)) => {
                        // Log at different levels based on signal strength
                        let confidence_pct = (signal.confidence * 10000.0).round() / 100.0;
                        if signal.direction == crate::models::signal::SignalDirection::Neutral {
                            debug!(
                                symbol = %symbol,
                                direction = ?signal.direction,
                                confidence = confidence_pct,
                                "Signal for {}: {:?} (confidence: {:.2}%)",
                                symbol,
                                signal.direction,
                                confidence_pct
                            );
                        } else {
                            info!(
                                symbol = %symbol,
                                direction = ?signal.direction,
                                confidence = confidence_pct,
                                "Signal for {}: {:?} (confidence: {:.2}%)",
                                symbol,
                                signal.direction,
                                confidence_pct
                            );
                        }

                        // Record successful evaluation
                        if let Some(ref metrics) = self.metrics {
                            metrics.signal_evaluations_total.inc();
                        }

                        // Store signal in database if available
                        if let Some(ref db) = self.database {
                            if let Err(e) = db.store_signal(&signal).await {
                                error!(symbol = %symbol, error = %e, "Failed to store signal in database");
                            }
                        }
                    }
                    Ok(None) => {
                        debug!(symbol = %symbol, "No signal generated for {}", symbol);
                        // Still count as evaluation (no signal is a valid result)
                        if let Some(ref metrics) = self.metrics {
                            metrics.signal_evaluations_total.inc();
                        }
                    }
                    Err(e) => {
                        error!(symbol = %symbol, error = %e, "Error evaluating signal for {}", symbol);
                        // Record error
                        if let Some(ref metrics) = self.metrics {
                            metrics.signal_evaluation_errors_total.inc();
                        }
                    }
                }
            }
        }
    }

    /// Evaluate signal for a symbol
    async fn evaluate_signal(
        &self,
        symbol: &str,
    ) -> Result<Option<crate::models::signal::SignalOutput>, Box<dyn std::error::Error + Send + Sync>>
    {
        let start = Instant::now();

        // Track active evaluation
        if let Some(ref metrics) = self.metrics {
            metrics.signal_evaluations_active.inc();
        }

        let result = self.evaluate_signal_internal(symbol).await;

        // Record duration and decrement active
        if let Some(ref metrics) = self.metrics {
            let duration = start.elapsed();
            metrics
                .signal_evaluation_duration_seconds
                .observe(duration.as_secs_f64());
            metrics.signal_evaluations_active.dec();
        }

        result
    }

    async fn evaluate_signal_internal(
        &self,
        symbol: &str,
    ) -> Result<Option<crate::models::signal::SignalOutput>, Box<dyn std::error::Error + Send + Sync>>
    {
        let candles = self
            .data_provider
            .get_candles(symbol, 250)
            .await
            .map_err(|e| {
                Box::new(std::io::Error::other(format!("Market data error: {}", e)))
                    as Box<dyn std::error::Error + Send + Sync>
            })?;

        if candles.is_empty() {
            debug!(symbol = %symbol, "No candles available yet - waiting for WebSocket data");
            return Ok(None);
        }

        debug!(
            symbol = %symbol,
            candle_count = candles.len(),
            min_candles = crate::signals::engine::MIN_CANDLES,
            "Evaluating with {} candles (need {})",
            candles.len(),
            crate::signals::engine::MIN_CANDLES
        );

        if candles.len() < crate::signals::engine::MIN_CANDLES {
            debug!(
                symbol = %symbol,
                candle_count = candles.len(),
                min_candles = crate::signals::engine::MIN_CANDLES,
                "Not enough candles ({} < {}) - waiting for more candles to accumulate (1m candles arrive every minute)",
                candles.len(),
                crate::signals::engine::MIN_CANDLES
            );
            return Ok(None);
        }

        let signal = SignalEngine::evaluate(&candles, symbol);

        if signal.is_none() {
            debug!(symbol = %symbol, "Signal evaluation returned None (likely insufficient data or neutral score)");
        } else if let Some(ref sig) = signal {
            let confidence_pct = (sig.confidence * 10000.0).round() / 100.0;
            info!(
                symbol = %symbol,
                direction = ?sig.direction,
                confidence = confidence_pct,
                reasons = ?sig.reasons,
                "Signal generated - Direction: {:?}, Confidence: {:.2}%, Reasons: {:?}",
                sig.direction,
                confidence_pct,
                sig.reasons
            );
        }

        Ok(signal)
    }
}
