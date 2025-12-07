//! Main optimization logic.
//!
//! This module provides the core optimization algorithms including
//! grid search, and utilities for running optimization campaigns.

use crate::constraints::OptimizationConstraints;
use crate::objective::ObjectiveFunction;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::cmp::Ordering;

/// Result of a single optimization candidate evaluation.
#[derive(Debug, Clone)]
pub struct CandidateResult {
    /// The range width parameter.
    pub range_width: Decimal,
    /// Expected fees.
    pub expected_fees: Decimal,
    /// Expected IL.
    pub expected_il: Decimal,
    /// Net PnL.
    pub net_pnl: Decimal,
    /// Time in range percentage.
    pub time_in_range: Decimal,
    /// Objective score.
    pub score: Decimal,
}

impl CandidateResult {
    /// Creates a new candidate result.
    #[must_use]
    pub fn new(
        range_width: Decimal,
        expected_fees: Decimal,
        expected_il: Decimal,
        net_pnl: Decimal,
        time_in_range: Decimal,
        score: Decimal,
    ) -> Self {
        Self {
            range_width,
            expected_fees,
            expected_il,
            net_pnl,
            time_in_range,
            score,
        }
    }
}

/// Grid search optimizer for finding optimal parameters.
#[derive(Debug, Clone)]
pub struct GridSearchOptimizer {
    /// Grid of range widths to search.
    pub range_widths: Vec<Decimal>,
    /// Constraints to apply.
    pub constraints: OptimizationConstraints,
}

impl Default for GridSearchOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

impl GridSearchOptimizer {
    /// Creates a new grid search optimizer with default parameters.
    #[must_use]
    pub fn new() -> Self {
        // Default grid: 1%, 2%, 5%, 10%, 15%, 20%, 30%, 50%
        let widths = vec![0.01, 0.02, 0.05, 0.10, 0.15, 0.20, 0.30, 0.50];
        Self {
            range_widths: widths.into_iter().filter_map(Decimal::from_f64).collect(),
            constraints: OptimizationConstraints::default(),
        }
    }

    /// Creates a grid search optimizer with custom widths.
    #[must_use]
    pub fn with_widths(widths: Vec<Decimal>) -> Self {
        Self {
            range_widths: widths,
            constraints: OptimizationConstraints::default(),
        }
    }

    /// Sets the constraints.
    #[must_use]
    pub fn with_constraints(mut self, constraints: OptimizationConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    /// Generates a fine-grained grid between min and max.
    #[must_use]
    pub fn generate_grid(min: Decimal, max: Decimal, steps: usize) -> Vec<Decimal> {
        if steps < 2 {
            return vec![min];
        }

        let step_size = (max - min) / Decimal::from(steps - 1);
        (0..steps)
            .map(|i| min + step_size * Decimal::from(i))
            .collect()
    }

    /// Filters candidates based on constraints.
    #[must_use]
    pub fn filter_valid_widths(&self) -> Vec<Decimal> {
        self.range_widths
            .iter()
            .copied()
            .filter(|w| self.constraints.position.is_valid_range_width(*w))
            .collect()
    }

    /// Ranks candidates by score (descending).
    pub fn rank_candidates(candidates: &mut [CandidateResult]) {
        candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
    }

    /// Returns the top N candidates.
    #[must_use]
    pub fn top_n(candidates: &[CandidateResult], n: usize) -> Vec<CandidateResult> {
        candidates.iter().take(n).cloned().collect()
    }
}

/// Configuration for optimization runs.
#[derive(Debug, Clone)]
pub struct OptimizationConfig {
    /// Number of Monte Carlo iterations.
    pub mc_iterations: usize,
    /// Number of simulation steps.
    pub simulation_steps: usize,
    /// Time step in years.
    pub time_step_years: f64,
    /// Volatility assumption.
    pub volatility: f64,
    /// Drift assumption.
    pub drift: f64,
    /// Current price.
    pub current_price: Decimal,
    /// Pool liquidity.
    pub pool_liquidity: u128,
    /// Fee rate.
    pub fee_rate: Decimal,
    /// Transaction cost per rebalance.
    pub tx_cost: Decimal,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            mc_iterations: 100,
            simulation_steps: 30,
            time_step_years: 1.0 / 365.0, // 1 day
            volatility: 0.5,              // 50% annual
            drift: 0.0,
            current_price: Decimal::from(100),
            pool_liquidity: 1_000_000_000,
            fee_rate: Decimal::from_f64(0.003).unwrap(),
            tx_cost: Decimal::from_f64(0.001).unwrap(),
        }
    }
}

impl OptimizationConfig {
    /// Creates a new optimization config.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the Monte Carlo iterations.
    #[must_use]
    pub fn with_iterations(mut self, iterations: usize) -> Self {
        self.mc_iterations = iterations;
        self
    }

    /// Sets the simulation steps.
    #[must_use]
    pub fn with_steps(mut self, steps: usize) -> Self {
        self.simulation_steps = steps;
        self
    }

    /// Sets the volatility.
    #[must_use]
    pub fn with_volatility(mut self, volatility: f64) -> Self {
        self.volatility = volatility;
        self
    }

    /// Sets the current price.
    #[must_use]
    pub fn with_price(mut self, price: Decimal) -> Self {
        self.current_price = price;
        self
    }

    /// Sets the fee rate.
    #[must_use]
    pub fn with_fee_rate(mut self, fee_rate: Decimal) -> Self {
        self.fee_rate = fee_rate;
        self
    }
}

/// Trait for optimization algorithms.
pub trait Optimizer {
    /// Runs the optimization and returns ranked candidates.
    fn optimize<O: ObjectiveFunction>(
        &self,
        config: &OptimizationConfig,
        objective: &O,
    ) -> Vec<CandidateResult>;

    /// Returns the best candidate.
    fn best<O: ObjectiveFunction>(
        &self,
        config: &OptimizationConfig,
        objective: &O,
    ) -> Option<CandidateResult> {
        self.optimize(config, objective).into_iter().next()
    }
}

/// Simple analytical optimizer for quick estimates.
///
/// Uses simplified formulas instead of full Monte Carlo simulation.
#[derive(Debug, Clone, Default)]
pub struct AnalyticalOptimizer {
    /// Grid of range widths to evaluate.
    pub range_widths: Vec<Decimal>,
    /// Constraints.
    pub constraints: OptimizationConstraints,
}

impl AnalyticalOptimizer {
    /// Creates a new analytical optimizer.
    #[must_use]
    pub fn new() -> Self {
        let widths = vec![0.01, 0.02, 0.05, 0.10, 0.15, 0.20, 0.30, 0.50];
        Self {
            range_widths: widths.into_iter().filter_map(Decimal::from_f64).collect(),
            constraints: OptimizationConstraints::default(),
        }
    }

    /// Estimates fees for a given range width.
    ///
    /// Narrower ranges earn more fees when in range but are in range less often.
    #[must_use]
    pub fn estimate_fees(
        &self,
        width: Decimal,
        config: &OptimizationConfig,
        time_in_range: Decimal,
    ) -> Decimal {
        // Fee estimation: fees ∝ (1/width) * time_in_range * volume * fee_rate
        // Simplified: assume volume is proportional to pool_liquidity
        let volume_factor = Decimal::from(config.pool_liquidity) / Decimal::from(1_000_000_000u64);
        let width_factor = if width.is_zero() {
            Decimal::ONE
        } else {
            Decimal::ONE / width
        };

        let steps = Decimal::from(config.simulation_steps);
        let base_fees = config.fee_rate * volume_factor * steps;

        base_fees * width_factor * time_in_range / Decimal::from(100)
    }

    /// Estimates IL for a given range width and volatility.
    ///
    /// Narrower ranges have higher IL when price moves out of range.
    #[must_use]
    pub fn estimate_il(&self, width: Decimal, volatility: f64) -> Decimal {
        // IL estimation: IL ∝ volatility^2 / width
        // This is a simplified approximation
        let vol_dec = Decimal::from_f64(volatility).unwrap_or(Decimal::ZERO);
        let vol_squared = vol_dec * vol_dec;

        if width.is_zero() {
            return vol_squared;
        }

        // Scale IL by inverse of width (narrower = more IL)
        vol_squared / width / Decimal::from(10)
    }

    /// Estimates time in range for a given width and volatility.
    #[must_use]
    pub fn estimate_time_in_range(&self, width: Decimal, volatility: f64) -> Decimal {
        // Time in range estimation based on width and volatility
        // Wider range = more time in range
        // Higher volatility = less time in range
        let vol_factor = Decimal::from_f64(1.0 - volatility.min(0.9)).unwrap_or(Decimal::ONE);
        let width_factor = width * Decimal::from(2); // 10% width -> 20% factor

        let base_time = Decimal::from(50); // 50% base
        let time_in_range = base_time + (width_factor * Decimal::from(100) * vol_factor);

        time_in_range.min(Decimal::from(100))
    }
}

impl Optimizer for AnalyticalOptimizer {
    fn optimize<O: ObjectiveFunction>(
        &self,
        config: &OptimizationConfig,
        objective: &O,
    ) -> Vec<CandidateResult> {
        let mut candidates: Vec<CandidateResult> = self
            .range_widths
            .iter()
            .filter(|w| self.constraints.position.is_valid_range_width(**w))
            .map(|&width| {
                let time_in_range = self.estimate_time_in_range(width, config.volatility);
                let fees = self.estimate_fees(width, config, time_in_range);
                let il = self.estimate_il(width, config.volatility);
                let net_pnl = fees - il;

                // Create a mock SimulationResult for the objective function
                let sim_result =
                    clmm_lp_domain::value_objects::simulation_result::SimulationResult {
                        final_position_value: Decimal::ZERO,
                        total_fees_earned: fees,
                        total_il: il,
                        net_pnl,
                        max_drawdown: il,
                        time_in_range_percentage: time_in_range,
                        sharpe_ratio: None,
                    };

                let score = objective.evaluate(&sim_result);

                CandidateResult::new(width, fees, il, net_pnl, time_in_range, score)
            })
            .collect();

        GridSearchOptimizer::rank_candidates(&mut candidates);
        candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objective::MaximizeNetPnL;

    #[test]
    fn test_grid_search_optimizer_creation() {
        let optimizer = GridSearchOptimizer::new();
        assert!(!optimizer.range_widths.is_empty());
    }

    #[test]
    fn test_generate_grid() {
        let grid = GridSearchOptimizer::generate_grid(
            Decimal::from_f64(0.01).unwrap(),
            Decimal::from_f64(0.10).unwrap(),
            10,
        );
        assert_eq!(grid.len(), 10);
        assert_eq!(grid[0], Decimal::from_f64(0.01).unwrap());
        assert_eq!(grid[9], Decimal::from_f64(0.10).unwrap());
    }

    #[test]
    fn test_filter_valid_widths() {
        let optimizer = GridSearchOptimizer::new();
        let valid = optimizer.filter_valid_widths();

        // All default widths should be valid with default constraints
        assert!(!valid.is_empty());
        for w in &valid {
            assert!(optimizer.constraints.position.is_valid_range_width(*w));
        }
    }

    #[test]
    fn test_rank_candidates() {
        let mut candidates = vec![
            CandidateResult::new(
                Decimal::from_f64(0.10).unwrap(),
                Decimal::from(10),
                Decimal::from(2),
                Decimal::from(8),
                Decimal::from(70),
                Decimal::from(8),
            ),
            CandidateResult::new(
                Decimal::from_f64(0.05).unwrap(),
                Decimal::from(15),
                Decimal::from(5),
                Decimal::from(10),
                Decimal::from(60),
                Decimal::from(10),
            ),
            CandidateResult::new(
                Decimal::from_f64(0.20).unwrap(),
                Decimal::from(5),
                Decimal::from(1),
                Decimal::from(4),
                Decimal::from(80),
                Decimal::from(4),
            ),
        ];

        GridSearchOptimizer::rank_candidates(&mut candidates);

        assert_eq!(candidates[0].score, Decimal::from(10));
        assert_eq!(candidates[1].score, Decimal::from(8));
        assert_eq!(candidates[2].score, Decimal::from(4));
    }

    #[test]
    fn test_analytical_optimizer() {
        let optimizer = AnalyticalOptimizer::new();
        let config = OptimizationConfig::new()
            .with_iterations(10)
            .with_steps(30)
            .with_volatility(0.3);

        let candidates = optimizer.optimize(&config, &MaximizeNetPnL);

        assert!(!candidates.is_empty());
        // Candidates should be sorted by score descending
        for i in 1..candidates.len() {
            assert!(candidates[i - 1].score >= candidates[i].score);
        }
    }

    #[test]
    fn test_analytical_optimizer_best() {
        let optimizer = AnalyticalOptimizer::new();
        let config = OptimizationConfig::default();

        let best = optimizer.best(&config, &MaximizeNetPnL);
        assert!(best.is_some());
    }

    #[test]
    fn test_optimization_config_builder() {
        let config = OptimizationConfig::new()
            .with_iterations(50)
            .with_steps(60)
            .with_volatility(0.4)
            .with_price(Decimal::from(150));

        assert_eq!(config.mc_iterations, 50);
        assert_eq!(config.simulation_steps, 60);
        assert_eq!(config.volatility, 0.4);
        assert_eq!(config.current_price, Decimal::from(150));
    }
}
