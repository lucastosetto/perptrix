//! Subscription management for Hyperliquid WebSocket

use std::collections::HashSet;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SubscriptionKey {
    pub sub_type: String,
    pub coin: Option<String>,
    pub interval: Option<String>,
}

impl SubscriptionKey {
    pub fn candle(coin: &str, interval: &str) -> Self {
        Self {
            sub_type: "candle".to_string(),
            coin: Some(coin.to_string()),
            interval: Some(interval.to_string()),
        }
    }

    pub fn all_mids() -> Self {
        Self {
            sub_type: "allMids".to_string(),
            coin: None,
            interval: None,
        }
    }
}

pub struct SubscriptionManager {
    active: RwLock<HashSet<SubscriptionKey>>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            active: RwLock::new(HashSet::new()),
        }
    }

    pub async fn add(&self, key: SubscriptionKey) {
        let mut active = self.active.write().await;
        active.insert(key);
    }

    pub async fn remove(&self, key: &SubscriptionKey) {
        let mut active = self.active.write().await;
        active.remove(key);
    }

    pub async fn contains(&self, key: &SubscriptionKey) -> bool {
        let active = self.active.read().await;
        active.contains(key)
    }

    pub async fn is_empty(&self) -> bool {
        let active = self.active.read().await;
        active.is_empty()
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

