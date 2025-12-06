//! Value objects for the domain.
/// Amount value object.
pub mod amount;
/// Optimization result value object.
pub mod optimization_result;
/// Percentage value object.
pub mod percentage;
/// Price value object.
pub mod price;
/// Price range value object.
pub mod price_range;
/// Simulation result value object.
pub mod simulation_result;

pub use optimization_result::OptimizationResult;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents a volatility estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityEstimate {
    /// Annualized volatility.
    pub annualized_volatility: Decimal,
    /// Method used for estimation.
    pub method: String,
}

/// Represents impermanent loss result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpermanentLossResult {
    /// IL percentage.
    pub il_percentage: Decimal,
    /// IL amount in USD.
    pub il_amount_usd: Decimal,
}

/// Represents fee earnings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeEarnings {
    /// Amount of token A.
    pub amount_a: Decimal,
    /// Amount of token B.
    pub amount_b: Decimal,
    /// Total value in USD.
    pub total_usd: Decimal,
}

/// Represents pool metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolMetrics {
    /// Total Value Locked in USD.
    pub tvl_usd: Decimal,
    /// 24h volume in USD.
    pub volume_24h_usd: Decimal,
    /// Fee APR for 24h.
    pub fee_apr_24h: Decimal,
}

/// Represents risk metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMetrics {
    /// Value at Risk (95%).
    pub var_95: Decimal,
    /// Maximum drawdown.
    pub max_drawdown: Decimal,
}
