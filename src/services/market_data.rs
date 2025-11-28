//! Market data provider interface for future data source integration.

use crate::models::indicators::Candle;

#[async_trait::async_trait]
pub trait MarketDataProvider: Send + Sync {
    /// Get historical candles for a symbol
    async fn get_candles(
        &self,
        symbol: &str,
        limit: usize,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get the latest price for a symbol
    async fn get_latest_price(
        &self,
        symbol: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>>;

    async fn subscribe(&self, symbol: &str)
        -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

pub struct PlaceholderMarketDataProvider;

#[async_trait::async_trait]
impl MarketDataProvider for PlaceholderMarketDataProvider {
    async fn get_candles(
        &self,
        _symbol: &str,
        _limit: usize,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Vec::new())
    }

    async fn get_latest_price(
        &self,
        _symbol: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        Ok(0.0)
    }

    async fn subscribe(
        &self,
        _symbol: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}
