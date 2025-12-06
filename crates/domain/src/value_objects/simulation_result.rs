use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents the comprehensive result of a simulation run.
///
/// This struct aggregates various performance metrics such as final value,
/// fees earned, impermanent loss, and risk measures like maximum drawdown
/// and Sharpe ratio to evaluate the effectiveness of a liquidity strategy.
/// Result of a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Final value of the position.
    pub final_position_value: Decimal,

    /// Total fees earned.
    pub total_fees_earned: Decimal,

    /// Total impermanent loss.
    pub total_il: Decimal,

    /// Net Profit and Loss.
    /// The Net Profit and Loss (PnL) of the strategy.
    /// Typically calculated as `(Final Value + Fees) - Initial Value`.
    pub net_pnl: Decimal,

    /// The maximum drawdown percentage experienced during the simulation.
    /// Represents the largest observed peak-to-trough decline in portfolio value.
    pub max_drawdown: Decimal,

    /// The percentage of time the asset price remained within the active liquidity range.
    /// A value of 1.0 represents 100%, while 0.5 represents 50%.
    pub time_in_range_percentage: Decimal,

    /// The Sharpe ratio of the strategy, if sufficient data exists to calculate it.
    /// Used to measure the risk-adjusted return.
    pub sharpe_ratio: Option<Decimal>,
}
