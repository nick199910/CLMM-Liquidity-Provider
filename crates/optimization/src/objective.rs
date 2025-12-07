use clmm_lp_domain::value_objects::simulation_result::SimulationResult;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::cmp::Ordering;

/// Trait for objective functions.
pub trait ObjectiveFunction {
    /// Evaluates the simulation result and returns a score.
    fn evaluate(&self, result: &SimulationResult) -> Decimal;

    /// Compares two simulation results.
    fn compare(&self, a: &SimulationResult, b: &SimulationResult) -> Ordering {
        self.evaluate(a)
            .partial_cmp(&self.evaluate(b))
            .unwrap_or(Ordering::Equal)
    }

    /// Returns the name of the objective function.
    fn name(&self) -> &'static str;
}

/// Objective function to maximize Net PnL.
#[derive(Debug, Clone, Copy, Default)]
pub struct MaximizeNetPnL;

impl ObjectiveFunction for MaximizeNetPnL {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        result.net_pnl
    }

    fn name(&self) -> &'static str {
        "MaximizeNetPnL"
    }
}

/// Objective function to maximize Fees.
#[derive(Debug, Clone, Copy, Default)]
pub struct MaximizeFees;

impl ObjectiveFunction for MaximizeFees {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        result.total_fees_earned
    }

    fn name(&self) -> &'static str {
        "MaximizeFees"
    }
}

/// Objective function to maximize Sharpe Ratio.
#[derive(Debug, Clone, Copy)]
pub struct MaximizeSharpeRatio {
    /// Risk-free rate.
    pub risk_free_rate: Decimal,
}

impl Default for MaximizeSharpeRatio {
    fn default() -> Self {
        Self {
            risk_free_rate: Decimal::ZERO,
        }
    }
}

impl MaximizeSharpeRatio {
    /// Creates a new MaximizeSharpeRatio objective.
    #[must_use]
    pub fn new(risk_free_rate: Decimal) -> Self {
        Self { risk_free_rate }
    }
}

impl ObjectiveFunction for MaximizeSharpeRatio {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        // Use sharpe_ratio if available, otherwise calculate from PnL/drawdown
        if let Some(sharpe) = result.sharpe_ratio {
            return sharpe;
        }

        // Simplified Sharpe: (Return - RiskFree) / Risk
        // Use max_drawdown as risk proxy
        if result.max_drawdown.is_zero() {
            return result.net_pnl - self.risk_free_rate;
        }

        (result.net_pnl - self.risk_free_rate) / result.max_drawdown
    }

    fn name(&self) -> &'static str {
        "MaximizeSharpeRatio"
    }
}

/// Objective function to minimize IL while maintaining minimum fees.
#[derive(Debug, Clone, Copy)]
pub struct MinimizeIL {
    /// Minimum acceptable fees.
    pub min_fees: Decimal,
}

impl Default for MinimizeIL {
    fn default() -> Self {
        Self {
            min_fees: Decimal::ZERO,
        }
    }
}

impl MinimizeIL {
    /// Creates a new MinimizeIL objective.
    #[must_use]
    pub fn new(min_fees: Decimal) -> Self {
        Self { min_fees }
    }
}

impl ObjectiveFunction for MinimizeIL {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        // Penalize if fees are below minimum
        if result.total_fees_earned < self.min_fees {
            return Decimal::MIN;
        }

        // Return negative IL (so minimizing IL = maximizing score)
        -result.total_il
    }

    fn name(&self) -> &'static str {
        "MinimizeIL"
    }
}

/// Objective function to maximize time in range.
#[derive(Debug, Clone, Copy, Default)]
pub struct MaximizeTimeInRange;

impl ObjectiveFunction for MaximizeTimeInRange {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        result.time_in_range_percentage
    }

    fn name(&self) -> &'static str {
        "MaximizeTimeInRange"
    }
}

/// Risk-adjusted return objective (Sortino-like).
#[derive(Debug, Clone, Copy)]
pub struct RiskAdjustedReturn {
    /// Weight for downside risk penalty.
    pub risk_weight: Decimal,
}

impl Default for RiskAdjustedReturn {
    fn default() -> Self {
        Self {
            risk_weight: Decimal::ONE,
        }
    }
}

impl RiskAdjustedReturn {
    /// Creates a new RiskAdjustedReturn objective.
    #[must_use]
    pub fn new(risk_weight: Decimal) -> Self {
        Self { risk_weight }
    }
}

impl ObjectiveFunction for RiskAdjustedReturn {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        // Score = PnL - (risk_weight * max_drawdown)
        result.net_pnl - (self.risk_weight * result.max_drawdown)
    }

    fn name(&self) -> &'static str {
        "RiskAdjustedReturn"
    }
}

/// Composite objective that combines multiple objectives with weights.
#[derive(Debug, Clone)]
pub struct CompositeObjective {
    /// Weights for each component.
    weights: CompositeWeights,
}

/// Weights for composite objective.
#[derive(Debug, Clone)]
pub struct CompositeWeights {
    /// Weight for net PnL.
    pub pnl_weight: Decimal,
    /// Weight for fees.
    pub fees_weight: Decimal,
    /// Weight for IL (negative = penalize).
    pub il_weight: Decimal,
    /// Weight for time in range.
    pub time_in_range_weight: Decimal,
    /// Weight for drawdown (negative = penalize).
    pub drawdown_weight: Decimal,
}

impl Default for CompositeWeights {
    fn default() -> Self {
        Self {
            pnl_weight: Decimal::ONE,
            fees_weight: Decimal::ZERO,
            il_weight: Decimal::from_f64(-0.5).unwrap(),
            time_in_range_weight: Decimal::ZERO,
            drawdown_weight: Decimal::from_f64(-0.3).unwrap(),
        }
    }
}

impl CompositeObjective {
    /// Creates a new composite objective with default weights.
    #[must_use]
    pub fn new() -> Self {
        Self {
            weights: CompositeWeights::default(),
        }
    }

    /// Creates a composite objective with custom weights.
    #[must_use]
    pub fn with_weights(weights: CompositeWeights) -> Self {
        Self { weights }
    }
}

impl Default for CompositeObjective {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectiveFunction for CompositeObjective {
    fn evaluate(&self, result: &SimulationResult) -> Decimal {
        let w = &self.weights;

        w.pnl_weight * result.net_pnl
            + w.fees_weight * result.total_fees_earned
            + w.il_weight * result.total_il
            + w.time_in_range_weight * result.time_in_range_percentage
            + w.drawdown_weight * result.max_drawdown
    }

    fn name(&self) -> &'static str {
        "CompositeObjective"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_result() -> SimulationResult {
        SimulationResult {
            final_position_value: Decimal::from(1100),
            total_fees_earned: Decimal::from(50),
            total_il: Decimal::from(20),
            net_pnl: Decimal::from(30),
            max_drawdown: Decimal::from(10),
            time_in_range_percentage: Decimal::from(75),
            sharpe_ratio: Some(Decimal::from(2)),
        }
    }

    #[test]
    fn test_maximize_net_pnl() {
        let obj = MaximizeNetPnL;
        let result = create_test_result();
        assert_eq!(obj.evaluate(&result), Decimal::from(30));
        assert_eq!(obj.name(), "MaximizeNetPnL");
    }

    #[test]
    fn test_maximize_fees() {
        let obj = MaximizeFees;
        let result = create_test_result();
        assert_eq!(obj.evaluate(&result), Decimal::from(50));
    }

    #[test]
    fn test_maximize_sharpe_ratio() {
        let obj = MaximizeSharpeRatio::default();
        let result = create_test_result();
        // Should use the sharpe_ratio field
        assert_eq!(obj.evaluate(&result), Decimal::from(2));
    }

    #[test]
    fn test_maximize_sharpe_ratio_fallback() {
        let obj = MaximizeSharpeRatio::default();
        let result = SimulationResult {
            final_position_value: Decimal::from(1100),
            total_fees_earned: Decimal::from(50),
            total_il: Decimal::from(20),
            net_pnl: Decimal::from(30),
            max_drawdown: Decimal::from(10),
            time_in_range_percentage: Decimal::from(75),
            sharpe_ratio: None,
        };
        // Should calculate: 30 / 10 = 3
        assert_eq!(obj.evaluate(&result), Decimal::from(3));
    }

    #[test]
    fn test_minimize_il() {
        let obj = MinimizeIL::new(Decimal::from(10));
        let result = create_test_result();
        // Fees (50) >= min_fees (10), so return -IL = -20
        assert_eq!(obj.evaluate(&result), Decimal::from(-20));
    }

    #[test]
    fn test_minimize_il_penalty() {
        let obj = MinimizeIL::new(Decimal::from(100));
        let result = create_test_result();
        // Fees (50) < min_fees (100), so return MIN
        assert_eq!(obj.evaluate(&result), Decimal::MIN);
    }

    #[test]
    fn test_maximize_time_in_range() {
        let obj = MaximizeTimeInRange;
        let result = create_test_result();
        assert_eq!(obj.evaluate(&result), Decimal::from(75));
    }

    #[test]
    fn test_risk_adjusted_return() {
        let obj = RiskAdjustedReturn::new(Decimal::from(2));
        let result = create_test_result();
        // 30 - (2 * 10) = 10
        assert_eq!(obj.evaluate(&result), Decimal::from(10));
    }

    #[test]
    fn test_composite_objective() {
        let obj = CompositeObjective::with_weights(CompositeWeights {
            pnl_weight: Decimal::ONE,
            fees_weight: Decimal::ZERO,
            il_weight: Decimal::ZERO,
            time_in_range_weight: Decimal::ZERO,
            drawdown_weight: Decimal::ZERO,
        });
        let result = create_test_result();
        // Only PnL with weight 1
        assert_eq!(obj.evaluate(&result), Decimal::from(30));
    }

    #[test]
    fn test_objective_compare() {
        let obj = MaximizeNetPnL;

        let result_a = SimulationResult {
            net_pnl: Decimal::from(30),
            ..create_test_result()
        };

        let result_b = SimulationResult {
            net_pnl: Decimal::from(20),
            ..create_test_result()
        };

        assert_eq!(obj.compare(&result_a, &result_b), Ordering::Greater);
        assert_eq!(obj.compare(&result_b, &result_a), Ordering::Less);
        assert_eq!(obj.compare(&result_a, &result_a), Ordering::Equal);
    }
}
