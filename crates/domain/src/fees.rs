use crate::token::TokenAmount;
use serde::{Deserialize, Serialize};

/// Represents a fee tier.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FeeTier {
    /// Fee in basis points.
    pub bps: u32,
    /// Tick spacing.
    pub tick_spacing: i32,
}

/// Represents accumulated fees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeAccumulation {
    /// Amount of token 0.
    pub amount0: TokenAmount,
    /// Amount of token 1.
    pub amount1: TokenAmount,
    /// Uncollected amount of token 0.
    pub uncollected0: TokenAmount,
    /// Uncollected amount of token 1.
    pub uncollected1: TokenAmount,
}
