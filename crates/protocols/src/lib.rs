//! Protocol adapters and interfaces.
pub mod orca;
pub mod parsers;
pub mod raydium;
pub mod rpc;
pub mod solana_client; // Whirlpools

use anyhow::Result;
use async_trait::async_trait;
use clmm_lp_domain::entities::pool::Pool;

/// Trait for fetching pool data.
#[async_trait]
pub trait PoolFetcher {
    /// Fetches pool data by address.
    async fn fetch_pool(&self, pool_address: &str) -> Result<Pool>;
}
