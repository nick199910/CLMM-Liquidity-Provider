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
/// Common value object types.
mod types;

pub use optimization_result::OptimizationResult;
pub use types::{FeeEarnings, ImpermanentLossResult, PoolMetrics, RiskMetrics, VolatilityEstimate};
