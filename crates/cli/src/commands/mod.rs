//! CLI command implementations.
//!
//! This module contains the implementation of all CLI commands,
//! separated into logical modules for maintainability.

pub mod analyze;
pub mod backtest;
pub mod data;
pub mod optimize;

pub use analyze::run_analyze;
pub use backtest::run_backtest;
pub use data::run_data;
pub use optimize::run_optimize;
