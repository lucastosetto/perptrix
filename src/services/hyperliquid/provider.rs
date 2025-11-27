//! Hyperliquid market data provider implementation

use crate::models::indicators::Candle;
use crate::services::market_data::MarketDataProvider;
use chrono::{DateTime, Utc};
use serde_json;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

use super::client::{ClientEvent, HyperliquidClient};
use super::messages::{CandleUpdate, RequestMessage, Subscription, WebSocketMessage};
use super::subscriptions::{SubscriptionKey, SubscriptionManager};

pub struct HyperliquidMarketDataProvider {
    client: Arc<HyperliquidClient>,
    subscriptions: Arc<SubscriptionManager>,
    candles: Arc<RwLock<HashMap<String, VecDeque<Candle>>>>,
    latest_prices: Arc<RwLock<HashMap<String, f64>>>,
    candle_intervals: Vec<String>,
}

impl HyperliquidMarketDataProvider {
    pub fn new() -> Self {
        Self::with_intervals(vec!["1m".to_string(), "5m".to_string(), "15m".to_string(), "1h".to_string()])
    }

    pub fn with_intervals(candle_intervals: Vec<String>) -> Self {
        let provider = Self {
            client: Arc::new(HyperliquidClient::new()),
            subscriptions: Arc::new(SubscriptionManager::new()),
            candles: Arc::new(RwLock::new(HashMap::new())),
            latest_prices: Arc::new(RwLock::new(HashMap::new())),
            candle_intervals,
        };

        // Start connection task in background
        let client_clone = provider.client.clone();
        tokio::spawn(async move {
            let _ = client_clone.connect().await;
        });

        // Start message handler task
        let provider_clone = provider.clone_for_task();
        tokio::spawn(async move {
            provider_clone.handle_messages().await;
        });

        provider
    }

    fn clone_for_task(&self) -> TaskProvider {
        TaskProvider {
            client: self.client.clone(),
            subscriptions: self.subscriptions.clone(),
            candles: self.candles.clone(),
            latest_prices: self.latest_prices.clone(),
        }
    }

    async fn subscribe_candle(&self, coin: &str, interval: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let key = SubscriptionKey::candle(coin, interval);
        
        if self.subscriptions.contains(&key).await {
            return Ok(()); // Already subscribed
        }

        let subscription = Subscription::candle(coin, interval);
        let request = RequestMessage::Subscribe { subscription };

        let json = serde_json::to_string(&request)
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;
        self.client.send_text(json).await
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, format!("WebSocket send error: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

        self.subscriptions.add(key).await;
        Ok(())
    }


    fn get_primary_interval(&self) -> &str {
        self.candle_intervals.first().map(|s| s.as_str()).unwrap_or("1m")
    }
}

#[derive(Clone)]
struct TaskProvider {
    client: Arc<HyperliquidClient>,
    #[allow(dead_code)] // Kept for future resubscription functionality
    subscriptions: Arc<SubscriptionManager>,
    candles: Arc<RwLock<HashMap<String, VecDeque<Candle>>>>,
    latest_prices: Arc<RwLock<HashMap<String, f64>>>,
}

impl TaskProvider {
    async fn handle_messages(&self) {
        loop {
            if let Some(event) = self.client.receive().await {
                match event {
                    ClientEvent::Message(text) => {
                        if let Err(e) = self.process_message(&text).await {
                            eprintln!("Error processing message: {}", e);
                        }
                    }
                    ClientEvent::Connected => {
                        println!("Hyperliquid WebSocket connected");
                    }
                    ClientEvent::Disconnected => {
                        eprintln!("Hyperliquid WebSocket disconnected");
                    }
                    ClientEvent::Error(e) => {
                        eprintln!("Hyperliquid WebSocket error: {}", e);
                    }
                }
            } else {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }

    async fn process_message(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let msg: WebSocketMessage = serde_json::from_str(text)
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())) as Box<dyn std::error::Error + Send + Sync>)?;

        match msg {
            WebSocketMessage::CandleData(candle_data) => {
                for update in candle_data.data {
                    if let Err(e) = self.process_candle_update(update).await {
                        eprintln!("Error processing candle update: {}", e);
                    }
                }
            }
            WebSocketMessage::AllMidsData(mids_data) => {
                for mid in mids_data.data {
                    let price: f64 = mid.px.parse().unwrap_or(0.0);
                    let mut prices = self.latest_prices.write().await;
                    prices.insert(mid.coin, price);
                }
            }
            WebSocketMessage::SubscriptionResponse(_) => {
                // Subscription acknowledged
            }
            WebSocketMessage::Error(err) => {
                eprintln!("WebSocket error: {}", err.data.error);
            }
        }

        Ok(())
    }

    async fn process_candle_update(&self, update: CandleUpdate) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let coin = update.coin.as_ref().ok_or_else(|| {
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing coin in candle update")) as Box<dyn std::error::Error + Send + Sync>
        })?;
        let interval = update.interval.as_ref().ok_or_else(|| {
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing interval in candle update")) as Box<dyn std::error::Error + Send + Sync>
        })?;

        let open: f64 = update.open.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid open price: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
        let high: f64 = update.high.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid high price: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
        let low: f64 = update.low.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid low price: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
        let close: f64 = update.close.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid close price: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;
        let volume: f64 = update.volume.parse()
            .map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Invalid volume: {}", e))) as Box<dyn std::error::Error + Send + Sync>)?;

        let timestamp = DateTime::from_timestamp(update.timestamp as i64 / 1000, 0)
            .unwrap_or_else(Utc::now);

        let candle = Candle::new(open, high, low, close, volume, timestamp);

        let symbol_key = format!("{}_{}", coin, interval);
        let mut candles_map = self.candles.write().await;
        let candles = candles_map.entry(symbol_key.clone()).or_insert_with(VecDeque::new);

        candles.retain(|c| c.timestamp != timestamp);
        candles.push_back(candle.clone());
        
        while candles.len() > 1000 {
            candles.pop_front();
        }

        let mut prices = self.latest_prices.write().await;
        prices.insert(coin.clone(), close);

        Ok(())
    }
}

#[async_trait::async_trait]
impl MarketDataProvider for HyperliquidMarketDataProvider {
    async fn get_candles(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        let interval = self.get_primary_interval();
        let symbol_key = format!("{}_{}", symbol, interval);
        
        let candles_map = self.candles.read().await;
        if let Some(candles) = candles_map.get(&symbol_key) {
            let mut result: Vec<Candle> = candles.iter().cloned().collect();
            result.sort_by_key(|c| c.timestamp);
            
            // Return last `limit` candles
            if result.len() > limit {
                result = result.into_iter().rev().take(limit).collect();
                result.reverse();
            }
            
            Ok(result)
        } else {
            // Try to subscribe if we don't have data yet
            if let Err(e) = self.subscribe_candle(symbol, interval).await {
                eprintln!("Failed to subscribe to {}: {}", symbol, e);
            }
            Ok(Vec::new())
        }
    }

    async fn get_latest_price(&self, symbol: &str) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let prices = self.latest_prices.read().await;
        if let Some(&price) = prices.get(symbol) {
            Ok(price)
        } else {
            // Subscribe to get price updates
            if let Err(e) = self.subscribe_candle(symbol, self.get_primary_interval()).await {
                eprintln!("Failed to subscribe to {}: {}", symbol, e);
            }
            // Wait a bit for price to arrive
            tokio::time::sleep(Duration::from_millis(500)).await;
            let prices = self.latest_prices.read().await;
            Ok(prices.get(symbol).copied().unwrap_or(0.0))
        }
    }

    async fn subscribe(&self, symbol: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Subscribe to all intervals for this symbol
        for interval in &self.candle_intervals {
            if let Err(e) = self.subscribe_candle(symbol, interval).await {
                eprintln!("Failed to subscribe to {} {}: {}", symbol, interval, e);
                // Continue with other intervals even if one fails
            }
        }
        Ok(())
    }
}

impl Default for HyperliquidMarketDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

