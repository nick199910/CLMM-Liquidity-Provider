//! Mathematical functions and utilities for CLMM pools.
//!
//! This module provides core mathematical operations for:
//! - Concentrated liquidity calculations
//! - Constant product AMM math
//! - Price/tick conversions
//! - Fee calculations
//! - Price impact estimation

/// Concentrated liquidity math.
pub mod concentrated_liquidity;
/// Constant product AMM math.
pub mod constant_product;
/// Fee tier and fee calculations.
pub mod fee_math;
/// Price impact estimation for swaps.
pub mod price_impact;
/// Price tick conversions.
pub mod price_tick;
