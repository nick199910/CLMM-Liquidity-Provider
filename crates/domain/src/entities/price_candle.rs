use crate::entities::token::Token;
use crate::value_objects::amount::Amount;
use crate::value_objects::price::Price;
use serde::{Deserialize, Serialize};

/// Represents a price candle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceCandle {
    /// The first token in the pair.
    pub token_a: Token,
    /// The second token in the pair.
    pub token_b: Token,
    /// The start timestamp of the candle.
    pub start_timestamp: u64,
    /// The duration of the candle in seconds.
    pub duration_seconds: u64,
    /// The opening price.
    pub open: Price,
    /// The highest price.
    pub high: Price,
    /// The lowest price.
    pub low: Price,
    /// The closing price.
    pub close: Price,
    /// The volume of token A.
    pub volume_token_a: Amount,
}
