use crate::network::connection;
use common::protocol;
use flexi_logger::{ Duplicate, FileSpec, Logger };
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::broadcast::channel;
use tokio::task::JoinSet;
use tokio_tungstenite::accept_async;
use log::{ info, warn, error };

pub async fn run(listener: TcpListener) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = JoinSet::new();

    Logger::try_with_str("info")?
        .format_for_files(flexi_logger::detailed_format)
        .format_for_stderr(flexi_logger::detailed_format)
        .log_to_file(FileSpec::default().directory("logs").basename("server").suppress_timestamp()) 
        .append()
        .duplicate_to_stderr(Duplicate::Warn)
        .start()?;

    info!("Starting server");
    warn!("A warning");
    error!("An error!");
    loop {
        // Non blocking accept
        // If new connection, create an async under tasks to handle it.
        // If ctrl+C is hit, let's join gracefully the spawned tasks.
        tokio::select! {
            Ok((stream, addr)) = listener.accept() => {
                tasks.spawn(async move {

                    let Ok(ws) = accept_async(stream).await else {
                        eprintln!("{addr} failed to connect");
                        return;
                    };
                    connection::handle(ws, addr).await;
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
