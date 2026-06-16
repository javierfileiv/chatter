#[cfg(test)]
mod connection_tests;

use crate::auth::client;
use crate::core::broker::{BrokerClient, BrokerEvent, BrokerToClientMsg};
use common::ws_messages::{AuthenticateUser, ClientMessage, ServerMessage};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use log::{error, info, warn};
use std::net::SocketAddr;
use tokio::select;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::tungstenite::{Error, Message};

// Parse client raw JSON and convert it to a broker authentication request.
fn parse_authenticate(raw: &str) -> Result<AuthenticateUser, ServerMessage> {
    match serde_json::from_str::<ClientMessage>(raw) {
        Ok(ClientMessage::Authenticate(auth)) => Ok(auth),
        _ => Err(ServerMessage::AuthResult {
            success: false,
            error: Some("First message must be authenticate".to_string()),
        }),
    }
}

// Parse client raw JSON and convert it to a broker broadcast request.
fn parse_broadcast_message(raw: &str, client_addr: SocketAddr) -> Option<BrokerEvent> {
    match serde_json::from_str::<ClientMessage>(raw) {
        Ok(ClientMessage::Broadcast(send_msg)) => Some(BrokerEvent::Broadcast {
            sender_addr: client_addr,
            str_message: send_msg.message,
        }),
        Ok(ClientMessage::Logout(_)) => Some(BrokerEvent::Disconnect { addr: client_addr }),
        Ok(ClientMessage::Authenticate(_)) => {
            warn!("Unexpected authenticate message in reader loop");
            None
        }
        Err(e) => {
            error!("Failed to parse client message: {}", e);
            None
        }
    }
}

async fn ws_half_reader<T>(
    mut stream: T,
    to_broker: UnboundedSender<BrokerEvent>,
    client_addr: SocketAddr,
) where
    T: Stream<Item = Result<Message, Error>> + Unpin,
{
    while let Some(ws_msg) = stream.next().await {
        match ws_msg {
            Ok(Message::Text(json_str)) => {
                if let Some(event) = parse_broadcast_message(&json_str, client_addr) {
                    let is_disconnect = matches!(event, BrokerEvent::Disconnect { .. });
                    let _ = to_broker.send(event);
                    if is_disconnect {
                        return;
                    }
                }
            }
            Ok(Message::Close(_)) | Ok(_) => {
                info!("Client closed WebSocket, disconnect client");
                let _ = to_broker.send(BrokerEvent::Disconnect { addr: client_addr });
                return;
            }
            Err(e) => {
                error!("WebSocket read error: {}, disconnect client", e);
                let _ = to_broker.send(BrokerEvent::Disconnect { addr: client_addr });
                return;
            }
        }
    }
}

async fn ws_half_writer<S>(mut sink: S, mut broker_rx: UnboundedReceiver<BrokerToClientMsg>)
where
    S: Sink<Message> + Unpin,
{
    while let Some(broker_msg) = broker_rx.recv().await {
        let server_msg = match ServerMessage::try_from(broker_msg) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to convert BrokerToClientMsg: {}", e);
                continue;
            }
        };

        let json = match serde_json::to_string(&server_msg) {
            Ok(j) => j,
            Err(e) => {
                error!("Failed to serialize ServerMessage: {}", e);
                continue;
            }
        };

        if sink.send(Message::Text(json.into())).await.is_err() {
            error!("Failed to send message to client");
            return;
        }
    }

    info!("Broker channel closed, writer shutting down");
}

pub async fn handle<S>(ws: S, addr: SocketAddr, broker_sender: UnboundedSender<BrokerEvent>)
where
    S: Stream<Item = Result<Message, Error>> + Sink<Message>,
{
    let (mut sink, mut stream) = ws.split();

    let Some(Ok(Message::Text(data))) = stream.next().await else {
        info!("{addr} disconnected");
        return;
    };

    let auth_msg = match parse_authenticate(&data) {
        Ok(auth) => auth,
        Err(response) => {
            let json = serde_json::to_string(&response).unwrap();
            let _ = sink.send(Message::Text(json.into())).await;
            return;
        }
    };

    let str_id = auth_msg.username.clone();
    let password = auth_msg.password.clone();

    if !client::authenticate(&str_id, &password) {
        let response = ServerMessage::AuthResult {
            success: false,
            error: Some("Authentication failed".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        let _ = sink.send(Message::Text(json.into())).await;
        warn!("Bad authentication for {str_id}, disconnecting..");
        return;
    }

    let (broker_tx, broker_rx) = mpsc::unbounded_channel::<BrokerToClientMsg>();

    let broker_client = BrokerClient {
        str_id,
        addr,
        room_name: auth_msg.room_name.clone(),
        broker_to_client: broker_tx,
    };

    let msg_to_broker = BrokerEvent::Connect {
        client: broker_client,
    };

    if broker_sender.send(msg_to_broker).is_err() {
        warn!("Broker has shut down, disconnecting {addr}");
        return;
    }

    let reader = ws_half_reader(stream, broker_sender.clone(), addr);
    let writer = ws_half_writer(sink, broker_rx);

    // we use select to drop one when the other dies.
    select! {
        _ = reader => info!("{addr} reader closed"),
        _ = writer => info!("{addr} writer closed"),
    }
}
