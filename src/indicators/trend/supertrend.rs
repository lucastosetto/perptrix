//! SuperTrend indicator built atop ATR for dynamic trailing stops.

use crate::indicators::volatility::atr::ATR;

#[derive(Debug, Clone, PartialEq)]
pub enum SuperTrendSignal {
    Bullish,
    Bearish,
    BullishFlip,
    BearishFlip,
}

#[derive(Debug, Clone)]
pub struct SuperTrend {
    atr: ATR,
    multiplier: f64,
    upper_band: Option<f64>,
    lower_band: Option<f64>,
    supertrend: Option<f64>,
    prev_signal: Option<SuperTrendSignal>,
}

impl SuperTrend {
    pub fn new(period: usize, multiplier: f64) -> Self {
        Self {
            atr: ATR::new(period),
            multiplier,
            upper_band: None,
            lower_band: None,
            supertrend: None,
            prev_signal: None,
        }
    }

    pub fn update(&mut self, high: f64, low: f64, close: f64) -> SuperTrendSignal {
        let atr_value = self.atr.update(high, low, close);
        let hl_avg = (high + low) / 2.0;
        let basic_upper = hl_avg + (self.multiplier * atr_value);
        let basic_lower = hl_avg - (self.multiplier * atr_value);

        let prev_upper = self.upper_band;
        let prev_lower = self.lower_band;
        let prev_supertrend = self.supertrend;

        let final_upper = match prev_upper {
            Some(prev_upper) if basic_upper < prev_upper || close > prev_upper => basic_upper,
            Some(prev_upper) => prev_upper,
            None => basic_upper,
        };

        let final_lower = match prev_lower {
            Some(prev_lower) if basic_lower > prev_lower || close < prev_lower => basic_lower,
            Some(prev_lower) => prev_lower,
            None => basic_lower,
        };

        let supertrend = match (prev_supertrend, prev_upper) {
            (Some(prev_st), Some(prev_upper)) if (prev_st - prev_upper).abs() < f64::EPSILON => {
                if close <= final_upper {
                    final_upper
                } else {
                    final_lower
                }
            }
            _ => {
                if close >= final_lower {
                    final_lower
                } else {
                    final_upper
                }
            }
        };

        self.upper_band = Some(final_upper);
        self.lower_band = Some(final_lower);
        self.supertrend = Some(supertrend);

        let current_signal = if close > supertrend {
            SuperTrendSignal::Bullish
        } else {
            SuperTrendSignal::Bearish
        };

        let signal = match (&self.prev_signal, &current_signal) {
            (Some(SuperTrendSignal::Bearish), SuperTrendSignal::Bullish) => {
                SuperTrendSignal::BullishFlip
            }
            (Some(SuperTrendSignal::Bullish), SuperTrendSignal::Bearish) => {
                SuperTrendSignal::BearishFlip
            }
            _ => current_signal.clone(),
        };

        self.prev_signal = Some(current_signal);
        signal
    }

    pub fn value(&self) -> Option<f64> {
        self.supertrend
    }
}
