//! Strategy builder system data models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use serde_json::Value;
use utoipa::ToSchema;

/// Main strategy entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Option<i64>,
    pub name: String,
    pub symbol: String,
    pub config: StrategyConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Main strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StrategyConfig {
    /// List of rules to evaluate
    pub rules: Vec<Rule>,
    /// Aggregation configuration
    pub aggregation: AggregationConfig,
}

/// Individual condition or group
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Rule {
    /// Rule identifier
    pub id: String,
    #[serde(rename = "type")]
    /// Rule type (Condition, Group, or WeightedGroup)
    pub rule_type: RuleType,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional weight for weighted aggregation
    pub weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Logical operator for groups (AND/OR)
    pub operator: Option<LogicalOperator>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Condition for Condition-type rules
    pub condition: Option<Condition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Child rules for Group-type rules (recursive structure)
    #[schema(no_recursion)]
    pub children: Option<Vec<Rule>>,
}

/// Rule type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum RuleType {
    Condition,
    Group,
    WeightedGroup,
}

/// Indicator comparison condition
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Condition {
    /// Indicator type to evaluate
    pub indicator: IndicatorType,
    #[serde(default)]
    /// Optional indicator-specific parameters
    pub indicator_params: HashMap<String, Value>,
    /// Comparison operation
    pub comparison: Comparison,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Threshold value for numeric comparisons
    pub threshold: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Indicator-specific signal state (e.g., "Oversold", "BullishCross")
    pub signal_state: Option<String>,
}

/// Available indicator types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum IndicatorType {
    MACD,
    RSI,
    EMA,
    SuperTrend,
    Bollinger,
    ATR,
    OBV,
    VolumeProfile,
    FundingRate,
    OpenInterest,
}

/// Comparison operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum Comparison {
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    Equal,
    NotEqual,
    InRange,
    SignalState,
}

/// Logical operators for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogicalOperator {
    AND,
    OR,
}

/// How to combine rule results
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AggregationConfig {
    /// Aggregation method
    pub method: AggregationMethod,
    /// Signal thresholds
    pub thresholds: SignalThresholds,
}

/// Aggregation methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum AggregationMethod {
    Sum,
    WeightedSum,
    Majority,
    All,
    Any,
}

/// Score thresholds for signal generation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignalThresholds {
    /// Minimum score for long signal
    pub long_min: i32,
    /// Maximum score for short signal
    pub short_max: i32,
}

/// Result of evaluating a rule
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub rule_id: String,
    pub passed: bool,
    pub score: i32,
    pub weight: f64,
}

impl RuleResult {
    pub fn new(rule_id: String, passed: bool, score: i32, weight: f64) -> Self {
        Self {
            rule_id,
            passed,
            score,
            weight,
        }
    }
}

