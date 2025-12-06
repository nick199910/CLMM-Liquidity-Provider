//! REST API server and endpoints.

/// Prelude module for convenient imports.
pub mod prelude;

/// Request handlers.
pub mod handlers;
/// Middleware components.
pub mod middleware;
/// API models.
pub mod models;
/// Route definitions.
pub mod routes;
/// Server configuration.
pub mod server;

/// Runs the API server.
pub async fn run() {
    println!("Server running...");
}
