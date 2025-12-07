//! Output formatting for CLI.
//!
//! This module provides rich output formatting including tables,
//! charts, and export functionality.

pub mod chart;
pub mod export;
pub mod table;

pub use chart::*;
pub use export::*;
pub use table::*;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Analysis report structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Trading pair.
    pub pair: String,
    /// Analysis period in days.
    pub period_days: u64,
    /// Current price.
    pub current_price: Decimal,
    /// Highest price in period.
    pub high_price: Decimal,
    /// Lowest price in period.
    pub low_price: Decimal,
    /// Average price.
    pub avg_price: Decimal,
    /// Daily volatility.
    pub volatility_daily: Decimal,
    /// Annualized volatility.
    pub volatility_annual: Decimal,
    /// Recommended lower price bound.
    pub recommended_lower: Decimal,
    /// Recommended upper price bound.
    pub recommended_upper: Decimal,
    /// Recommended range width.
    pub recommended_width: Decimal,
    /// Estimated time in range percentage.
    pub estimated_time_in_range: Decimal,
    /// Number of data points analyzed.
    pub data_points: usize,
}

/// Backtest report structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReport {
    /// Trading pair.
    pub pair: String,
    /// Backtest period in days.
    pub period_days: u64,
    /// Entry price.
    pub entry_price: Decimal,
    /// Exit price.
    pub exit_price: Decimal,
    /// Range lower bound.
    pub range_lower: Decimal,
    /// Range upper bound.
    pub range_upper: Decimal,
    /// Initial capital.
    pub initial_capital: Decimal,
    /// Final portfolio value.
    pub final_value: Decimal,
    /// Total return percentage.
    pub total_return: Decimal,
    /// Total fees earned.
    pub fee_earnings: Decimal,
    /// Total impermanent loss.
    pub impermanent_loss: Decimal,
    /// Performance vs HODL.
    pub vs_hodl: Decimal,
    /// Time in range percentage.
    pub time_in_range: Decimal,
    /// Maximum drawdown.
    pub max_drawdown: Decimal,
    /// Number of rebalances.
    pub rebalance_count: u32,
    /// Total transaction costs.
    pub total_tx_costs: Decimal,
    /// Strategy used.
    pub strategy: String,
    /// Sharpe ratio if calculable.
    pub sharpe_ratio: Option<Decimal>,
}

/// Optimization report structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationReport {
    /// Trading pair.
    pub pair: String,
    /// Current price.
    pub current_price: Decimal,
    /// Volatility estimate.
    pub volatility: Decimal,
    /// Capital to optimize for.
    pub capital: Decimal,
    /// Optimization objective.
    pub objective: String,
    /// Ranked candidates.
    pub candidates: Vec<RangeCandidate>,
    /// Strategy recommendations.
    pub strategy_recommendations: Vec<crate::commands::optimize::StrategyRecommendation>,
}

/// Range candidate from optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeCandidate {
    /// Rank (1 = best).
    pub rank: usize,
    /// Range width as percentage.
    pub range_width_pct: Decimal,
    /// Lower price bound.
    pub lower_price: Decimal,
    /// Upper price bound.
    pub upper_price: Decimal,
    /// Expected fees.
    pub expected_fees: Decimal,
    /// Expected IL.
    pub expected_il: Decimal,
    /// Expected net PnL.
    pub expected_pnl: Decimal,
    /// Estimated time in range.
    pub time_in_range: Decimal,
    /// Optimization score.
    pub score: Decimal,
}
