//! IL Limit rebalancing strategy.
//!
//! This strategy closes or rebalances the position when impermanent loss
//! exceeds a specified threshold, protecting against excessive losses.

use super::{RebalanceAction, RebalanceReason, RebalanceStrategy, StrategyContext};
use clmm_lp_domain::value_objects::price::Price;
use clmm_lp_domain::value_objects::price_range::PriceRange;
use rust_decimal::Decimal;

/// Strategy that triggers action when IL exceeds a threshold.
///
/// This is a risk management strategy that protects against excessive
/// impermanent loss by either rebalancing or closing the position.
#[derive(Debug, Clone)]
pub struct ILLimitStrategy {
    /// Maximum IL percentage before action (as decimal, e.g., 0.05 = 5%).
    max_il_pct: Decimal,
    /// Range width as percentage of current price for new positions.
    range_width_pct: Decimal,
    /// Whether to close instead of rebalance when IL limit is hit.
    close_on_limit: bool,
    /// Minimum steps before IL check is active (grace period).
    min_steps_before_check: u64,
    /// Whether to also rebalance when out of range.
    rebalance_on_out_of_range: bool,
}

impl ILLimitStrategy {
    /// Creates a new IL limit strategy.
    ///
    /// # Arguments
    /// * `max_il_pct` - Maximum IL percentage before action (e.g., 0.05 = 5%)
    /// * `range_width_pct` - Range width for new positions (e.g., 0.10 = 10%)
    #[must_use]
    pub fn new(max_il_pct: Decimal, range_width_pct: Decimal) -> Self {
        Self {
            max_il_pct,
            range_width_pct,
            close_on_limit: false,
            min_steps_before_check: 0,
            rebalance_on_out_of_range: true,
        }
    }

    /// Sets whether to close instead of rebalance when IL limit is hit.
    #[must_use]
    pub fn with_close_on_limit(mut self, close: bool) -> Self {
        self.close_on_limit = close;
        self
    }

    /// Sets minimum steps before IL check becomes active.
    #[must_use]
    pub fn with_grace_period(mut self, steps: u64) -> Self {
        self.min_steps_before_check = steps;
        self
    }

    /// Sets whether to rebalance when price goes out of range.
    #[must_use]
    pub fn with_rebalance_on_out_of_range(mut self, rebalance: bool) -> Self {
        self.rebalance_on_out_of_range = rebalance;
        self
    }

    /// Checks if IL exceeds the threshold.
    fn is_il_exceeded(&self, context: &StrategyContext) -> bool {
        // IL is typically negative, so we compare absolute value
        context.current_il_pct.abs() >= self.max_il_pct
    }

    /// Checks if the grace period has passed.
    fn is_grace_period_over(&self, context: &StrategyContext) -> bool {
        context.steps_since_open >= self.min_steps_before_check
    }
}

impl RebalanceStrategy for ILLimitStrategy {
    fn evaluate(&self, context: &StrategyContext) -> RebalanceAction {
        // Check if grace period has passed
        if !self.is_grace_period_over(context) {
            return RebalanceAction::Hold;
        }

        // Check IL threshold
        if self.is_il_exceeded(context) {
            if self.close_on_limit {
                return RebalanceAction::Close {
                    reason: RebalanceReason::ILThreshold {
                        il_pct: context.current_il_pct,
                    },
                };
            }
            let new_range = self.calculate_new_range(context.current_price, self.range_width_pct);
            return RebalanceAction::Rebalance {
                new_range,
                reason: RebalanceReason::ILThreshold {
                    il_pct: context.current_il_pct,
                },
            };
        }

        // Check if out of range
        if self.rebalance_on_out_of_range && !context.is_in_range() {
            let new_range = self.calculate_new_range(context.current_price, self.range_width_pct);
            return RebalanceAction::Rebalance {
                new_range,
                reason: RebalanceReason::OutOfRange {
                    current_price: context.current_price.value,
                },
            };
        }

        RebalanceAction::Hold
    }

    fn name(&self) -> &'static str {
        "IL Limit"
    }

    fn calculate_new_range(&self, current_price: Price, range_width_pct: Decimal) -> PriceRange {
        let half_width = current_price.value * range_width_pct / Decimal::from(2);
        PriceRange::new(
            Price::new(current_price.value - half_width),
            Price::new(current_price.value + half_width),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_context() -> StrategyContext {
        StrategyContext {
            current_price: Price::new(dec!(100)),
            current_range: PriceRange::new(Price::new(dec!(90)), Price::new(dec!(110))),
            entry_price: Price::new(dec!(100)),
            steps_since_open: 10,
            steps_since_rebalance: 5,
            current_il_pct: dec!(-0.02), // 2% IL
            total_fees_earned: dec!(50),
        }
    }

    #[test]
    fn test_il_limit_holds_when_below_threshold() {
        let strategy = ILLimitStrategy::new(dec!(0.05), dec!(0.10)); // 5% IL limit
        let context = create_test_context(); // 2% IL

        let action = strategy.evaluate(&context);
        assert_eq!(action, RebalanceAction::Hold);
    }

    #[test]
    fn test_il_limit_rebalances_when_exceeded() {
        let strategy = ILLimitStrategy::new(dec!(0.05), dec!(0.10));
        let mut context = create_test_context();
        context.current_il_pct = dec!(-0.06); // 6% IL exceeds 5% limit

        let action = strategy.evaluate(&context);
        match action {
            RebalanceAction::Rebalance { reason, .. } => {
                assert!(matches!(reason, RebalanceReason::ILThreshold { .. }));
            }
            _ => panic!("Expected Rebalance action"),
        }
    }

    #[test]
    fn test_il_limit_closes_when_configured() {
        let strategy = ILLimitStrategy::new(dec!(0.05), dec!(0.10)).with_close_on_limit(true);
        let mut context = create_test_context();
        context.current_il_pct = dec!(-0.06);

        let action = strategy.evaluate(&context);
        match action {
            RebalanceAction::Close { reason } => {
                assert!(matches!(reason, RebalanceReason::ILThreshold { .. }));
            }
            _ => panic!("Expected Close action"),
        }
    }

    #[test]
    fn test_il_limit_respects_grace_period() {
        let strategy = ILLimitStrategy::new(dec!(0.05), dec!(0.10)).with_grace_period(20);
        let mut context = create_test_context();
        context.current_il_pct = dec!(-0.10); // 10% IL
        context.steps_since_open = 10; // Only 10 steps, grace period is 20

        let action = strategy.evaluate(&context);
        assert_eq!(action, RebalanceAction::Hold);

        // After grace period
        context.steps_since_open = 25;
        let action = strategy.evaluate(&context);
        assert!(matches!(action, RebalanceAction::Rebalance { .. }));
    }

    #[test]
    fn test_il_limit_rebalances_on_out_of_range() {
        let strategy = ILLimitStrategy::new(dec!(0.10), dec!(0.10));
        let mut context = create_test_context();
        context.current_price = Price::new(dec!(120)); // Out of range

        let action = strategy.evaluate(&context);
        match action {
            RebalanceAction::Rebalance { reason, .. } => {
                assert!(matches!(reason, RebalanceReason::OutOfRange { .. }));
            }
            _ => panic!("Expected Rebalance action"),
        }
    }

    #[test]
    fn test_il_limit_ignores_out_of_range_when_disabled() {
        let strategy =
            ILLimitStrategy::new(dec!(0.10), dec!(0.10)).with_rebalance_on_out_of_range(false);
        let mut context = create_test_context();
        context.current_price = Price::new(dec!(120)); // Out of range

        let action = strategy.evaluate(&context);
        assert_eq!(action, RebalanceAction::Hold);
    }
}
