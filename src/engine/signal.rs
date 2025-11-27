#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MarketBias {
    StrongBullish,
    Bullish,
    Neutral,
    Bearish,
    StrongBearish,
}

impl MarketBias {
    pub fn from_score(score: i32) -> Self {
        match score {
            s if s >= 7 => MarketBias::StrongBullish,
            s if s >= 3 => MarketBias::Bullish,
            s if s > -3 => MarketBias::Neutral,
            s if s > -7 => MarketBias::Bearish,
            _ => MarketBias::StrongBearish,
        }
    }

    pub fn to_position(&self) -> Position {
        match self {
            MarketBias::StrongBullish | MarketBias::Bullish => Position::Long,
            MarketBias::Neutral => Position::Neutral,
            MarketBias::Bearish | MarketBias::StrongBearish => Position::Short,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Long,
    Short,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    pub trend_score: i32,
    pub momentum_score: i32,
    pub volatility_score: i32,
    pub volume_score: i32,
    pub perp_score: i32,
    pub total_score: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct TradingSignal {
    pub position: Position,
    pub confidence: f64,
    pub bias: MarketBias,
    pub score_breakdown: ScoreBreakdown,
    pub risk_level: RiskLevel,
    pub reasons: Vec<String>,
}
