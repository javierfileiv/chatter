use super::broker;
use super::connection;
use flexi_logger::{Duplicate, FileSpec, Logger};
use log::info;
use tokio::net::TcpListener;
use tokio::signal;
use tokio::task::JoinSet;
use tokio_tungstenite::accept_async;

// Example taken from: https://websocket.org/guides/languages/rust/
pub async fn run(listener: TcpListener, log_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
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
        .duplicate_to_stderr(Duplicate::Warn)
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
                info!("New connection from {addr}");
                tasks.spawn(async move {

                    let Ok(ws) = accept_async(stream).await else {
                        eprintln!("{addr} failed to connect");
                        return;
                    };
                    connection::handle(ws, addr, tx_clone).await;
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
