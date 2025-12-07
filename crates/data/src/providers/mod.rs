//! Market data providers for fetching price history.
//!
//! This module provides different data sources for historical price data,
//! including API providers and file-based providers.

mod birdeye;
/// CSV provider module for file-based data loading.
pub mod csv_provider;
/// Jupiter Price API provider.
pub mod jupiter;
mod mock;

pub use birdeye::BirdeyeProvider;
pub use csv_provider::CsvProvider;
pub use jupiter::JupiterProvider;
pub use mock::MockMarketDataProvider;
