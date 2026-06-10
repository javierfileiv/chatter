use crate::auth::client;
use crate::core::broker::{BrokerClient, BrokerEvent, BrokerToClientMsg};
use futures_util::{Sink, Stream, StreamExt};
use std::net::SocketAddr;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_tungstenite::tungstenite::{Error, Message};

pub async fn handle<S>(ws: S, addr: SocketAddr, broker_sender: UnboundedSender<BrokerEvent>)
where
    S: Stream<Item = Result<Message, Error>> + Sink<Message>,
{
    let (_, mut source) = ws.split();

    // Get first frame where client authentification data should be.
    let Some(Ok(Message::Text(_data))) = source.next().await else {
        eprintln!("{addr} disconnected");
        return;
    };

    // extract client info from _data
    let username = "Alice".to_string();
    let password = "password".to_string();

    client::authenticate(&username, &password);

    let room = "games".to_string();
    let (from_broker, _) = mpsc::unbounded_channel::<BrokerToClientMsg>();

    let broker_client = BrokerClient {
        user: username,
        addr,
        room,
        broker_to_client: from_broker,
    };

    let msg = BrokerEvent::Connect {
        client: broker_client,
    };

    broker_sender.send(msg).unwrap();

    while let Some(Ok(msg)) = source.next().await {
        if let Message::Text(_) = msg {
            todo!("Handle broadcast message")
        }
    }
    eprintln!("{addr} disconnected");
    todo!("Spawn thread to handle received messages from broker.");
}
