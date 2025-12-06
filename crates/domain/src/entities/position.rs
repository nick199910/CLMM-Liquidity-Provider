use crate::enums::PositionStatus;
use crate::value_objects::{amount::Amount, price_range::PriceRange};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PositionId(pub Uuid);

/// Represents a liquidity position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// The unique identifier of the position.
    pub id: PositionId,
    /// The address of the pool.
    pub pool_address: String,
    /// The address of the owner.
    pub owner_address: String,
    /// The liquidity amount.
    pub liquidity_amount: u128,
    /// The deposited amount of token A.
    pub deposited_amount_a: Amount,
    /// The deposited amount of token B.
    pub deposited_amount_b: Amount,
    /// The current amount of token A.
    pub current_amount_a: Amount,
    /// The current amount of token B.
    pub current_amount_b: Amount,
    /// The unclaimed fees of token A.
    pub unclaimed_fees_a: Amount,
    /// The unclaimed fees of token B.
    pub unclaimed_fees_b: Amount,
    /// The price range of the position.
    pub range: Option<PriceRange>, // For CLMM
    /// The timestamp when the position was opened.
    pub opened_at: u64,
    /// The status of the position.
    pub status: PositionStatus,
}
