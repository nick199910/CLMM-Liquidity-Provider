//! Strategy executor for automated position management.

use super::{Decision, DecisionContext, DecisionEngine};
use crate::monitor::PositionMonitor;
use crate::transaction::TransactionManager;
use crate::wallet::Wallet;
use clmm_lp_protocols::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Configuration for strategy execution.
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Evaluation interval in seconds.
    pub eval_interval_secs: u64,
    /// Whether to execute decisions automatically.
    pub auto_execute: bool,
    /// Whether to require confirmation before executing.
    pub require_confirmation: bool,
    /// Maximum slippage tolerance (as percentage).
    pub max_slippage_pct: rust_decimal::Decimal,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            eval_interval_secs: 300, // 5 minutes
            auto_execute: false,     // Require manual confirmation by default
            require_confirmation: true,
            max_slippage_pct: rust_decimal::Decimal::new(5, 3), // 0.5%
        }
    }
}

/// Strategy executor for automated position management.
pub struct StrategyExecutor {
    /// Position monitor.
    monitor: Arc<PositionMonitor>,
    /// Decision engine.
    decision_engine: DecisionEngine,
    /// Transaction manager.
    #[allow(dead_code)]
    tx_manager: Arc<TransactionManager>,
    /// Wallet for signing.
    #[allow(dead_code)]
    wallet: Option<Arc<Wallet>>,
    /// Configuration.
    config: ExecutorConfig,
    /// Running flag.
    running: std::sync::atomic::AtomicBool,
}

impl StrategyExecutor {
    /// Creates a new strategy executor.
    pub fn new(
        monitor: Arc<PositionMonitor>,
        tx_manager: Arc<TransactionManager>,
        config: ExecutorConfig,
    ) -> Self {
        Self {
            monitor,
            decision_engine: DecisionEngine::default(),
            tx_manager,
            wallet: None,
            config,
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Sets the wallet for signing transactions.
    pub fn set_wallet(&mut self, wallet: Arc<Wallet>) {
        self.wallet = Some(wallet);
    }

    /// Sets the decision engine configuration.
    pub fn set_decision_config(&mut self, config: super::DecisionConfig) {
        self.decision_engine.set_config(config);
    }

    /// Starts the strategy execution loop.
    pub async fn start(&self) {
        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let eval_interval = Duration::from_secs(self.config.eval_interval_secs);
        let mut ticker = interval(eval_interval);

        info!(
            interval_secs = self.config.eval_interval_secs,
            auto_execute = self.config.auto_execute,
            "Starting strategy executor"
        );

        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            ticker.tick().await;

            if let Err(e) = self.evaluate_all().await {
                error!(error = %e, "Strategy evaluation failed");
            }
        }

        info!("Strategy executor stopped");
    }

    /// Stops the strategy execution loop.
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Evaluates all monitored positions.
    async fn evaluate_all(&self) -> anyhow::Result<()> {
        let positions = self.monitor.get_positions().await;

        debug!(count = positions.len(), "Evaluating positions");

        for position in positions {
            if let Err(e) = self.evaluate_position(&position).await {
                warn!(
                    position = %position.address,
                    error = %e,
                    "Failed to evaluate position"
                );
            }
        }

        Ok(())
    }

    /// Evaluates a single position.
    async fn evaluate_position(
        &self,
        position: &crate::monitor::MonitoredPosition,
    ) -> anyhow::Result<()> {
        // Create decision context
        // Note: In a real implementation, we would fetch the pool state
        let context = DecisionContext {
            position: position.clone(),
            pool: WhirlpoolState {
                address: position.pool.to_string(),
                token_mint_a: solana_sdk::pubkey::Pubkey::default(),
                token_mint_b: solana_sdk::pubkey::Pubkey::default(),
                tick_current: 0,
                tick_spacing: 64,
                sqrt_price: 1 << 64,
                price: rust_decimal::Decimal::ONE,
                liquidity: 0,
                fee_rate_bps: 30,
                protocol_fee_rate_bps: 0,
                fee_growth_global_a: 0,
                fee_growth_global_b: 0,
            },
            hours_since_rebalance: 0, // TODO: Track this
        };

        let decision = self.decision_engine.decide(&context);

        if decision.requires_transaction() {
            info!(
                position = %position.address,
                decision = %decision.description(),
                "Decision requires action"
            );

            if self.config.auto_execute {
                self.execute_decision(&position.address, &decision).await?;
            }
        }

        Ok(())
    }

    /// Executes a decision.
    async fn execute_decision(
        &self,
        position: &solana_sdk::pubkey::Pubkey,
        decision: &Decision,
    ) -> anyhow::Result<()> {
        info!(
            position = %position,
            decision = %decision.description(),
            "Executing decision"
        );

        match decision {
            Decision::Hold => {
                // Nothing to do
            }
            Decision::Rebalance {
                new_tick_lower,
                new_tick_upper,
            } => {
                // TODO: Build and execute rebalance transaction
                info!(
                    new_lower = new_tick_lower,
                    new_upper = new_tick_upper,
                    "Would execute rebalance"
                );
            }
            Decision::Close => {
                // TODO: Build and execute close transaction
                info!("Would execute close");
            }
            Decision::IncreaseLiquidity { amount } => {
                // TODO: Build and execute increase liquidity transaction
                info!(amount = %amount, "Would execute increase liquidity");
            }
            Decision::DecreaseLiquidity { amount } => {
                // TODO: Build and execute decrease liquidity transaction
                info!(amount = %amount, "Would execute decrease liquidity");
            }
            Decision::CollectFees => {
                // TODO: Build and execute collect fees transaction
                info!("Would execute collect fees");
            }
        }

        Ok(())
    }
}
