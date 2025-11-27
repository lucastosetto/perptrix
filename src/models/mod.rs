//! Shared data models spanning the engine layers.

pub mod indicators;
pub mod signal;

// Re-export signal types
pub use signal::{SignalDirection, SignalEvaluation, SignalOutput, SignalReason};

// Re-export indicator types
pub use indicators::{
    EmaIndicator, IndicatorSet, MacdIndicator, RsiIndicator, SmaIndicator, VolumeIndicator,
};
