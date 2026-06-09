use crate::auth::client;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use std::net::SocketAddr;
use tokio_tungstenite::tungstenite::{Error, Message};

pub async fn handle<S>(ws: S, addr: SocketAddr) -> ()
where
    S: Stream<Item = Result<Message, Error>> + Sink<Message>,
{
    client::authenticate("user", "password");

    let (mut sink, mut source) = ws.split();

    // Get first frame where client data should be.
    let Some(Ok(Message::Text(_data))) = source.next().await else {
        eprintln!("{addr} disconnected");
        return;
    };

    while let Some(Ok(msg)) = source.next().await {
        if let Message::Text(_) = msg {
            let _ = sink.send(msg).await;
        }
    }
    eprintln!("{addr} disconnected");
}
