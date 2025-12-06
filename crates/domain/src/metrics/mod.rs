use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Metrics for analysis.
pub mod fees;
pub mod impermanent_loss;

/// Represents impermanent loss.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpermanentLoss {
    /// Absolute loss in USD.
    pub absolute_loss_usd: Decimal,
    /// Percentage loss.
    pub percentage_loss: Decimal,
}

/// Represents Annual Percentage Yield (APY).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APY {
    /// Estimated annual return.
    pub estimated_annual_return: Decimal,
    /// Number of days the APY is based on.
    pub based_on_days: u32,
}

/// Represents Profit and Loss (PnL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnL {
    /// Unrealized PnL in USD.
    pub unrealized_pnl_usd: Decimal,
    /// Realized PnL in USD.
    pub realized_pnl_usd: Decimal,
    /// Total PnL in USD.
    pub total_pnl_usd: Decimal,
    /// Return on Investment (ROI) percentage.
    pub roi_percent: Decimal,
}
