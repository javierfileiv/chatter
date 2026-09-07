//! Chatter Server - WebSocket-based chat server
//!
//! This is the main entry point for the chatter server application.
//! It parses command-line arguments, binds to a TCP port, and starts
//! the WebSocket server that handles client connections.
//!
//! # Architecture
//!
//! The server uses a broker pattern for message routing:
//! - Each client connection is handled in a separate async task
//! - Messages are routed through a central broker that manages rooms
//! - Authentication uses Argon2 password hashing with auto-registration
//!
//! # Example
//!
//! ```bash
//! # Start server on default port 1234
//! cargo run --bin server
//!
//! # Start on custom port
//! cargo run --bin server -- --port 3000
//! ```

mod auth;
mod core;

use auth::users::UserStore;
use clap::Parser;
use core::server;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
#[command(name = "chatter-server", about = "WebSocket chat server")]
struct Args {
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
    #[arg(short, long, default_value = "1234")]
    port: u16,
    #[arg(short, long, default_value = "logs")]
    log_dir: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let addr = format!("{}:{}", args.host, args.port);
    let listener = TcpListener::bind(&addr).await?;

    let user_store = Arc::new(UserStore::new());
    server::run(listener, &args.log_dir, user_store).await?;
    Ok(())
}
