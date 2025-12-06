//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_domain::prelude::*;
//! ```

// Entities
pub use crate::entities::pool::Pool;
pub use crate::entities::position::{Position, PositionId};
pub use crate::entities::price_candle::PriceCandle;
pub use crate::entities::token::Token;

// Enums
pub use crate::enums::{OptimizationObjective, PoolType, PositionStatus, Protocol, TimeHorizon};

// Fees
pub use crate::fees::{FeeAccumulation, FeeTier};

// Math functions
pub use crate::math::concentrated_liquidity::{
    get_amount0_delta, get_amount1_delta, get_liquidity_for_amount0, get_liquidity_for_amount1,
};
pub use crate::math::constant_product::{calculate_k, calculate_out_amount, calculate_spot_price};
pub use crate::math::price_tick::{price_to_tick, tick_to_price};

// Metrics
pub use crate::metrics::fees::{calculate_apy, calculate_pool_fees};
pub use crate::metrics::impermanent_loss::{
    calculate_il_concentrated, calculate_il_constant_product,
};
pub use crate::metrics::{APY, ImpermanentLoss, PnL};

// Value objects
pub use crate::value_objects::amount::Amount;
pub use crate::value_objects::optimization_result::OptimizationResult;
pub use crate::value_objects::percentage::Percentage;
pub use crate::value_objects::price::Price;
pub use crate::value_objects::price_range::PriceRange;
pub use crate::value_objects::simulation_result::SimulationResult;
pub use crate::value_objects::{
    FeeEarnings, ImpermanentLossResult, PoolMetrics, RiskMetrics, VolatilityEstimate,
};
