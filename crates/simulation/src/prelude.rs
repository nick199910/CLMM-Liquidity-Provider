//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_simulation::prelude::*;
//! ```

// Engine
pub use crate::engine::SimulationEngine;

// Liquidity models
pub use crate::liquidity::{ConstantLiquidity, LiquidityModel};

// Monte Carlo
pub use crate::monte_carlo::{AggregateResult, MonteCarloRunner};

// Price path generators
pub use crate::price_path::{
    DeterministicPricePath, GeometricBrownianMotion, HistoricalPricePath, PricePathGenerator,
};

// Volume models
pub use crate::volume::{ConstantVolume, VolumeModel};
