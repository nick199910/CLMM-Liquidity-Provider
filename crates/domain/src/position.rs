use crate::token::{Price, TokenAmount};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a position.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionId(pub Uuid);

/// Represents a price range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    /// The lower price bound.
    pub lower_price: Price,
    /// The upper price bound.
    pub upper_price: Price,
    /// The lower tick bound.
    pub lower_tick: Option<i32>,
    /// The upper tick bound.
    pub upper_tick: Option<i32>,
}

impl Range {
    /// Checks if the current price is within the range.
    pub fn is_in_range(&self, current_price: Price) -> bool {
        current_price.0 >= self.lower_price.0 && current_price.0 <= self.upper_price.0
    }
}

/// Represents a liquidity position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPosition {
    /// The position ID.
    pub id: PositionId,
    /// The pool ID.
    pub pool_id: String,
    /// The range of the position.
    pub range: Range,
    /// The liquidity amount.
    pub liquidity: u128,
    /// The amount of token 0.
    pub amount0: TokenAmount,
    /// The amount of token 1.
    pub amount1: TokenAmount,
    /// The fee growth inside 0 last.
    pub fee_growth_inside0_last: U256,
    /// The fee growth inside 1 last.
    pub fee_growth_inside1_last: U256,
}

impl LiquidityPosition {
    /// Calculates the share of fees this position earns relative to total liquidity at the current tick.
    /// Returns a decimal between 0 and 1.
    pub fn calculate_fee_share(&self, total_liquidity_at_tick: u128) -> Decimal {
        if total_liquidity_at_tick == 0 {
            return Decimal::ZERO;
        }
        // Handle potential overflow if u128 is max, but Decimal supports high precision.
        Decimal::from(self.liquidity) / Decimal::from(total_liquidity_at_tick)
    }
}

use primitive_types::U256;
