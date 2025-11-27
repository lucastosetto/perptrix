//! Unit tests for market data provider

use perptrix::services::market_data::{MarketDataProvider, PlaceholderMarketDataProvider};

#[tokio::test]
async fn test_placeholder_provider() {
    let provider = PlaceholderMarketDataProvider;
    assert!(provider.get_candles("BTC", 100).await.is_ok());
    assert!(provider.get_latest_price("BTC").await.is_ok());
    assert!(provider.subscribe("BTC").await.is_ok());
}



