use crate::token::{Price, Token, TokenAmount};
use serde::{Deserialize, Serialize};

/// Types of pools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolType {
    /// Constant product pool.
    ConstantProduct,
    /// Concentrated liquidity pool.
    ConcentratedLiquidity,
    /// Stable swap pool.
    StableSwap,
}

/// Represents a pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    /// The ID of the pool.
    pub id: String,
    /// The address of the pool.
    pub address: String,
    /// The chain ID where the pool resides.
    pub chain_id: u64,
    /// The first token.
    pub token0: Token,
    /// The second token.
    pub token1: Token,
    /// The fee tier in basis points.
    pub fee_tier: u32, // in bps, e.g. 3000 for 0.3%
    /// The type of the pool.
    pub pool_type: PoolType,
}

/// Represents the state of a pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolState {
    /// The pool ID.
    pub pool_id: String,
    /// The reserve of token 0.
    pub reserve0: TokenAmount,
    /// The reserve of token 1.
    pub reserve1: TokenAmount,
    /// The current price.
    pub price: Price,
    /// The current tick (for concentrated liquidity).
    pub tick: Option<i32>, // for concentrated liquidity
    /// The liquidity (for concentrated liquidity).
    pub liquidity: Option<u128>, // for concentrated liquidity
    /// The timestamp of the state.
    pub timestamp: u64,
}
