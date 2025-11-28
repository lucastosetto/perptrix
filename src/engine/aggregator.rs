use crate::engine::signal::{MarketBias, RiskLevel, ScoreBreakdown, TradingSignal};
use crate::indicators::momentum::{macd, rsi};
use crate::indicators::perp::{funding_rate, open_interest};
use crate::indicators::trend::{ema, supertrend};
use crate::indicators::volatility::{atr, bollinger};
use crate::indicators::volume::{obv, volume_profile};

pub struct SignalAggregator;

impl SignalAggregator {
    pub fn new() -> Self {
        Self
    }

    pub fn aggregate(&self, signals: IndicatorSignals) -> TradingSignal {
        let mut reasons = Vec::new();

        let trend_score = self.score_trend(&signals, &mut reasons);
        let momentum_score = self.score_momentum(&signals, &mut reasons);
        let volatility_score = self.score_volatility(&signals, &mut reasons);
        let volume_score = self.score_volume(&signals, &mut reasons);
        let perp_score = self.score_perp(&signals, &mut reasons);

        let total_score =
            trend_score + momentum_score + volatility_score + volume_score + perp_score;
        let bias = MarketBias::from_score(total_score);
        let position = bias.to_position();
        let confidence =
            self.calculate_confidence(trend_score, momentum_score, volume_score, perp_score);
        let risk_level = self.assess_risk(&signals, total_score);

        TradingSignal {
            position,
            confidence,
            bias,
            score_breakdown: ScoreBreakdown {
                trend_score,
                momentum_score,
                volatility_score,
                volume_score,
                perp_score,
                total_score,
            },
            risk_level,
            reasons,
        }
    }

    fn score_trend(&self, signals: &IndicatorSignals, reasons: &mut Vec<String>) -> i32 {
        let mut score = 0;
        match signals.ema_signal {
            ema::EMATrendSignal::BullishCross => {
                score += 2;
                reasons.push("Golden Cross EMA20/50".into());
            }
            ema::EMATrendSignal::BearishCross => {
                score -= 2;
                reasons.push("Death Cross EMA20/50".into());
            }
            ema::EMATrendSignal::StrongUptrend => {
                score += 1;
                reasons.push("Strong uptrend structure".into());
            }
            ema::EMATrendSignal::StrongDowntrend => {
                score -= 1;
                reasons.push("Strong downtrend structure".into());
            }
            _ => {}
        }

        match signals.supertrend_signal {
            supertrend::SuperTrendSignal::BullishFlip => {
                score += 2;
                reasons.push("SuperTrend flip bullish".into());
            }
            supertrend::SuperTrendSignal::BearishFlip => {
                score -= 2;
                reasons.push("SuperTrend flip bearish".into());
            }
            supertrend::SuperTrendSignal::Bullish => score += 1,
            supertrend::SuperTrendSignal::Bearish => score -= 1,
        }

        score.clamp(-3, 3)
    }

    fn score_momentum(&self, signals: &IndicatorSignals, reasons: &mut Vec<String>) -> i32 {
        let mut score = 0;

        match signals.rsi_signal {
            rsi::RSISignal::BullishDivergence => {
                score += 2;
                reasons.push("RSI bullish divergence".into());
            }
            rsi::RSISignal::BearishDivergence => {
                score -= 2;
                reasons.push("RSI bearish divergence".into());
            }
            rsi::RSISignal::Oversold => {
                score += 1;
                reasons.push("RSI oversold".into());
            }
            rsi::RSISignal::Overbought => {
                score -= 1;
                reasons.push("RSI overbought".into());
            }
            _ => {}
        }

        match signals.macd_signal {
            macd::MACDSignal::BullishCross => {
                score += 2;
                reasons.push("MACD bullish cross".into());
            }
            macd::MACDSignal::BearishCross => {
                score -= 2;
                reasons.push("MACD bearish cross".into());
            }
            macd::MACDSignal::BullishMomentum => score += 1,
            macd::MACDSignal::BearishMomentum => score -= 1,
            _ => {}
        }

        score.clamp(-3, 3)
    }

    fn score_volatility(&self, signals: &IndicatorSignals, reasons: &mut Vec<String>) -> i32 {
        let mut score = 0;

        match signals.bollinger_signal {
            bollinger::BollingerSignal::Squeeze => {
                reasons.push("Bollinger Squeeze - breakout setup".into());
            }
            bollinger::BollingerSignal::UpperBreakout => {
                score += 1;
                reasons.push("Price broke above Bollinger upper".into());
            }
            bollinger::BollingerSignal::LowerBreakout => {
                score -= 1;
                reasons.push("Price broke below Bollinger lower".into());
            }
            bollinger::BollingerSignal::MeanReversion => {
                reasons.push("Bollinger mean reversion".into());
            }
            bollinger::BollingerSignal::WalkingBands => {
                reasons.push("Walking the bands - strong trend".into());
            }
            _ => {}
        }

        match signals.volatility_regime {
            atr::VolatilityRegime::High => {
                reasons.push("High volatility - reduce size".into());
            }
            atr::VolatilityRegime::Low => {
                reasons.push("Low volatility - breakout potential".into());
            }
            atr::VolatilityRegime::Elevated => {
                reasons.push("Elevated volatility".into());
            }
            atr::VolatilityRegime::Normal => {}
        }

        score.clamp(-2, 2)
    }

    fn score_volume(&self, signals: &IndicatorSignals, reasons: &mut Vec<String>) -> i32 {
        let mut score = 0;

        match signals.obv_signal {
            obv::OBVSignal::BullishDivergence => {
                score += 2;
                reasons.push("OBV bullish divergence".into());
            }
            obv::OBVSignal::BearishDivergence => {
                score -= 2;
                reasons.push("OBV bearish divergence".into());
            }
            obv::OBVSignal::Confirmation => {
                score += 1;
                reasons.push("Volume confirms price action".into());
            }
            _ => {}
        }

        match signals.volume_profile_signal {
            volume_profile::VolumeProfileSignal::POCSupport => {
                score += 1;
                reasons.push("Price at POC support".into());
            }
            volume_profile::VolumeProfileSignal::POCResistance => {
                score -= 1;
                reasons.push("Price at POC resistance".into());
            }
            volume_profile::VolumeProfileSignal::NearLVN => {
                reasons.push("Near LVN - expect fast move".into());
            }
            _ => {}
        }

        score.clamp(-2, 2)
    }

    fn score_perp(&self, signals: &IndicatorSignals, reasons: &mut Vec<String>) -> i32 {
        let mut score = 0;

        match signals.oi_signal {
            open_interest::OpenInterestSignal::BullishExpansion => {
                score += 2;
                reasons.push("New money entering longs".into());
            }
            open_interest::OpenInterestSignal::BearishExpansion => {
                score -= 2;
                reasons.push("New money entering shorts".into());
            }
            open_interest::OpenInterestSignal::ShortSqueeze => {
                score += 1;
                reasons.push("Potential short squeeze".into());
            }
            open_interest::OpenInterestSignal::LongSqueeze => {
                score -= 1;
                reasons.push("Long squeeze in progress".into());
            }
            _ => {}
        }

        match signals.funding_signal {
            funding_rate::FundingSignal::ExtremeLongBias => {
                score -= 1;
                reasons.push("Extreme long bias - caution".into());
            }
            funding_rate::FundingSignal::ExtremShortBias => {
                score += 1;
                reasons.push("Extreme short bias - bounce potential".into());
            }
            _ => {}
        }

        score.clamp(-2, 2)
    }

    fn calculate_confidence(&self, trend: i32, momentum: i32, volume: i32, perp: i32) -> f64 {
        let scores = vec![trend, momentum, volume, perp];
        
        // Calculate alignment based on magnitude, not just count
        // Maximum possible scores: trend=±3, momentum=±3, volume=±2, perp=±2
        let max_possible_scores = vec![3, 3, 2, 2];
        
        // Calculate total positive and negative magnitudes
        let mut positive_magnitude = 0.0;
        let mut negative_magnitude = 0.0;
        let mut total_possible = 0.0;
        
        for (score, &max_score) in scores.iter().zip(max_possible_scores.iter()) {
            total_possible += max_score as f64;
            if *score > 0 {
                positive_magnitude += score.abs() as f64;
            } else if *score < 0 {
                negative_magnitude += score.abs() as f64;
            }
        }
        
        if total_possible == 0.0 {
            return 0.0;
        }
        
        // Base confidence is the proportion of maximum possible alignment
        let alignment = positive_magnitude.max(negative_magnitude);
        let mut base_confidence = alignment / total_possible;
        
        // Apply trend-momentum alignment bonus/penalty
        let trend_momentum_aligned = (trend > 0 && momentum > 0) || (trend < 0 && momentum < 0);
        if trend_momentum_aligned {
            base_confidence = (base_confidence * 1.2).min(1.0);
        } else {
            base_confidence *= 0.8;
        }
        
        // Ensure confidence is in valid range [0.0, 1.0]
        base_confidence.max(0.0).min(1.0)
    }

    fn assess_risk(&self, signals: &IndicatorSignals, total_score: i32) -> RiskLevel {
        let mut risk_factors: i32 = 0;

        if matches!(signals.volatility_regime, atr::VolatilityRegime::High) {
            risk_factors += 2;
        }

        if matches!(
            signals.funding_signal,
            funding_rate::FundingSignal::ExtremeLongBias
                | funding_rate::FundingSignal::ExtremShortBias
        ) {
            risk_factors += 1;
        }

        if total_score.abs() < 2 {
            risk_factors += 1;
        }

        if matches!(
            signals.rsi_signal,
            rsi::RSISignal::BullishDivergence | rsi::RSISignal::BearishDivergence
        ) {
            risk_factors = risk_factors.saturating_sub(1);
        }

        match risk_factors {
            r if r >= 3 => RiskLevel::High,
            r if r >= 1 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndicatorSignals {
    pub ema_signal: ema::EMATrendSignal,
    pub supertrend_signal: supertrend::SuperTrendSignal,
    pub rsi_signal: rsi::RSISignal,
    pub macd_signal: macd::MACDSignal,
    pub bollinger_signal: bollinger::BollingerSignal,
    pub volatility_regime: atr::VolatilityRegime,
    pub obv_signal: obv::OBVSignal,
    pub volume_profile_signal: volume_profile::VolumeProfileSignal,
    pub oi_signal: open_interest::OpenInterestSignal,
    pub funding_signal: funding_rate::FundingSignal,
}
