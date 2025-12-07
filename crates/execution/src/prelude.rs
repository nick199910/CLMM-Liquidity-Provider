//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from the crate.
//!
//! # Example
//!
//! ```rust
//! use clmm_lp_execution::prelude::*;
//! ```

// Alerts
pub use crate::alerts::{
    Alert, AlertData, AlertLevel, AlertRule, AlertType, ConsoleNotifier, FileNotifier,
    MultiNotifier, Notifier, RuleCondition, RuleContext, RulesEngine, WebhookNotifier,
};

// Monitor
pub use crate::monitor::{
    MonitorConfig, MonitoredPosition, PnLResult, PnLTracker, PortfolioMetrics, PositionEntry,
    PositionMonitor, PositionPnL, ReconcileResult, StateSynchronizer, SyncState,
};

// Strategy
pub use crate::strategy::{
    Decision, DecisionConfig, DecisionContext, DecisionEngine, ExecutorConfig, StrategyExecutor,
};

// Transaction
pub use crate::transaction::{
    PriorityLevel, SimulationResult, TransactionBuilder, TransactionConfig, TransactionManager,
    TransactionResult, TransactionStatus,
};

// Wallet
pub use crate::wallet::{Wallet, WalletManager};
