pub mod providers;
pub mod repository;

use amm_domain::entities::price_candle::PriceCandle;
use amm_domain::entities::token::Token;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait MarketDataProvider {
    async fn get_price_history(
        &self,
        token_a: &Token,
        token_b: &Token,
        start_time: u64,
        end_time: u64,
        resolution: u64, // seconds
    ) -> Result<Vec<PriceCandle>>;
}
