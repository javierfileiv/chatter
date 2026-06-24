use crate::Context;
use cursive::{views::TextView, CbSink};
use futures_util::{stream::SplitStream, StreamExt};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::{net::TcpStream, select};
use tokio_tungstenite::{
    tungstenite::{Error, Message},
    WebSocketStream,
};

pub fn connect_to_server(
    ctx: Arc<Context>,
    cb_sink: CbSink,
    username: String,
    _password: String,
    _room: String,
) {
    tokio::spawn(async move {
        let addr = format!("{}:{}", ctx.server_ip, ctx.server_port);
        info!("Connecting {username} to {addr}");

        // Ref: https://medium.com/@abhishekranjandev/a-novices-guide-to-rust-building-your-own-discord-like-application-bcbb362a73d1
        match tokio::net::TcpStream::connect(&addr).await {
            Ok(stream) => {
                let url = format!("ws://{addr}");
                match tokio_tungstenite::client_async(&url, stream).await {
                    Ok((ws_stream, _)) => {
                        info!("Connected as {username}, handling connection...");
                        // save room name, credentials and update connection status
                        *ctx.connected.lock().unwrap() = true;
                        notify_connection_status(&cb_sink, true);
                        handle_connection(ctx, ws_stream, cb_sink);
                    }
                    Err(e) => {
                        let err = format!("Handshake failed: {e}");
                        *ctx.connected.lock().unwrap() = false;
                        cb_sink
                            .send(Box::new(move |s| {
                                s.call_on_name("notification", |view: &mut TextView| {
                                    view.set_content(err)
                                });
                            }))
                            .ok();
                    }
                }
            }
            Err(e) => {
                let err = format!("Connection refused: {e}");
                *ctx.connected.lock().unwrap() = false;
                cb_sink
                    .send(Box::new(move |s| {
                        s.call_on_name("notification", |view: &mut TextView| view.set_content(err));
                    }))
                    .ok();
            }
        }
    });
}

fn handle_connection(ctx: Arc<Context>, ws: WebSocketStream<TcpStream>, cb_sink: CbSink) {
    let (_writer, reader) = ws.split();

    let (tx, rx) = unbounded_channel::<String>();
    *ctx.tx_msg.lock().unwrap() = Some(tx);

    let r = ws_half_reader(ctx.clone(), reader, cb_sink.clone());
    let w = ws_half_writer(ctx.clone(), rx, cb_sink.clone());

    tokio::spawn(async move {
        select! {
        _ = r => info!("{} reader closed", ctx.username.lock().unwrap()),
        _ = w => info!("{} writer closed", ctx.username.lock().unwrap()),
        }
    });
}

async fn ws_half_reader(
    ctx: Arc<Context>,
    stream: SplitStream<WebSocketStream<TcpStream>>,
    cb_sink: CbSink,
) {
    stream
        .for_each(|message| async {
            match message {
                Ok(Message::Text(text)) => {
                    // Convert JSON into ServerMessage and fill in Messages ScrollView
                    info!("Received: {}", text);
                }
                Ok(Message::Close(_)) => {
                    info!("connection closed");
                    *ctx.connected.lock().unwrap() = false;
                    notify_connection_status(&cb_sink, false);
                }
                Err(Error::ConnectionClosed) => {
                    error!("connection closed");
                    *ctx.connected.lock().unwrap() = false;
                    notify_connection_status(&cb_sink, false);
                }
                Err(e) => {
                    error!("Network error: {}", e);
                    *ctx.connected.lock().unwrap() = false;
                    notify_connection_status(&cb_sink, false);
                }
                _ => {}
            }
        })
        .await
}

async fn ws_half_writer(_ctx: Arc<Context>, _rx: UnboundedReceiver<String>, _cb_sink: CbSink) {}

fn notify_connection_status(cb_sink: &CbSink, connected: bool) {
    match connected {
        true => {
            cb_sink
                .send(Box::new(move |s| {
                    s.call_on_name("notification", |view: &mut TextView| {
                        view.set_content("Connected")
                    });
                }))
                .ok();
        }
        false => {
            cb_sink
                .send(Box::new(move |s| {
                    s.call_on_name("notification", |view: &mut TextView| {
                        view.set_content("Disconnected")
                    });
                }))
                .ok();
        }
    }
}
