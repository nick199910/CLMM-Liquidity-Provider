//! Repository implementations for database persistence.
//!
//! This module provides repository patterns for storing and retrieving
//! simulation data, pool configurations, and price history.

mod database;
mod pool_repository;
mod price_repository;
mod simulation_repository;

pub use database::Database;
pub use pool_repository::{PoolRecord, PoolRepository};
pub use price_repository::{PriceRecord, PriceRepository};
pub use simulation_repository::{
    OptimizationRecord, SimulationRecord, SimulationRepository, SimulationResultRecord,
};
