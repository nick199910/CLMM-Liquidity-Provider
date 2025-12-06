//! REST API server and endpoints.
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod server;

/// Runs the API server.
pub async fn run() {
    println!("Server running...");
}
