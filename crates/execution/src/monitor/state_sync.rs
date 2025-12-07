//! State synchronization for position monitoring.

use clmm_lp_protocols::prelude::*;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Synchronization state for an account.
#[derive(Debug, Clone)]
pub struct SyncState {
    /// Last known slot.
    pub last_slot: u64,
    /// Last sync timestamp.
    pub last_sync: chrono::DateTime<chrono::Utc>,
    /// Number of sync errors.
    pub error_count: u32,
    /// Whether sync is healthy.
    pub is_healthy: bool,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_slot: 0,
            last_sync: chrono::Utc::now(),
            error_count: 0,
            is_healthy: true,
        }
    }
}

/// State synchronizer for keeping local state in sync with on-chain.
pub struct StateSynchronizer {
    /// RPC provider.
    provider: Arc<RpcProvider>,
    /// Sync state for each account.
    sync_states: Arc<RwLock<HashMap<Pubkey, SyncState>>>,
    /// Maximum errors before marking unhealthy.
    max_errors: u32,
}

impl StateSynchronizer {
    /// Creates a new state synchronizer.
    pub fn new(provider: Arc<RpcProvider>) -> Self {
        Self {
            provider,
            sync_states: Arc::new(RwLock::new(HashMap::new())),
            max_errors: 3,
        }
    }

    /// Registers an account for synchronization.
    pub async fn register_account(&self, account: Pubkey) {
        let mut states = self.sync_states.write().await;
        states.insert(account, SyncState::default());

        info!(account = %account, "Registered account for sync");
    }

    /// Unregisters an account from synchronization.
    pub async fn unregister_account(&self, account: &Pubkey) {
        let mut states = self.sync_states.write().await;
        states.remove(account);

        info!(account = %account, "Unregistered account from sync");
    }

    /// Records a successful sync.
    pub async fn record_success(&self, account: &Pubkey, slot: u64) {
        let mut states = self.sync_states.write().await;
        if let Some(state) = states.get_mut(account) {
            state.last_slot = slot;
            state.last_sync = chrono::Utc::now();
            state.error_count = 0;
            state.is_healthy = true;

            debug!(
                account = %account,
                slot = slot,
                "Sync success"
            );
        }
    }

    /// Records a sync error.
    pub async fn record_error(&self, account: &Pubkey) {
        let mut states = self.sync_states.write().await;
        if let Some(state) = states.get_mut(account) {
            state.error_count += 1;
            if state.error_count >= self.max_errors {
                state.is_healthy = false;
                warn!(
                    account = %account,
                    errors = state.error_count,
                    "Account sync marked unhealthy"
                );
            }
        }
    }

    /// Gets the sync state for an account.
    pub async fn get_sync_state(&self, account: &Pubkey) -> Option<SyncState> {
        let states = self.sync_states.read().await;
        states.get(account).cloned()
    }

    /// Gets all sync states.
    pub async fn get_all_states(&self) -> HashMap<Pubkey, SyncState> {
        let states = self.sync_states.read().await;
        states.clone()
    }

    /// Checks if all accounts are healthy.
    pub async fn is_all_healthy(&self) -> bool {
        let states = self.sync_states.read().await;
        states.values().all(|s| s.is_healthy)
    }

    /// Gets the current slot from RPC.
    pub async fn get_current_slot(&self) -> anyhow::Result<u64> {
        self.provider.get_slot().await
    }

    /// Reconciles local state with on-chain state.
    pub async fn reconcile(&self) -> anyhow::Result<ReconcileResult> {
        let current_slot = self.get_current_slot().await?;
        let states = self.sync_states.read().await;

        let mut result = ReconcileResult::default();

        for (account, state) in states.iter() {
            if state.last_slot < current_slot {
                result.stale_accounts.push(*account);
            }

            if !state.is_healthy {
                result.unhealthy_accounts.push(*account);
            }
        }

        result.current_slot = current_slot;
        result.total_accounts = states.len();

        Ok(result)
    }
}

/// Result of a reconciliation check.
#[derive(Debug, Clone, Default)]
pub struct ReconcileResult {
    /// Current slot.
    pub current_slot: u64,
    /// Total accounts being tracked.
    pub total_accounts: usize,
    /// Accounts with stale data.
    pub stale_accounts: Vec<Pubkey>,
    /// Unhealthy accounts.
    pub unhealthy_accounts: Vec<Pubkey>,
}

impl ReconcileResult {
    /// Checks if all accounts are up to date.
    #[must_use]
    pub fn is_all_synced(&self) -> bool {
        self.stale_accounts.is_empty() && self.unhealthy_accounts.is_empty()
    }
}
