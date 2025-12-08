//! Route definitions.

use crate::handlers;
use crate::state::AppState;
use crate::websocket;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

/// Creates the API router with all routes.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health routes
        .route("/health", get(handlers::health_check))
        .route("/health/live", get(handlers::liveness))
        .route("/health/ready", get(handlers::readiness))
        .route("/metrics", get(handlers::metrics))
        // Position routes
        .route("/positions", get(handlers::list_positions))
        .route("/positions", post(handlers::open_position))
        .route("/positions/{address}", get(handlers::get_position))
        .route("/positions/{address}", delete(handlers::close_position))
        .route("/positions/{address}/collect", post(handlers::collect_fees))
        .route(
            "/positions/{address}/rebalance",
            post(handlers::rebalance_position),
        )
        .route("/positions/{address}/pnl", get(handlers::get_position_pnl))
        // Strategy routes
        .route("/strategies", get(handlers::list_strategies))
        .route("/strategies", post(handlers::create_strategy))
        .route("/strategies/{id}", get(handlers::get_strategy))
        .route("/strategies/{id}", put(handlers::update_strategy))
        .route("/strategies/{id}", delete(handlers::delete_strategy))
        .route("/strategies/{id}/start", post(handlers::start_strategy))
        .route("/strategies/{id}/stop", post(handlers::stop_strategy))
        .route(
            "/strategies/{id}/performance",
            get(handlers::get_strategy_performance),
        )
        // Pool routes
        .route("/pools", get(handlers::list_pools))
        .route("/pools/{address}", get(handlers::get_pool))
        .route("/pools/{address}/state", get(handlers::get_pool_state))
        // Analytics routes
        .route(
            "/analytics/portfolio",
            get(handlers::get_portfolio_analytics),
        )
        .route("/analytics/simulate", post(handlers::run_simulation))
        // WebSocket routes
        .route("/ws/positions", get(websocket::positions_ws))
        .route("/ws/alerts", get(websocket::alerts_ws))
        // Add state
        .with_state(state)
}

/// Creates the API router with versioning prefix.
pub fn create_versioned_router(state: AppState) -> Router {
    Router::new().nest("/api/v1", create_router(state))
}
