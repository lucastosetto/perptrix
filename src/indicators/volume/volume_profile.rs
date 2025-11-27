//! Volume Profile helper for identifying HVN/LVN areas.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VolumeNode {
    pub price_level: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VolumeProfileSignal {
    NearHVN,
    NearLVN,
    POCSupport,
    POCResistance,
    Neutral,
}

#[derive(Debug, Clone)]
pub struct VolumeProfile {
    price_levels: HashMap<i64, f64>,
    tick_size: f64,
    lookback: usize,
    data_points: Vec<(f64, f64)>,
}

impl VolumeProfile {
    pub fn new(tick_size: f64, lookback: usize) -> Self {
        Self {
            price_levels: HashMap::new(),
            tick_size,
            lookback,
            data_points: Vec::new(),
        }
    }

    pub fn update(&mut self, close: f64, volume: f64) {
        self.data_points.push((close, volume));
        if self.data_points.len() > self.lookback {
            if let Some((removed_price, removed_volume)) = self.data_points.first().cloned() {
                self.data_points.remove(0);
                let bucket = (removed_price / self.tick_size).round() as i64;
                if let Some(entry) = self.price_levels.get_mut(&bucket) {
                    *entry = (*entry - removed_volume).max(0.0);
                }
            }
        }

        let bucket = (close / self.tick_size).round() as i64;
        *self.price_levels.entry(bucket).or_insert(0.0) += volume;
    }

    pub fn get_profile(&self) -> (Vec<VolumeNode>, f64, VolumeProfileSignal) {
        let mut nodes: Vec<VolumeNode> = self
            .price_levels
            .iter()
            .map(|(bucket, vol)| VolumeNode {
                price_level: *bucket as f64 * self.tick_size,
                volume: *vol,
            })
            .collect();
        nodes.sort_by(|a, b| b.volume.partial_cmp(&a.volume).unwrap());

        let poc = nodes.first().map(|n| n.price_level).unwrap_or(0.0);
        let total_volume: f64 = nodes.iter().map(|n| n.volume).sum();
        let avg_volume = if nodes.is_empty() {
            0.0
        } else {
            total_volume / nodes.len() as f64
        };
        let hvn_threshold = avg_volume * 1.5;
        let lvn_threshold = avg_volume * 0.5;
        let current_price = self.data_points.last().map(|(p, _)| *p).unwrap_or(0.0);

        let signal = if (current_price - poc).abs() < self.tick_size * 2.0 {
            if current_price > poc {
                VolumeProfileSignal::POCSupport
            } else {
                VolumeProfileSignal::POCResistance
            }
        } else {
            let current_bucket = (current_price / self.tick_size).round() as i64;
            let current_vol = self
                .price_levels
                .get(&current_bucket)
                .copied()
                .unwrap_or(0.0);

            if current_vol > hvn_threshold {
                VolumeProfileSignal::NearHVN
            } else if current_vol < lvn_threshold {
                VolumeProfileSignal::NearLVN
            } else {
                VolumeProfileSignal::Neutral
            }
        };

        (nodes, poc, signal)
    }
}
