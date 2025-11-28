//! On-Balance Volume indicator with divergence detection.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OBVSignal {
    BullishDivergence,
    BearishDivergence,
    Confirmation,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct OBV {
    current_obv: f64,
    prev_close: Option<f64>,
    obv_ema: Option<f64>,
}

impl OBV {
    pub fn new() -> Self {
        Self {
            current_obv: 0.0,
            prev_close: None,
            obv_ema: None,
        }
    }

    pub fn update(&mut self, close: f64, volume: f64) -> (f64, OBVSignal) {
        if let Some(prev_close) = self.prev_close {
            if close > prev_close {
                self.current_obv += volume;
            } else if close < prev_close {
                self.current_obv -= volume;
            }
        } else {
            self.current_obv = volume;
        }

        let previous_smoothed = self.obv_ema.unwrap_or(self.current_obv);
        let obv_ema = match self.obv_ema {
            Some(prev_ema) => prev_ema * 0.9 + self.current_obv * 0.1,
            None => self.current_obv,
        };
        self.obv_ema = Some(obv_ema);
        let obv_change = obv_ema - previous_smoothed;

        let signal = if let Some(prev_close_val) = self.prev_close {
            let price_change = close - prev_close_val;

            if price_change < 0.0 && obv_change > 0.0 {
                OBVSignal::BullishDivergence
            } else if price_change > 0.0 && obv_change < 0.0 {
                OBVSignal::BearishDivergence
            } else if (price_change > 0.0 && obv_change > 0.0)
                || (price_change < 0.0 && obv_change < 0.0)
            {
                OBVSignal::Confirmation
            } else {
                OBVSignal::Neutral
            }
        } else {
            OBVSignal::Neutral
        };

        self.prev_close = Some(close);
        (self.current_obv, signal)
    }

    pub fn smoothed(&self) -> Option<f64> {
        self.obv_ema
    }
}

impl Default for OBV {
    fn default() -> Self {
        Self::new()
    }
}
