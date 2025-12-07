//! Mock market data provider for testing.

use crate::MarketDataProvider;
use anyhow::Result;
use async_trait::async_trait;
use clmm_lp_domain::entities::price_candle::PriceCandle;
use clmm_lp_domain::entities::token::Token;
use clmm_lp_domain::value_objects::{amount::Amount, price::Price};
use primitive_types::U256;
use rust_decimal::Decimal;

/// Mock market data provider for testing purposes.
pub struct MockMarketDataProvider;

#[async_trait]
impl MarketDataProvider for MockMarketDataProvider {
    async fn get_price_history(
        &self,
        token_a: &Token,
        token_b: &Token,
        start_time: u64,
        _end_time: u64,
        resolution: u64,
    ) -> Result<Vec<PriceCandle>> {
        Ok(vec![PriceCandle {
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            start_timestamp: start_time,
            duration_seconds: resolution,
            open: Price::new(Decimal::from(100)),
            high: Price::new(Decimal::from(100)),
            low: Price::new(Decimal::from(100)),
            close: Price::new(Decimal::from(100)),
            volume_token_a: Amount::new(U256::from(0), token_a.decimals),
        }])
    }
}
