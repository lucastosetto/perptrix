use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Canonical signal direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalDirection {
    Long,
    Short,
    None,
}

/// Individual reason behind a generated signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalReason {
    pub description: String,
    pub weight: f64,
}

/// Output artifact produced by the signal engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalOutput {
    pub direction: SignalDirection,
    pub confidence: f64,
    pub recommended_sl_pct: f64,
    pub recommended_tp_pct: f64,
    pub reasons: Vec<SignalReason>,
    pub symbol: String,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

impl SignalOutput {
    pub fn new(
        direction: SignalDirection,
        confidence: f64,
        recommended_sl_pct: f64,
        recommended_tp_pct: f64,
        reasons: Vec<SignalReason>,
        symbol: String,
        price: f64,
    ) -> Self {
        Self {
            direction,
            confidence,
            recommended_sl_pct,
            recommended_tp_pct,
            reasons,
            symbol,
            price,
            timestamp: Utc::now(),
        }
    }
}
