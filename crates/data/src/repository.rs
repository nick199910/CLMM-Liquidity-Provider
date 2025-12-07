//! In-memory data repository for simulation.
//!
//! This module provides a unified in-memory storage for market data,
//! pool states, and simulation results.

use crate::pool_state::{PoolStateHistory, PoolStateSnapshot};
use crate::timeseries::{OhlcvCandle, TimeSeries};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// In-memory repository for simulation data.
#[derive(Debug, Default)]
pub struct SimulationDataRepository {
    /// Price time series indexed by token pair (e.g., "SOL/USDC").
    price_series: HashMap<String, TimeSeries>,
    /// Pool state histories indexed by pool ID.
    pool_histories: HashMap<String, PoolStateHistory>,
}

impl SimulationDataRepository {
    /// Creates a new empty repository.
    #[must_use]
    pub fn new() -> Self {
        Self {
            price_series: HashMap::new(),
            pool_histories: HashMap::new(),
        }
    }

    // ========== Price Series Methods ==========

    /// Adds a price time series for a token pair.
    pub fn add_price_series(&mut self, pair: String, series: TimeSeries) {
        self.price_series.insert(pair, series);
    }

    /// Gets a price time series for a token pair.
    #[must_use]
    pub fn get_price_series(&self, pair: &str) -> Option<&TimeSeries> {
        self.price_series.get(pair)
    }

    /// Gets a mutable price time series for a token pair.
    pub fn get_price_series_mut(&mut self, pair: &str) -> Option<&mut TimeSeries> {
        self.price_series.get_mut(pair)
    }

    /// Adds a single candle to a price series.
    pub fn add_candle(&mut self, pair: &str, candle: OhlcvCandle) {
        if let Some(series) = self.price_series.get_mut(pair) {
            series.insert(candle);
        } else {
            let mut series = TimeSeries::new(3600); // Default 1-hour interval
            series.insert(candle);
            self.price_series.insert(pair.to_string(), series);
        }
    }

    /// Gets the price at a specific timestamp for a pair.
    #[must_use]
    pub fn get_price_at(&self, pair: &str, timestamp: u64) -> Option<Decimal> {
        self.price_series
            .get(pair)
            .and_then(|s| s.interpolate_price(timestamp))
    }

    /// Gets prices in a time range for a pair.
    #[must_use]
    pub fn get_prices_in_range(&self, pair: &str, from: u64, to: u64) -> Vec<Decimal> {
        self.price_series
            .get(pair)
            .map(|s| s.range(from, to).iter().map(|c| c.close).collect())
            .unwrap_or_default()
    }

    /// Returns all available token pairs.
    #[must_use]
    pub fn available_pairs(&self) -> Vec<&String> {
        self.price_series.keys().collect()
    }

    // ========== Pool History Methods ==========

    /// Adds a pool state history.
    pub fn add_pool_history(&mut self, history: PoolStateHistory) {
        self.pool_histories.insert(history.pool_id.clone(), history);
    }

    /// Gets a pool state history by pool ID.
    #[must_use]
    pub fn get_pool_history(&self, pool_id: &str) -> Option<&PoolStateHistory> {
        self.pool_histories.get(pool_id)
    }

    /// Gets a mutable pool state history by pool ID.
    pub fn get_pool_history_mut(&mut self, pool_id: &str) -> Option<&mut PoolStateHistory> {
        self.pool_histories.get_mut(pool_id)
    }

    /// Adds a single pool state snapshot.
    pub fn add_pool_snapshot(&mut self, pool_id: &str, snapshot: PoolStateSnapshot) {
        if let Some(history) = self.pool_histories.get_mut(pool_id) {
            history.insert(snapshot);
        } else {
            let mut history = PoolStateHistory::new(pool_id.to_string());
            history.insert(snapshot);
            self.pool_histories.insert(pool_id.to_string(), history);
        }
    }

    /// Gets the pool state at a specific timestamp.
    #[must_use]
    pub fn get_pool_state_at(&self, pool_id: &str, timestamp: u64) -> Option<&PoolStateSnapshot> {
        self.pool_histories
            .get(pool_id)
            .and_then(|h| h.get_at_or_before(timestamp))
    }

    /// Gets pool states in a time range.
    #[must_use]
    pub fn get_pool_states_in_range(
        &self,
        pool_id: &str,
        from: u64,
        to: u64,
    ) -> Vec<&PoolStateSnapshot> {
        self.pool_histories
            .get(pool_id)
            .map(|h| h.range(from, to))
            .unwrap_or_default()
    }

    /// Returns all available pool IDs.
    #[must_use]
    pub fn available_pools(&self) -> Vec<&String> {
        self.pool_histories.keys().collect()
    }

    // ========== Utility Methods ==========

    /// Clears all data from the repository.
    pub fn clear(&mut self) {
        self.price_series.clear();
        self.pool_histories.clear();
    }

    /// Returns the total number of data points stored.
    #[must_use]
    pub fn total_data_points(&self) -> usize {
        let price_points: usize = self.price_series.values().map(|s| s.len()).sum();
        let pool_points: usize = self.pool_histories.values().map(|h| h.len()).sum();
        price_points + pool_points
    }

    /// Returns the time range covered by all data.
    #[must_use]
    pub fn global_time_range(&self) -> Option<(u64, u64)> {
        let mut min_time: Option<u64> = None;
        let mut max_time: Option<u64> = None;

        for series in self.price_series.values() {
            if let Some(start) = series.start_time() {
                min_time = Some(min_time.map_or(start, |m| m.min(start)));
            }
            if let Some(end) = series.end_time() {
                max_time = Some(max_time.map_or(end, |m| m.max(end)));
            }
        }

        for history in self.pool_histories.values() {
            if let Some((start, end)) = history.time_range() {
                min_time = Some(min_time.map_or(start, |m| m.min(start)));
                max_time = Some(max_time.map_or(end, |m| m.max(end)));
            }
        }

        match (min_time, max_time) {
            (Some(min), Some(max)) => Some((min, max)),
            _ => None,
        }
    }
}

/// Builder for creating simulation data repositories with fluent API.
#[derive(Debug, Default)]
pub struct SimulationDataRepositoryBuilder {
    repo: SimulationDataRepository,
}

impl SimulationDataRepositoryBuilder {
    /// Creates a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            repo: SimulationDataRepository::new(),
        }
    }

    /// Adds a price time series.
    #[must_use]
    pub fn with_price_series(mut self, pair: String, series: TimeSeries) -> Self {
        self.repo.add_price_series(pair, series);
        self
    }

    /// Adds a pool state history.
    #[must_use]
    pub fn with_pool_history(mut self, history: PoolStateHistory) -> Self {
        self.repo.add_pool_history(history);
        self
    }

    /// Builds the repository.
    #[must_use]
    pub fn build(self) -> SimulationDataRepository {
        self.repo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_series() -> TimeSeries {
        let candles = vec![
            OhlcvCandle::new(1000, dec!(100), dec!(105), dec!(98), dec!(102), dec!(1000)),
            OhlcvCandle::new(2000, dec!(102), dec!(108), dec!(101), dec!(106), dec!(1200)),
            OhlcvCandle::new(3000, dec!(106), dec!(110), dec!(104), dec!(108), dec!(1100)),
        ];
        TimeSeries::from_candles(candles, 1000)
    }

    fn create_test_pool_history() -> PoolStateHistory {
        let snapshots = vec![
            PoolStateSnapshot::new(
                1000,
                dec!(100),
                1_000_000,
                dec!(10000),
                dec!(1000000),
                dec!(0.003),
            ),
            PoolStateSnapshot::new(
                2000,
                dec!(105),
                1_100_000,
                dec!(9500),
                dec!(997500),
                dec!(0.003),
            ),
            PoolStateSnapshot::new(
                3000,
                dec!(102),
                1_050_000,
                dec!(9800),
                dec!(999600),
                dec!(0.003),
            ),
        ];
        PoolStateHistory::from_snapshots("pool1".to_string(), snapshots)
    }

    #[test]
    fn test_repository_price_series() {
        let mut repo = SimulationDataRepository::new();
        repo.add_price_series("SOL/USDC".to_string(), create_test_series());

        assert!(repo.get_price_series("SOL/USDC").is_some());
        assert!(repo.get_price_series("ETH/USDC").is_none());

        let price = repo.get_price_at("SOL/USDC", 2000);
        assert_eq!(price, Some(dec!(106)));
    }

    #[test]
    fn test_repository_pool_history() {
        let mut repo = SimulationDataRepository::new();
        repo.add_pool_history(create_test_pool_history());

        assert!(repo.get_pool_history("pool1").is_some());
        assert!(repo.get_pool_history("pool2").is_none());

        let state = repo.get_pool_state_at("pool1", 2500);
        assert!(state.is_some());
        assert_eq!(state.unwrap().timestamp, 2000);
    }

    #[test]
    fn test_repository_add_candle() {
        let mut repo = SimulationDataRepository::new();

        // Add to non-existent series (should create it)
        repo.add_candle(
            "BTC/USDC",
            OhlcvCandle::new(
                1000,
                dec!(50000),
                dec!(51000),
                dec!(49000),
                dec!(50500),
                dec!(100),
            ),
        );

        assert!(repo.get_price_series("BTC/USDC").is_some());
        assert_eq!(repo.get_price_series("BTC/USDC").unwrap().len(), 1);
    }

    #[test]
    fn test_repository_builder() {
        let repo = SimulationDataRepositoryBuilder::new()
            .with_price_series("SOL/USDC".to_string(), create_test_series())
            .with_pool_history(create_test_pool_history())
            .build();

        assert_eq!(repo.available_pairs().len(), 1);
        assert_eq!(repo.available_pools().len(), 1);
    }

    #[test]
    fn test_repository_global_time_range() {
        let repo = SimulationDataRepositoryBuilder::new()
            .with_price_series("SOL/USDC".to_string(), create_test_series())
            .with_pool_history(create_test_pool_history())
            .build();

        let range = repo.global_time_range();
        assert_eq!(range, Some((1000, 3000)));
    }

    #[test]
    fn test_repository_total_data_points() {
        let repo = SimulationDataRepositoryBuilder::new()
            .with_price_series("SOL/USDC".to_string(), create_test_series())
            .with_pool_history(create_test_pool_history())
            .build();

        // 3 candles + 3 pool snapshots = 6
        assert_eq!(repo.total_data_points(), 6);
    }
}
