//! Event types for CLMM protocols.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Protocol event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolEvent {
    /// A swap occurred in the pool.
    Swap(SwapEvent),
    /// Liquidity was added to a position.
    IncreaseLiquidity(LiquidityEvent),
    /// Liquidity was removed from a position.
    DecreaseLiquidity(LiquidityEvent),
    /// Fees were collected from a position.
    CollectFees(CollectFeesEvent),
    /// A new position was opened.
    OpenPosition(OpenPositionEvent),
    /// A position was closed.
    ClosePosition(ClosePositionEvent),
}

/// Swap event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapEvent {
    /// Transaction signature.
    pub signature: String,
    /// Pool address.
    pub pool: String,
    /// Timestamp in seconds.
    pub timestamp: u64,
    /// Slot number.
    pub slot: u64,
    /// Amount of token A swapped.
    pub amount_a: u64,
    /// Amount of token B swapped.
    pub amount_b: u64,
    /// Whether this was a buy (A -> B) or sell (B -> A).
    pub is_buy: bool,
    /// Price after the swap.
    pub sqrt_price_after: u128,
    /// Tick after the swap.
    pub tick_after: i32,
    /// Fee amount.
    pub fee_amount: u64,
}

/// Liquidity change event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityEvent {
    /// Transaction signature.
    pub signature: String,
    /// Pool address.
    pub pool: String,
    /// Position address.
    pub position: String,
    /// Timestamp in seconds.
    pub timestamp: u64,
    /// Slot number.
    pub slot: u64,
    /// Liquidity delta.
    pub liquidity_delta: u128,
    /// Token A amount.
    pub token_a_amount: u64,
    /// Token B amount.
    pub token_b_amount: u64,
    /// Lower tick of the position.
    pub tick_lower: i32,
    /// Upper tick of the position.
    pub tick_upper: i32,
}

/// Collect fees event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectFeesEvent {
    /// Transaction signature.
    pub signature: String,
    /// Pool address.
    pub pool: String,
    /// Position address.
    pub position: String,
    /// Timestamp in seconds.
    pub timestamp: u64,
    /// Slot number.
    pub slot: u64,
    /// Token A fees collected.
    pub fee_a: u64,
    /// Token B fees collected.
    pub fee_b: u64,
}

/// Open position event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenPositionEvent {
    /// Transaction signature.
    pub signature: String,
    /// Pool address.
    pub pool: String,
    /// Position address.
    pub position: String,
    /// Owner address.
    pub owner: String,
    /// Timestamp in seconds.
    pub timestamp: u64,
    /// Slot number.
    pub slot: u64,
    /// Lower tick.
    pub tick_lower: i32,
    /// Upper tick.
    pub tick_upper: i32,
}

/// Close position event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosePositionEvent {
    /// Transaction signature.
    pub signature: String,
    /// Pool address.
    pub pool: String,
    /// Position address.
    pub position: String,
    /// Timestamp in seconds.
    pub timestamp: u64,
    /// Slot number.
    pub slot: u64,
}

/// Aggregated volume data for a time period.
#[derive(Debug, Clone, Default)]
pub struct VolumeData {
    /// Total volume in token A.
    pub volume_a: u64,
    /// Total volume in token B.
    pub volume_b: u64,
    /// Total volume in USD.
    pub volume_usd: Decimal,
    /// Number of swaps.
    pub swap_count: u64,
    /// Total fees in token A.
    pub fees_a: u64,
    /// Total fees in token B.
    pub fees_b: u64,
}

/// Position state from on-chain data.
#[derive(Debug, Clone)]
pub struct OnChainPosition {
    /// Position address.
    pub address: Pubkey,
    /// Pool address.
    pub pool: Pubkey,
    /// Owner address.
    pub owner: Pubkey,
    /// Lower tick.
    pub tick_lower: i32,
    /// Upper tick.
    pub tick_upper: i32,
    /// Liquidity amount.
    pub liquidity: u128,
    /// Fee growth inside for token A.
    pub fee_growth_inside_a: u128,
    /// Fee growth inside for token B.
    pub fee_growth_inside_b: u128,
    /// Uncollected fees for token A.
    pub fees_owed_a: u64,
    /// Uncollected fees for token B.
    pub fees_owed_b: u64,
}
