//! Server accept loop and connection spawning
//!
//! This module implements the main server loop that accepts incoming TCP
//! connections and spawns a dedicated task for each client.
//!
//! # Responsibilities
//!
//! - Bind to TCP port and accept WebSocket connections
//! - Initialize the broker for message routing
//! - Spawn connection handler tasks for each client
//! - Handle graceful shutdown on Ctrl+C signal
//!
//! # Graceful Shutdown
//!
//! When Ctrl+C is received, the server:
//! 1. Stops accepting new connections
//! 2. Signals all active connection tasks to abort
//! 3. Waits for all tasks to complete
//! 4. Exits cleanly
//!
//! # Example
//!
//! ```rust,no_run
//! use tokio::net::TcpListener;
//! use server::core::server;
//! use server::auth::users::UserStore;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let listener = TcpListener::bind("0.0.0.0:1234").await.unwrap();
//!     let user_store = Arc::new(UserStore::new());
//!     server::run(listener, "logs", user_store).await.unwrap();
//! }
//! ```

use super::broker;
use super::connection;
use crate::auth::users::UserStore;
use flexi_logger::{Duplicate, FileSpec, Logger};
use log::info;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::task::JoinSet;
use tokio_tungstenite::accept_async;

// Example taken from: https://websocket.org/guides/languages/rust/
pub async fn run(
    listener: TcpListener,
    log_dir: &str,
    user_store: Arc<UserStore>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = JoinSet::new();

    Logger::try_with_str("info")?
        .format_for_files(flexi_logger::detailed_format)
        .format_for_stderr(flexi_logger::detailed_format)
        .log_to_file(
            FileSpec::default()
                .directory(log_dir)
                .basename("server")
                .suppress_timestamp(),
        )
        .append()
        .duplicate_to_stdout(Duplicate::All)
        .use_utc()
        .start()?;

    info!("Starting broker...");
    let tx_broker = broker::init();
    info!("Starting server on {}", listener.local_addr()?);
    loop {
        // Non blocking accept
        // If ctrl+C is hit, let's join gracefully the spawned tasks.
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                let tx_clone = tx_broker.clone();
                let user_store_clone = user_store.clone();
                info!("New connection from {addr}");
                tasks.spawn(async move {

                    let Ok(ws) = accept_async(stream).await else {
                        eprintln!("{addr} failed to connect");
                        return;
                    };
                    connection::handle(ws, addr, tx_clone, user_store_clone).await;
                });
            }
            _ = signal::ctrl_c() => {
                tasks.abort_all();
                eprintln!("shutting down, draining connections");
                break;
            }
        }
    }

    // Wait for all active connections to finish.
    while tasks.join_next().await.is_some() {}
    println!("Server shutdown cleanly.");
    Ok(())
}
