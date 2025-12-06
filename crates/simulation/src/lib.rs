//! Simulation engine for backtesting strategies.
/// Simulation engine implementation.
pub mod engine;
/// Event definitions.
pub mod event;
/// Monte Carlo simulation logic.
pub mod monte_carlo;
/// Position simulation logic.
pub mod position_simulator;
/// Position tracking logic.
pub mod position_tracker;
/// Price path generation.
pub mod price_path;
/// Simulation state management.
pub mod state;
/// Rebalancing strategies.
pub mod strategies;
/// Strategy simulation logic.
pub mod strategy_simulator;
/// Volume modeling.
pub mod volume;
