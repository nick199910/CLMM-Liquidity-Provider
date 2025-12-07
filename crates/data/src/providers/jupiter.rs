//! Jupiter Price API provider for market data.
//!
//! This module provides integration with Jupiter's Price API v2
//! for fetching token prices on Solana.

use crate::MarketDataProvider;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use clmm_lp_domain::entities::price_candle::PriceCandle;
use clmm_lp_domain::entities::token::Token;
use clmm_lp_domain::value_objects::{amount::Amount, price::Price};
use primitive_types::U256;
use reqwest::Client;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::Deserialize;
use std::collections::HashMap;

/// Base URL for Jupiter Price API v2.
const JUPITER_PRICE_API_V2: &str = "https://api.jup.ag/price/v2";

/// Response from Jupiter Price API.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JupiterPriceResponse {
    /// Price data keyed by token mint address.
    data: HashMap<String, JupiterPriceData>,
    /// Timestamp of the response in milliseconds.
    #[serde(rename = "timeTaken")]
    time_taken: Option<f64>,
}

/// Price data for a single token.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JupiterPriceData {
    /// Token mint address.
    id: String,
    /// Price in USD.
    price: String,
    /// Extra price info (optional).
    #[serde(rename = "extraInfo")]
    extra_info: Option<JupiterExtraInfo>,
}

/// Extra price information.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JupiterExtraInfo {
    /// Last swap price info.
    #[serde(rename = "lastSwappedPrice")]
    last_swapped_price: Option<JupiterSwapPrice>,
    /// Confidence level.
    #[serde(rename = "confidenceLevel")]
    confidence_level: Option<String>,
}

/// Last swap price information.
#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JupiterSwapPrice {
    /// Last Jupiter swap price.
    #[serde(rename = "lastJupiterSellPrice")]
    last_jupiter_sell_price: Option<String>,
    /// Last Jupiter buy price.
    #[serde(rename = "lastJupiterBuyPrice")]
    last_jupiter_buy_price: Option<String>,
}

/// Provider for Jupiter Price API.
pub struct JupiterProvider {
    /// The HTTP client.
    client: Client,
    /// Optional API key for higher rate limits.
    api_key: Option<String>,
    /// Base URL (can be overridden for testing).
    base_url: String,
}

impl JupiterProvider {
    /// Creates a new JupiterProvider without an API key.
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
            base_url: JUPITER_PRICE_API_V2.to_string(),
        }
    }

    /// Creates a new JupiterProvider with an API key.
    #[must_use]
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(api_key),
            base_url: JUPITER_PRICE_API_V2.to_string(),
        }
    }

    /// Sets a custom base URL (useful for testing).
    #[must_use]
    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    /// Fetches the current price for a single token.
    ///
    /// # Arguments
    /// * `mint_address` - The token's mint address
    ///
    /// # Returns
    /// The price in USD as a Decimal
    pub async fn get_price(&self, mint_address: &str) -> Result<Decimal> {
        let prices = self.get_prices(&[mint_address.to_string()]).await?;
        prices
            .get(mint_address)
            .copied()
            .ok_or_else(|| anyhow!("Price not found for {}", mint_address))
    }

    /// Fetches current prices for multiple tokens.
    ///
    /// # Arguments
    /// * `mint_addresses` - List of token mint addresses
    ///
    /// # Returns
    /// HashMap of mint address to price in USD
    pub async fn get_prices(&self, mint_addresses: &[String]) -> Result<HashMap<String, Decimal>> {
        if mint_addresses.is_empty() {
            return Ok(HashMap::new());
        }

        let ids = mint_addresses.join(",");
        let url = format!("{}?ids={}", self.base_url, ids);

        let mut request = self.client.get(&url);

        if let Some(ref api_key) = self.api_key {
            request = request.header("x-api-key", api_key);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Jupiter API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            ));
        }

        let data: JupiterPriceResponse = response.json().await?;

        let mut prices = HashMap::new();
        for (mint, price_data) in data.data {
            if let Ok(price) = price_data.price.parse::<f64>()
                && let Some(decimal_price) = Decimal::from_f64(price)
            {
                prices.insert(mint, decimal_price);
            }
        }

        Ok(prices)
    }

    /// Fetches the price ratio between two tokens.
    ///
    /// # Arguments
    /// * `base_mint` - Base token mint address
    /// * `quote_mint` - Quote token mint address
    ///
    /// # Returns
    /// Price of base token in terms of quote token
    pub async fn get_price_ratio(&self, base_mint: &str, quote_mint: &str) -> Result<Decimal> {
        let prices = self
            .get_prices(&[base_mint.to_string(), quote_mint.to_string()])
            .await?;

        let base_price = prices
            .get(base_mint)
            .ok_or_else(|| anyhow!("Price not found for base token {}", base_mint))?;

        let quote_price = prices
            .get(quote_mint)
            .ok_or_else(|| anyhow!("Price not found for quote token {}", quote_mint))?;

        if quote_price.is_zero() {
            return Err(anyhow!("Quote token price is zero"));
        }

        Ok(*base_price / *quote_price)
    }

    /// Generates synthetic historical candles from current price.
    ///
    /// Note: Jupiter API doesn't provide historical OHLCV data.
    /// This method creates synthetic candles for testing purposes.
    ///
    /// # Arguments
    /// * `mint_address` - Token mint address
    /// * `start_time` - Start timestamp in seconds
    /// * `end_time` - End timestamp in seconds
    /// * `resolution` - Candle interval in seconds
    ///
    /// # Returns
    /// Vector of synthetic price candles
    pub async fn get_synthetic_candles(
        &self,
        mint_address: &str,
        start_time: u64,
        end_time: u64,
        resolution: u64,
    ) -> Result<Vec<PriceCandle>> {
        let current_price = self.get_price(mint_address).await?;

        let mut candles = Vec::new();
        let mut timestamp = start_time;

        while timestamp <= end_time {
            // Create synthetic candle with current price (no historical data)
            let candle = PriceCandle {
                token_a: Token::new(mint_address, "UNKNOWN", 9, "Unknown Token"),
                token_b: Token::new("", "USD", 6, "US Dollar"),
                start_timestamp: timestamp,
                duration_seconds: resolution,
                open: Price::new(current_price),
                high: Price::new(current_price),
                low: Price::new(current_price),
                close: Price::new(current_price),
                volume_token_a: Amount::new(U256::zero(), 6),
            };
            candles.push(candle);
            timestamp += resolution;
        }

        Ok(candles)
    }
}

impl Default for JupiterProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketDataProvider for JupiterProvider {
    /// Fetches price history for a token pair.
    ///
    /// Note: Jupiter API doesn't provide historical OHLCV data.
    /// This implementation returns synthetic candles based on current price.
    async fn get_price_history(
        &self,
        token_a: &Token,
        token_b: &Token,
        start_time: u64,
        end_time: u64,
        resolution: u64,
    ) -> Result<Vec<PriceCandle>> {
        // Get current prices for both tokens
        let prices = self
            .get_prices(&[token_a.mint_address.clone(), token_b.mint_address.clone()])
            .await?;

        let price_a = prices
            .get(&token_a.mint_address)
            .ok_or_else(|| anyhow!("Price not found for {}", token_a.symbol))?;

        let price_b = prices
            .get(&token_b.mint_address)
            .ok_or_else(|| anyhow!("Price not found for {}", token_b.symbol))?;

        // Calculate price ratio (A in terms of B)
        let price_ratio = if price_b.is_zero() {
            Decimal::ONE
        } else {
            *price_a / *price_b
        };

        // Generate synthetic candles
        let mut candles = Vec::new();
        let mut timestamp = start_time;

        while timestamp <= end_time {
            let candle = PriceCandle {
                token_a: token_a.clone(),
                token_b: token_b.clone(),
                start_timestamp: timestamp,
                duration_seconds: resolution,
                open: Price::new(price_ratio),
                high: Price::new(price_ratio),
                low: Price::new(price_ratio),
                close: Price::new(price_ratio),
                volume_token_a: Amount::new(U256::zero(), 6),
            };
            candles.push(candle);
            timestamp += resolution;
        }

        Ok(candles)
    }
}

/// Well-known Solana token mint addresses.
pub mod known_mints {
    /// SOL (wrapped) mint address.
    pub const SOL: &str = "So11111111111111111111111111111111111111112";
    /// USDC mint address.
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    /// USDT mint address.
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    /// RAY (Raydium) mint address.
    pub const RAY: &str = "4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R";
    /// ORCA mint address.
    pub const ORCA: &str = "orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE";
    /// JUP (Jupiter) mint address.
    pub const JUP: &str = "JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN";
    /// BONK mint address.
    pub const BONK: &str = "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jupiter_provider_creation() {
        let provider = JupiterProvider::new();
        assert!(provider.api_key.is_none());
        assert_eq!(provider.base_url, JUPITER_PRICE_API_V2);
    }

    #[test]
    fn test_jupiter_provider_with_api_key() {
        let provider = JupiterProvider::with_api_key("test-key".to_string());
        assert_eq!(provider.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_known_mints() {
        // Solana addresses are base58 encoded and typically 43-44 characters
        assert!(known_mints::SOL.len() >= 43);
        assert!(known_mints::USDC.len() >= 43);
    }
}
