//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_optimization::prelude::*;
//! ```

// Objective functions
pub use crate::objective::{MaximizeFees, MaximizeNetPnL, MaximizeSharpeRatio, ObjectiveFunction};

// Range optimizer
pub use crate::range_optimizer::RangeOptimizer;
