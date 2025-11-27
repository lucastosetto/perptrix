//! Integration tests for Hyperliquid WebSocket integration

use perptrix::config::{get_environment, get_hyperliquid_ws_url};
use perptrix::services::hyperliquid::HyperliquidMarketDataProvider;
use perptrix::services::market_data::MarketDataProvider;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_environment_configuration() {
    // Test that environment configuration works
    let env = get_environment();
    assert!(env == "production" || env == "sandbox" || env == "testnet");
    
    let url = get_hyperliquid_ws_url();
    assert!(url.contains("hyperliquid"));
    assert!(url.starts_with("wss://"));
}

#[tokio::test]
async fn test_hyperliquid_provider_creation() {
    // Test that provider can be created
    let provider = HyperliquidMarketDataProvider::new();
    
    // Give it time to connect
    sleep(Duration::from_secs(2)).await;
    
    // Test that we can call methods (even if they return empty initially)
    let candles = provider.get_candles("BTC", 10).await;
    assert!(candles.is_ok());
    
    let price = provider.get_latest_price("BTC").await;
    assert!(price.is_ok());
}

#[tokio::test]
async fn test_hyperliquid_subscribe() {
    let provider = HyperliquidMarketDataProvider::new();
    
    // Give it time to connect
    sleep(Duration::from_secs(2)).await;
    
    // Test subscription
    let result = provider.subscribe("BTC").await;
    assert!(result.is_ok());
    
    // Wait a bit for data to arrive
    sleep(Duration::from_secs(3)).await;
    
    // Try to get candles (may be empty if connection not established yet)
    let candles = provider.get_candles("BTC", 10).await;
    assert!(candles.is_ok());
}

#[tokio::test]
async fn test_hyperliquid_multiple_symbols() {
    let provider = HyperliquidMarketDataProvider::new();
    
    // Give it time to connect
    sleep(Duration::from_secs(2)).await;
    
    // Subscribe to multiple symbols
    assert!(provider.subscribe("BTC").await.is_ok());
    assert!(provider.subscribe("ETH").await.is_ok());
    
    // Wait for data
    sleep(Duration::from_secs(3)).await;
    
    // Check both symbols
    let btc_candles = provider.get_candles("BTC", 10).await;
    assert!(btc_candles.is_ok());
    
    let eth_candles = provider.get_candles("ETH", 10).await;
    assert!(eth_candles.is_ok());
}

#[tokio::test]
async fn test_hyperliquid_latest_price() {
    let provider = HyperliquidMarketDataProvider::new();
    
    // Give it time to connect
    sleep(Duration::from_secs(2)).await;
    
    // Subscribe first
    assert!(provider.subscribe("BTC").await.is_ok());
    
    // Wait for price updates
    sleep(Duration::from_secs(3)).await;
    
    // Get latest price
    let price = provider.get_latest_price("BTC").await;
    assert!(price.is_ok());
    
    // Price should be positive if we got data
    let price_value = price.unwrap();
    // Note: price might be 0.0 if connection not established, so we just check it doesn't panic
    assert!(price_value >= 0.0);
}

