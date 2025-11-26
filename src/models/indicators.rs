use serde::{Deserialize, Serialize};

/// Minimal candle/indicator snapshot placeholder until native engines arrive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorSnapshot {
    pub symbol: String,
    pub macd: f64,
    pub rsi: f64,
    pub funding_rate: Option<f64>,
    pub price: f64,
}
