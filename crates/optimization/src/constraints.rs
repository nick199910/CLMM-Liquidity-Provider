//! Position constraints for optimization.
//!
//! This module defines constraints that limit the search space
//! during optimization, ensuring valid and practical solutions.

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

/// Constraints for position optimization.
#[derive(Debug, Clone)]
pub struct PositionConstraints {
    /// Minimum range width as a percentage (e.g., 0.01 = 1%).
    pub min_range_width: Decimal,
    /// Maximum range width as a percentage (e.g., 0.50 = 50%).
    pub max_range_width: Decimal,
    /// Minimum capital to deploy.
    pub min_capital: Decimal,
    /// Maximum capital to deploy.
    pub max_capital: Decimal,
    /// Maximum acceptable IL as a percentage.
    pub max_il_threshold: Decimal,
    /// Minimum time in range percentage.
    pub min_time_in_range: Decimal,
    /// Maximum transaction cost as percentage of capital.
    pub max_tx_cost_ratio: Decimal,
}

impl Default for PositionConstraints {
    fn default() -> Self {
        Self {
            min_range_width: Decimal::from_f64(0.01).unwrap(), // 1%
            max_range_width: Decimal::from_f64(0.50).unwrap(), // 50%
            min_capital: Decimal::from(100),                   // $100
            max_capital: Decimal::from(1_000_000),             // $1M
            max_il_threshold: Decimal::from_f64(0.10).unwrap(), // 10%
            min_time_in_range: Decimal::from_f64(0.50).unwrap(), // 50%
            max_tx_cost_ratio: Decimal::from_f64(0.01).unwrap(), // 1%
        }
    }
}

impl PositionConstraints {
    /// Creates new constraints with custom values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the minimum range width.
    #[must_use]
    pub fn with_min_range_width(mut self, width: Decimal) -> Self {
        self.min_range_width = width;
        self
    }

    /// Sets the maximum range width.
    #[must_use]
    pub fn with_max_range_width(mut self, width: Decimal) -> Self {
        self.max_range_width = width;
        self
    }

    /// Sets the capital bounds.
    #[must_use]
    pub fn with_capital_bounds(mut self, min: Decimal, max: Decimal) -> Self {
        self.min_capital = min;
        self.max_capital = max;
        self
    }

    /// Sets the maximum IL threshold.
    #[must_use]
    pub fn with_max_il(mut self, max_il: Decimal) -> Self {
        self.max_il_threshold = max_il;
        self
    }

    /// Sets the minimum time in range.
    #[must_use]
    pub fn with_min_time_in_range(mut self, min_time: Decimal) -> Self {
        self.min_time_in_range = min_time;
        self
    }

    /// Checks if a range width is valid.
    #[must_use]
    pub fn is_valid_range_width(&self, width: Decimal) -> bool {
        width >= self.min_range_width && width <= self.max_range_width
    }

    /// Checks if capital is within bounds.
    #[must_use]
    pub fn is_valid_capital(&self, capital: Decimal) -> bool {
        capital >= self.min_capital && capital <= self.max_capital
    }

    /// Checks if IL is acceptable.
    #[must_use]
    pub fn is_acceptable_il(&self, il: Decimal) -> bool {
        il.abs() <= self.max_il_threshold
    }

    /// Checks if time in range meets minimum.
    #[must_use]
    pub fn meets_time_in_range(&self, time_in_range: Decimal) -> bool {
        time_in_range >= self.min_time_in_range
    }
}

/// Constraints for rebalancing strategy optimization.
#[derive(Debug, Clone)]
pub struct RebalanceConstraints {
    /// Minimum rebalance interval in steps.
    pub min_rebalance_interval: u64,
    /// Maximum rebalance interval in steps.
    pub max_rebalance_interval: u64,
    /// Minimum price deviation threshold for rebalancing.
    pub min_price_threshold: Decimal,
    /// Maximum price deviation threshold for rebalancing.
    pub max_price_threshold: Decimal,
    /// Minimum IL threshold for rebalancing.
    pub min_il_threshold: Decimal,
    /// Maximum IL threshold for rebalancing.
    pub max_il_threshold: Decimal,
    /// Maximum number of rebalances allowed.
    pub max_rebalances: Option<u32>,
}

impl Default for RebalanceConstraints {
    fn default() -> Self {
        Self {
            min_rebalance_interval: 1,
            max_rebalance_interval: 168, // 1 week in hours
            min_price_threshold: Decimal::from_f64(0.01).unwrap(), // 1%
            max_price_threshold: Decimal::from_f64(0.20).unwrap(), // 20%
            min_il_threshold: Decimal::from_f64(0.01).unwrap(), // 1%
            max_il_threshold: Decimal::from_f64(0.15).unwrap(), // 15%
            max_rebalances: None,
        }
    }
}

impl RebalanceConstraints {
    /// Creates new rebalance constraints.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the rebalance interval bounds.
    #[must_use]
    pub fn with_interval_bounds(mut self, min: u64, max: u64) -> Self {
        self.min_rebalance_interval = min;
        self.max_rebalance_interval = max;
        self
    }

    /// Sets the price threshold bounds.
    #[must_use]
    pub fn with_price_threshold_bounds(mut self, min: Decimal, max: Decimal) -> Self {
        self.min_price_threshold = min;
        self.max_price_threshold = max;
        self
    }

    /// Sets the IL threshold bounds.
    #[must_use]
    pub fn with_il_threshold_bounds(mut self, min: Decimal, max: Decimal) -> Self {
        self.min_il_threshold = min;
        self.max_il_threshold = max;
        self
    }

    /// Sets the maximum number of rebalances.
    #[must_use]
    pub fn with_max_rebalances(mut self, max: u32) -> Self {
        self.max_rebalances = Some(max);
        self
    }

    /// Checks if an interval is valid.
    #[must_use]
    pub fn is_valid_interval(&self, interval: u64) -> bool {
        interval >= self.min_rebalance_interval && interval <= self.max_rebalance_interval
    }

    /// Checks if a price threshold is valid.
    #[must_use]
    pub fn is_valid_price_threshold(&self, threshold: Decimal) -> bool {
        threshold >= self.min_price_threshold && threshold <= self.max_price_threshold
    }

    /// Checks if an IL threshold is valid.
    #[must_use]
    pub fn is_valid_il_threshold(&self, threshold: Decimal) -> bool {
        threshold >= self.min_il_threshold && threshold <= self.max_il_threshold
    }
}

/// Combined constraints for full optimization.
#[derive(Debug, Clone, Default)]
pub struct OptimizationConstraints {
    /// Position constraints.
    pub position: PositionConstraints,
    /// Rebalance constraints.
    pub rebalance: RebalanceConstraints,
}

impl OptimizationConstraints {
    /// Creates new optimization constraints.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets position constraints.
    #[must_use]
    pub fn with_position(mut self, constraints: PositionConstraints) -> Self {
        self.position = constraints;
        self
    }

    /// Sets rebalance constraints.
    #[must_use]
    pub fn with_rebalance(mut self, constraints: RebalanceConstraints) -> Self {
        self.rebalance = constraints;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_constraints_default() {
        let constraints = PositionConstraints::default();
        assert_eq!(
            constraints.min_range_width,
            Decimal::from_f64(0.01).unwrap()
        );
        assert_eq!(
            constraints.max_range_width,
            Decimal::from_f64(0.50).unwrap()
        );
    }

    #[test]
    fn test_position_constraints_validation() {
        let constraints = PositionConstraints::default();

        assert!(constraints.is_valid_range_width(Decimal::from_f64(0.10).unwrap()));
        assert!(!constraints.is_valid_range_width(Decimal::from_f64(0.005).unwrap()));
        assert!(!constraints.is_valid_range_width(Decimal::from_f64(0.60).unwrap()));

        assert!(constraints.is_valid_capital(Decimal::from(1000)));
        assert!(!constraints.is_valid_capital(Decimal::from(50)));

        assert!(constraints.is_acceptable_il(Decimal::from_f64(0.05).unwrap()));
        assert!(!constraints.is_acceptable_il(Decimal::from_f64(0.15).unwrap()));
    }

    #[test]
    fn test_rebalance_constraints_validation() {
        let constraints = RebalanceConstraints::default();

        assert!(constraints.is_valid_interval(24));
        assert!(!constraints.is_valid_interval(0));
        assert!(!constraints.is_valid_interval(200));

        assert!(constraints.is_valid_price_threshold(Decimal::from_f64(0.05).unwrap()));
        assert!(!constraints.is_valid_price_threshold(Decimal::from_f64(0.005).unwrap()));
    }

    #[test]
    fn test_constraints_builder() {
        let constraints = PositionConstraints::new()
            .with_min_range_width(Decimal::from_f64(0.02).unwrap())
            .with_max_il(Decimal::from_f64(0.05).unwrap());

        assert_eq!(
            constraints.min_range_width,
            Decimal::from_f64(0.02).unwrap()
        );
        assert_eq!(
            constraints.max_il_threshold,
            Decimal::from_f64(0.05).unwrap()
        );
    }
}
