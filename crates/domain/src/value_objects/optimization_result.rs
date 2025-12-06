use crate::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents the result of an optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    /// The recommended price range.
    pub recommended_range: PriceRange,
    /// The expected Profit and Loss.
    pub expected_pnl: Decimal,
    /// The expected fees.
    pub expected_fees: Decimal,
    /// The expected impermanent loss.
    pub expected_il: Decimal,
    /// The Sharpe ratio.
    pub sharpe_ratio: Option<Decimal>,
}
