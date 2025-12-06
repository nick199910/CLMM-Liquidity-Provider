use crate::entities::token::Token;
use crate::enums::{PoolType, Protocol};
use crate::value_objects::amount::Amount;
use serde::{Deserialize, Serialize};

/// Represents a liquidity pool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    /// The address of the pool.
    pub address: String,
    /// The protocol the pool belongs to.
    pub protocol: Protocol,
    /// The type of the pool.
    pub pool_type: PoolType,
    /// The first token in the pool.
    pub token_a: Token,
    /// The second token in the pool.
    pub token_b: Token,
    /// The reserve amount of token A.
    pub reserve_a: Amount,
    /// The reserve amount of token B.
    pub reserve_b: Amount,
    /// The fee rate in basis points.
    pub fee_rate: u32, // bps

    // Specific to CLMM
    /// The tick spacing for concentrated liquidity pools.
    pub tick_spacing: Option<i32>,
    /// The current tick of the pool.
    pub current_tick: Option<i32>,
    /// The liquidity of the pool.
    pub liquidity: Option<u128>,

    // Specific to Stable
    /// The amplification coefficient for stable swap pools.
    pub amplification_coefficient: Option<u64>,

    /// The creation timestamp of the pool.
    pub created_at: u64,
}
