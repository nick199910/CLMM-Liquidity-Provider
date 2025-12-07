//! Decision engine for strategy execution.

use super::Decision;
use crate::monitor::MonitoredPosition;
use clmm_lp_protocols::prelude::WhirlpoolState;
use rust_decimal::Decimal;
use tracing::debug;

/// Configuration for the decision engine.
#[derive(Debug, Clone)]
pub struct DecisionConfig {
    /// IL threshold for rebalancing (as percentage).
    pub il_rebalance_threshold: Decimal,
    /// IL threshold for closing (as percentage).
    pub il_close_threshold: Decimal,
    /// Minimum time between rebalances in hours.
    pub min_rebalance_interval_hours: u64,
    /// Range width for new positions (as percentage).
    pub range_width_pct: Decimal,
    /// Whether to auto-collect fees.
    pub auto_collect_fees: bool,
    /// Minimum fees to collect in USD.
    pub min_fees_to_collect: Decimal,
}

impl Default for DecisionConfig {
    fn default() -> Self {
        Self {
            il_rebalance_threshold: Decimal::new(5, 2), // 5%
            il_close_threshold: Decimal::new(15, 2),    // 15%
            min_rebalance_interval_hours: 24,
            range_width_pct: Decimal::new(10, 2), // 10%
            auto_collect_fees: true,
            min_fees_to_collect: Decimal::new(10, 0), // $10
        }
    }
}

/// Context for making decisions.
#[derive(Debug, Clone)]
pub struct DecisionContext {
    /// Current position state.
    pub position: MonitoredPosition,
    /// Current pool state.
    pub pool: WhirlpoolState,
    /// Hours since last rebalance.
    pub hours_since_rebalance: u64,
}

/// Decision engine for automated strategy execution.
pub struct DecisionEngine {
    /// Configuration.
    config: DecisionConfig,
}

impl DecisionEngine {
    /// Creates a new decision engine.
    #[must_use]
    pub fn new(config: DecisionConfig) -> Self {
        Self { config }
    }

    /// Makes a decision for a position.
    pub fn decide(&self, context: &DecisionContext) -> Decision {
        let position = &context.position;
        let pool = &context.pool;

        debug!(
            position = %position.address,
            in_range = position.in_range,
            il_pct = %position.pnl.il_pct,
            "Evaluating position"
        );

        // Check for critical IL - close position
        if position.pnl.il_pct.abs() > self.config.il_close_threshold {
            debug!("IL exceeds close threshold, recommending close");
            return Decision::Close;
        }

        // Check for fee collection
        if self.config.auto_collect_fees && position.pnl.fees_usd > self.config.min_fees_to_collect
        {
            debug!("Fees exceed threshold, recommending collection");
            return Decision::CollectFees;
        }

        // Check if out of range
        if !position.in_range {
            // Check if enough time has passed since last rebalance
            if context.hours_since_rebalance >= self.config.min_rebalance_interval_hours {
                let (new_lower, new_upper) = self.calculate_new_range(pool);
                debug!(
                    new_lower = new_lower,
                    new_upper = new_upper,
                    "Position out of range, recommending rebalance"
                );
                return Decision::Rebalance {
                    new_tick_lower: new_lower,
                    new_tick_upper: new_upper,
                };
            }
        }

        // Check for IL-based rebalancing
        if position.pnl.il_pct.abs() > self.config.il_rebalance_threshold
            && context.hours_since_rebalance >= self.config.min_rebalance_interval_hours
        {
            let (new_lower, new_upper) = self.calculate_new_range(pool);
            debug!(
                il_pct = %position.pnl.il_pct,
                "IL exceeds threshold, recommending rebalance"
            );
            return Decision::Rebalance {
                new_tick_lower: new_lower,
                new_tick_upper: new_upper,
            };
        }

        // Default: hold
        Decision::Hold
    }

    /// Calculates a new range centered on current price.
    fn calculate_new_range(&self, pool: &WhirlpoolState) -> (i32, i32) {
        clmm_lp_protocols::prelude::calculate_tick_range(
            pool.tick_current,
            self.config.range_width_pct,
            pool.tick_spacing,
        )
    }

    /// Updates the configuration.
    pub fn set_config(&mut self, config: DecisionConfig) {
        self.config = config;
    }

    /// Gets the current configuration.
    #[must_use]
    pub fn config(&self) -> &DecisionConfig {
        &self.config
    }
}

impl Default for DecisionEngine {
    fn default() -> Self {
        Self::new(DecisionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::PositionPnL;
    use solana_sdk::pubkey::Pubkey;

    fn create_test_context(in_range: bool, il_pct: Decimal) -> DecisionContext {
        let position = MonitoredPosition {
            address: Pubkey::new_unique(),
            pool: Pubkey::new_unique(),
            on_chain: clmm_lp_protocols::prelude::OnChainPosition {
                address: Pubkey::new_unique(),
                pool: Pubkey::new_unique(),
                owner: Pubkey::new_unique(),
                tick_lower: -1000,
                tick_upper: 1000,
                liquidity: 1000000,
                fee_growth_inside_a: 0,
                fee_growth_inside_b: 0,
                fees_owed_a: 0,
                fees_owed_b: 0,
            },
            pnl: PositionPnL {
                il_pct,
                ..Default::default()
            },
            in_range,
            last_updated: chrono::Utc::now(),
        };

        let pool = WhirlpoolState {
            address: String::new(),
            token_mint_a: Pubkey::new_unique(),
            token_mint_b: Pubkey::new_unique(),
            tick_current: 0,
            tick_spacing: 64,
            sqrt_price: 1 << 64,
            price: Decimal::ONE,
            liquidity: 1000000,
            fee_rate_bps: 30,
            protocol_fee_rate_bps: 0,
            fee_growth_global_a: 0,
            fee_growth_global_b: 0,
        };

        DecisionContext {
            position,
            pool,
            hours_since_rebalance: 48,
        }
    }

    #[test]
    fn test_hold_decision() {
        let engine = DecisionEngine::default();
        let context = create_test_context(true, Decimal::ZERO);

        let decision = engine.decide(&context);
        assert!(matches!(decision, Decision::Hold));
    }

    #[test]
    fn test_rebalance_on_range_exit() {
        let engine = DecisionEngine::default();
        let context = create_test_context(false, Decimal::ZERO);

        let decision = engine.decide(&context);
        assert!(matches!(decision, Decision::Rebalance { .. }));
    }

    #[test]
    fn test_close_on_high_il() {
        let engine = DecisionEngine::default();
        let context = create_test_context(true, Decimal::new(20, 2)); // 20% IL

        let decision = engine.decide(&context);
        assert!(matches!(decision, Decision::Close));
    }
}
