use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Represents the comprehensive result of a simulation run.
///
/// This struct aggregates various performance metrics such as final value,
/// fees earned, impermanent loss, and risk measures like maximum drawdown
/// and Sharpe ratio to evaluate the effectiveness of a liquidity strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationResult {
    /// The total value of the position at the end of the simulation (usually in USD or quote currency).
    pub final_position_value: Decimal,

    /// The total amount of fees collected during the simulation period.
    pub total_fees_earned: Decimal,

    /// The calculated total Impermanent Loss (IL).
    /// Represents the opportunity cost of providing liquidity versus holding the assets.
    pub total_il: Decimal,

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
