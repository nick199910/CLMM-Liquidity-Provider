//! Application state shared across handlers.

use clmm_lp_execution::prelude::{
    CircuitBreaker, LifecycleTracker, PositionMonitor, StrategyExecutor, TransactionManager,
};
use clmm_lp_protocols::prelude::{RpcConfig, RpcProvider};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// RPC provider.
    pub provider: Arc<RpcProvider>,
    /// Position monitor.
    pub monitor: Arc<PositionMonitor>,
    /// Transaction manager.
    pub tx_manager: Arc<TransactionManager>,
    /// Circuit breaker.
    pub circuit_breaker: Arc<CircuitBreaker>,
    /// Lifecycle tracker.
    pub lifecycle: Arc<LifecycleTracker>,
    /// Active strategies.
    pub strategies: Arc<RwLock<HashMap<String, StrategyState>>>,
    /// WebSocket broadcast channel for position updates.
    pub position_updates: broadcast::Sender<PositionUpdate>,
    /// WebSocket broadcast channel for alerts.
    pub alert_updates: broadcast::Sender<AlertUpdate>,
    /// API configuration.
    pub config: ApiConfig,
    /// Strategy executors by ID.
    pub executors: Arc<RwLock<HashMap<String, Arc<RwLock<StrategyExecutor>>>>>,
    /// Whether in dry-run mode.
    pub dry_run: bool,
}

impl AppState {
    /// Creates a new application state.
    pub fn new(rpc_config: RpcConfig, api_config: ApiConfig) -> Self {
        let provider = Arc::new(RpcProvider::new(rpc_config));
        let monitor = Arc::new(PositionMonitor::new(
            provider.clone(),
            clmm_lp_execution::prelude::MonitorConfig::default(),
        ));
        let tx_manager = Arc::new(TransactionManager::new(
            provider.clone(),
            clmm_lp_execution::prelude::TransactionConfig::default(),
        ));
        let circuit_breaker = Arc::new(CircuitBreaker::default());
        let lifecycle = Arc::new(LifecycleTracker::new());

        let (position_tx, _) = broadcast::channel(1000);
        let (alert_tx, _) = broadcast::channel(1000);

        Self {
            provider,
            monitor,
            tx_manager,
            circuit_breaker,
            lifecycle,
            strategies: Arc::new(RwLock::new(HashMap::new())),
            position_updates: position_tx,
            alert_updates: alert_tx,
            config: api_config,
            executors: Arc::new(RwLock::new(HashMap::new())),
            dry_run: true, // Default to dry-run for safety
        }
    }

    /// Sets dry-run mode.
    pub fn set_dry_run(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
    }

    /// Broadcasts a position update.
    pub fn broadcast_position_update(&self, update: PositionUpdate) {
        let _ = self.position_updates.send(update);
    }

    /// Broadcasts an alert update.
    pub fn broadcast_alert(&self, alert: AlertUpdate) {
        let _ = self.alert_updates.send(alert);
    }

    /// Subscribes to position updates.
    pub fn subscribe_positions(&self) -> broadcast::Receiver<PositionUpdate> {
        self.position_updates.subscribe()
    }

    /// Subscribes to alert updates.
    pub fn subscribe_alerts(&self) -> broadcast::Receiver<AlertUpdate> {
        self.alert_updates.subscribe()
    }
}

/// API configuration.
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// Server host.
    pub host: String,
    /// Server port.
    pub port: u16,
    /// API keys for authentication.
    pub api_keys: Vec<String>,
    /// Whether to enable CORS.
    pub enable_cors: bool,
    /// Request timeout in seconds.
    pub request_timeout_secs: u64,
    /// Rate limit per minute.
    pub rate_limit_per_minute: u32,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            api_keys: vec![],
            enable_cors: true,
            request_timeout_secs: 30,
            rate_limit_per_minute: 100,
        }
    }
}

/// State for an active strategy.
#[derive(Debug, Clone)]
pub struct StrategyState {
    /// Strategy ID.
    pub id: String,
    /// Strategy name.
    pub name: String,
    /// Whether strategy is running.
    pub running: bool,
    /// Strategy configuration as JSON.
    pub config: serde_json::Value,
    /// Created timestamp.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated timestamp.
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Position update for WebSocket broadcast.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PositionUpdate {
    /// Update type.
    pub update_type: String,
    /// Position address.
    pub position_address: String,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Update data.
    pub data: serde_json::Value,
}

/// Alert update for WebSocket broadcast.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AlertUpdate {
    /// Alert level.
    pub level: String,
    /// Alert message.
    pub message: String,
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Related position (if any).
    pub position_address: Option<String>,
}
