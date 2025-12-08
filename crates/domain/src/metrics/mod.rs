//! Metrics for analysis.

/// Fee related metrics.
pub mod fees;
/// Impermanent loss metrics.
pub mod impermanent_loss;
/// Metric types.
mod types;

pub use types::{APY, ImpermanentLoss, PnL};
