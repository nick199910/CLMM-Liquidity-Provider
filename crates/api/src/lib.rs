//! REST API server and endpoints.
//!
//! This crate provides a REST API for the CLMM LP Strategy Optimizer:
//! - Position management endpoints
//! - Strategy configuration and execution
//! - Pool information and analytics
//! - Real-time WebSocket updates
//! - OpenAPI documentation

/// Prelude module for convenient imports.
pub mod prelude;

/// Error types.
pub mod error;
/// Request handlers.
pub mod handlers;
/// Middleware components.
pub mod middleware;
/// API request/response models.
pub mod models;
/// Route definitions.
pub mod routes;
/// Server configuration and startup.
pub mod server;
/// Application state.
pub mod state;
/// WebSocket handlers.
pub mod websocket;

pub use error::ApiError;
pub use server::{ApiServer, ServerConfig};
pub use state::AppState;
