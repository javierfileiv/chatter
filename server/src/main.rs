mod auth;
mod core;

use clap::Parser;
use core::server;
use tokio::net::TcpListener;

#[derive(Parser, Debug)]
#[command(name = "chatter-server", about = "WebSocket chat server")]
struct Args {
    #[arg(short, long, default_value = "1234")]
    port: u16,
    #[arg(short, long, default_value = "logs")]
    log_dir: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let addr = format!("127.0.0.1:{}", args.port);
    let listener = TcpListener::bind(&addr).await?;

    server::run(listener, &args.log_dir).await?;
    Ok(())
}
