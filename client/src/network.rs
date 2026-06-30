use crate::{ui, Context};
use common::ws_messages::{AuthenticateUser, ClientMessage, ServerMessage};
use cursive::{CbSink, Cursive};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use log::{error, info};
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{timeout, Duration};
use tokio::{net::TcpStream, select};
use tokio_tungstenite::{
    tungstenite::{Error, Message},
    WebSocketStream,
};

pub fn connect_to_server(ctx: Arc<Context>, cb_sink: CbSink) {
    tokio::spawn(async move {
        // Ref: https://medium.com/@abhishekranjandev/a-novices-guide-to-rust-building-your-own-discord-like-application-bcbb362a73d1
        let addr = format!("{}:{}", ctx.server_ip, ctx.server_port);

        match tokio::net::TcpStream::connect(&addr).await {
            Ok(stream) => {
                let url = format!("ws://{addr}");
                let username = ctx.username.lock().unwrap().clone();

                match tokio_tungstenite::client_async(&url, stream).await {
                    Ok((ws_stream, _)) => {
                        info!("Starting connection as {username} to...");
                        handle_connection(ctx, ws_stream, cb_sink).await;
                    }
                    Err(e) => {
                        let err = format!("Handshake failed: {e}");

                        ui::status::set_connection_status(ctx, &cb_sink, false);
                        ui::dialogs::set_notification(&cb_sink, &err);
                    }
                }
            }
            Err(e) => {
                let err = format!("{e}");
                error!("{err}");
                ui::status::set_connection_status(ctx, &cb_sink, false);
                ui::dialogs::set_notification(&cb_sink, &err);
            }
        }
    });
}

async fn handle_connection(ctx: Arc<Context>, ws: WebSocketStream<TcpStream>, cb_sink: CbSink) {
    let (mut writer, mut reader) = ws.split();
    ui::dialogs::set_notification(&cb_sink, "Connecting...");

    let auth = ClientMessage::Authenticate(AuthenticateUser {
        username: ctx.username.lock().unwrap().clone(),
        password: ctx.password.lock().unwrap().clone(),
        room_name: ctx.room.lock().unwrap().clone(),
    });
    let json = serde_json::to_string(&auth).unwrap();
    // send authentication
    if writer.send(Message::Text(json.into())).await.is_err() {
        let msg = "Error sending authentication to server";

        error!("{}", msg);
        ui::dialogs::set_notification(&cb_sink, msg);
        return;
    }
    //wait for response
    match timeout(Duration::from_secs(5), reader.next()).await {
        Ok(Some(Ok(Message::Text(text)))) => match serde_json::from_str::<ServerMessage>(&text) {
            Ok(ServerMessage::AuthResult { success: true, msg }) => {
                if let Some(msg) = msg {
                    ui::dialogs::display_message(&cb_sink, msg);
                }
            }
            _ => {
                let msg = "Authentication failed";
                error!("{}", msg);
                ui::dialogs::set_notification(&cb_sink, msg);
                return;
            }
        },
        _ => {
            let msg = "Auth timeout";
            error!("{}", msg);
            ui::dialogs::set_notification(&cb_sink, "Auth Timeout");
            return;
        }
    }

    // update connection status
    ui::status::set_connection_status(ctx.clone(), &cb_sink, true);
    // update room name
    ui::status::set_room_name(&cb_sink, &ctx.room.lock().unwrap().clone());
    // Dismiss Connect dialog
    cb_sink
        .send(Box::new(|s: &mut Cursive| {
            s.pop_layer();
        }))
        .ok();
    ui::dialogs::set_notification(&cb_sink, "");
    let (tx, rx) = unbounded_channel::<String>();
    // Save tx channel in context for "input" TextView to send messages to server.
    *ctx.tx_msg.lock().unwrap() = Some(tx);

    let r = ws_half_reader(ctx.clone(), reader, cb_sink.clone());
    let w = ws_half_writer(writer, rx);
    select! {
        _ = r => info!("{} reader closed", ctx.username.lock().unwrap()),
        _ = w => info!("{} writer closed", ctx.username.lock().unwrap()),
    }
}

async fn ws_half_reader(
    ctx: Arc<Context>,
    mut stream: SplitStream<WebSocketStream<TcpStream>>,
    cb_sink: CbSink,
) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received from server: {}", text);
                handle_incoming_server_msg(&cb_sink, text.to_string());
            }
            Ok(Message::Close(_)) => {
                info!("connection closed");
                ui::status::set_connection_status(ctx.clone(), &cb_sink, false);
            }
            Err(Error::ConnectionClosed) => {
                error!("connection closed");
                ui::status::set_connection_status(ctx.clone(), &cb_sink, false);
            }
            Err(e) => {
                error!("Network error: {}", e);
                ui::status::set_connection_status(ctx.clone(), &cb_sink, false);
            }
            _ => {
                error!("Unknown network error");
                return;
            }
        }
    }
}

async fn ws_half_writer(
    mut sink: SplitSink<WebSocketStream<TcpStream>, Message>,
    mut rx_channel: UnboundedReceiver<String>,
) {
    while let Some(json_msg) = rx_channel.recv().await {
        if let Err(e) = sink.send(Message::Text(json_msg.into())).await {
            error!("Error sending msg to server: {}", e);
            continue;
        }
    }
    info!("Writer channel closed");
}

fn handle_incoming_server_msg(cb_sink: &CbSink, ws_server_msg: String) {
    // Convert JSON into ServerMessage and fill in Messages ScrollView
    if let Ok(msg_struct) = serde_json::from_str::<ServerMessage>(&ws_server_msg) {
        match msg_struct {
            ServerMessage::Chat {
                sender,
                message,
                timestamp,
            } => {
                ui::dialogs::display_message(
                    cb_sink,
                    format!("{}-{}:{}", timestamp, sender, message),
                );
            }
            ServerMessage::Notification { value, timestamp } => {
                let msg = format!("{}: {}", timestamp, value);
                ui::dialogs::set_notification(cb_sink, &msg);
            }
            ServerMessage::Error { value } => {
                let str = format!("Received Error ServerMessage: {}.", value);
                error!("{}", str);
                ui::dialogs::set_notification(cb_sink, &str);
            }
            _ => {
                let str = "Received wrong/unknown ServerMessage type.";
                error!("{}", str);
                ui::dialogs::set_notification(cb_sink, str);
            }
        }
    } else {
        let str = "Unable to deserialize Server message.";
        error!("{}", str);
        ui::dialogs::set_notification(cb_sink, str);
    }
}
