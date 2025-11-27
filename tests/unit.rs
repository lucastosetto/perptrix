//! Unit tests - organized by module structure

#[path = "common/math.rs"]
mod common_math;

#[path = "indicators/momentum/macd.rs"]
mod indicators_momentum_macd;

#[path = "indicators/momentum/rsi.rs"]
mod indicators_momentum_rsi;

#[path = "indicators/trend/ema.rs"]
mod indicators_trend_ema;

#[path = "indicators/trend/supertrend.rs"]
mod indicators_trend_supertrend;

#[path = "indicators/volatility/bollinger.rs"]
mod indicators_volatility_bollinger;

#[path = "indicators/volatility/atr.rs"]
mod indicators_volatility_atr;

#[path = "indicators/volume/obv.rs"]
mod indicators_volume_obv;

#[path = "indicators/volume/volume_profile.rs"]
mod indicators_volume_volume_profile;

#[path = "indicators/perp/open_interest.rs"]
mod indicators_perp_open_interest;

#[path = "indicators/perp/funding_rate.rs"]
mod indicators_perp_funding_rate;

#[path = "indicators/parser.rs"]
mod indicators_parser;

#[path = "indicators/registry.rs"]
mod indicators_registry;

#[path = "signals/decision.rs"]
mod signals_decision;

#[path = "signals/engine.rs"]
mod signals_engine;

#[path = "engine/aggregator.rs"]
mod engine_aggregator;

#[path = "services/market_data.rs"]
mod services_market_data;

#[path = "core/http.rs"]
mod core_http;

#[path = "core/runtime.rs"]
mod core_runtime;
