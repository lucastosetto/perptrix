//! Funding rate bias detector for perpetual swaps.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FundingSignal {
    ExtremeLongBias,
    ExtremShortBias,
    HighLongBias,
    HighShortBias,
    NeutralPositive,
    NeutralNegative,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct FundingRate {
    funding_history: Vec<f64>,
    lookback: usize,
}

impl FundingRate {
    pub fn new(lookback: usize) -> Self {
        Self {
            funding_history: Vec::new(),
            lookback,
        }
    }

    pub fn update(&mut self, funding_rate: f64) -> (FundingSignal, f64) {
        self.funding_history.push(funding_rate);
        if self.funding_history.len() > self.lookback {
            self.funding_history.remove(0);
        }

        let avg_funding = if self.funding_history.is_empty() {
            0.0
        } else {
            self.funding_history.iter().sum::<f64>() / self.funding_history.len() as f64
        };

        let signal = if funding_rate > 0.001 {
            FundingSignal::ExtremeLongBias
        } else if funding_rate < -0.001 {
            FundingSignal::ExtremShortBias
        } else if funding_rate > 0.0005 {
            FundingSignal::HighLongBias
        } else if funding_rate < -0.0005 {
            FundingSignal::HighShortBias
        } else if funding_rate > 0.0 {
            FundingSignal::NeutralPositive
        } else if funding_rate < 0.0 {
            FundingSignal::NeutralNegative
        } else {
            FundingSignal::Neutral
        };

        (signal, avg_funding)
    }
}
