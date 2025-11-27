//! Open interest trend detector for perp markets.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpenInterestSignal {
    BullishExpansion,
    BearishExpansion,
    LongSqueeze,
    ShortSqueeze,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct OpenInterest {
    prev_oi: Option<f64>,
    prev_price: Option<f64>,
    oi_ema: Option<f64>,
}

impl OpenInterest {
    pub fn new() -> Self {
        Self {
            prev_oi: None,
            prev_price: None,
            oi_ema: None,
        }
    }

    pub fn update(&mut self, current_oi: f64, price: f64) -> OpenInterestSignal {
        let oi_ema = match self.oi_ema {
            Some(prev) => prev * 0.8 + current_oi * 0.2,
            None => current_oi,
        };
        self.oi_ema = Some(oi_ema);

        let signal = if let (Some(prev_oi), Some(prev_price)) = (self.prev_oi, self.prev_price) {
            let oi_change = current_oi - prev_oi;
            let price_change = price - prev_price;
            let oi_pct_change = if prev_oi.abs() < f64::EPSILON {
                0.0
            } else {
                (oi_change / prev_oi) * 100.0
            };

            if oi_pct_change > 2.0 {
                if price_change > 0.0 {
                    OpenInterestSignal::BullishExpansion
                } else if price_change < 0.0 {
                    OpenInterestSignal::BearishExpansion
                } else {
                    OpenInterestSignal::Neutral
                }
            } else if oi_pct_change < -2.0 {
                if price_change < 0.0 {
                    OpenInterestSignal::LongSqueeze
                } else if price_change > 0.0 {
                    OpenInterestSignal::ShortSqueeze
                } else {
                    OpenInterestSignal::Neutral
                }
            } else {
                OpenInterestSignal::Neutral
            }
        } else {
            OpenInterestSignal::Neutral
        };

        self.prev_oi = Some(current_oi);
        self.prev_price = Some(price);
        signal
    }

    pub fn smoothed(&self) -> Option<f64> {
        self.oi_ema
    }
}
