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
