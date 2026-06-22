use super::*;
use crate::core::broker::BrokerRsp;
use futures::channel::mpsc as futures_mpsc;
use futures::stream;
use futures::StreamExt as _;
use tokio_tungstenite::tungstenite::protocol::frame::CloseFrame;
use tokio_tungstenite::tungstenite::Utf8Bytes;

// Parsing tests
#[test]
fn parse_authenticate_valid() {
    let json = r#"{"type":"authenticate","username":"alice","password":"p","room_name":"r"}"#;
    let result = parse_authenticate(json);
    assert!(result.is_ok());
    let auth = result.unwrap();
    assert_eq!(auth.username, "alice");
    assert_eq!(auth.password, "p");
    assert_eq!(auth.room_name, "r");
}

#[test]
fn parse_authenticate_rejects_send_first() {
    let json = r#"{"type":"send","username":"alice","message":"hello"}"#;
    let result = parse_authenticate(json);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ServerMessage::AuthResult { success: false, .. }
    ));
}

#[test]
fn parse_authenticate_rejects_invalid_json() {
    let result = parse_authenticate("not json");
    assert!(result.is_err());
}

#[test]
fn parse_client_message_send_produces_broadcast() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let json = r#"{"type":"send","username":"alice","message":"hello"}"#;
    let event = parse_broadcast_message(json, addr);
    assert!(event.is_some());
    match event.unwrap() {
        BrokerEvent::Broadcast {
            sender_addr,
            str_message,
            ..
        } => {
            assert_eq!(sender_addr, addr);
            assert_eq!(str_message, "hello");
        }
        other => panic!("expected Broadcast, got {other:?}"),
    }
}

#[test]
fn parse_client_message_logout_produces_disconnect() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let json = r#"{"type":"logout","message":"bye"}"#;
    let event = parse_broadcast_message(json, addr);
    assert!(event.is_some());
    match event.unwrap() {
        BrokerEvent::Disconnect { addr: a, .. } => {
            assert_eq!(a, addr);
        }
        other => panic!("expected Disconnect, got {other:?}"),
    }
}

#[test]
fn parse_client_message_unexpected_authenticate_returns_none() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let json = r#"{"type":"authenticate","username":"a","password":"p","room_name":"r"}"#;
    let event = parse_broadcast_message(json, addr);
    assert!(event.is_none());
}

#[test]
fn parse_client_message_invalid_json_returns_none() {
    let addr = "127.0.0.1:5000".parse().unwrap();
    let event = parse_broadcast_message("not json", addr);
    assert!(event.is_none());
}

fn test_addr() -> SocketAddr {
    "127.0.0.1:9000".parse().unwrap()
}

// Emulated stream channel (Half Reader)
// Receives fake broadcast event and verify it ok.
#[tokio::test]
async fn ws_half_reader_sends_broadcast_event() {
    let (tx, mut rx) = mpsc::unbounded_channel::<BrokerEvent>();
    let addr = test_addr();
    let json = r#"{"type":"send","username":"alice","message":"hello"}"#;

    let stream = stream::iter(vec![Ok(Message::Text(json.into()))]);
    ws_half_reader(stream, tx, addr).await;

    let event = rx.recv().await.unwrap();
    match event {
        BrokerEvent::Broadcast {
            sender_addr,
            str_message,
            ..
        } => {
            assert_eq!(sender_addr, addr);
            assert_eq!(str_message, "hello");
        }
        other => panic!("expected Broadcast, got {other:?}"),
    }
}

// Receives fake logout event and verify it ok.
#[tokio::test]
async fn ws_half_reader_sends_disconnect_on_logout() {
    let (tx, mut rx) = mpsc::unbounded_channel::<BrokerEvent>();
    let addr = test_addr();
    let json = r#"{"type":"logout","message":"bye"}"#;

    let stream = stream::iter(vec![Ok(Message::Text(json.into()))]);
    ws_half_reader(stream, tx, addr).await;

    let event = rx.recv().await.unwrap();
    assert!(matches!(event, BrokerEvent::Disconnect { addr: a, .. } if a == addr));
}

// Receives fake disconnect event and verify it ok.
#[tokio::test]
async fn ws_half_reader_sends_disconnect_on_close() {
    let (tx, mut rx) = mpsc::unbounded_channel::<BrokerEvent>();
    let addr = test_addr();

    let stream = stream::iter(vec![Ok(Message::Close(Some(CloseFrame {
        code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
        reason: Utf8Bytes::from("bye"),
    })))]);
    ws_half_reader(stream, tx, addr).await;

    let event = rx.recv().await.unwrap();
    assert!(matches!(event, BrokerEvent::Disconnect { addr: a, .. } if a == addr));
}

// Receives fake error socket event and verify it ok.
#[tokio::test]
async fn ws_half_reader_sends_disconnect_on_error() {
    let (tx, mut rx) = mpsc::unbounded_channel::<BrokerEvent>();
    let addr = test_addr();

    let stream = stream::iter(vec![Err(Error::ConnectionClosed)]);
    ws_half_reader(stream, tx, addr).await;

    let event = rx.recv().await.unwrap();
    assert!(matches!(event, BrokerEvent::Disconnect { addr: a, .. } if a == addr));
}

// Receives fake authenticate type event when we are not waiting for it and verify it is ignored.
// When the stream ends, a Disconnect event is sent.
#[tokio::test]
async fn ws_half_reader_ignores_unexpected_authenticate() {
    let (tx, mut rx) = mpsc::unbounded_channel::<BrokerEvent>();
    let addr = test_addr();
    let json = r#"{"type":"authenticate","username":"a","password":"p","room_name":"r"}"#;

    let stream = stream::iter(vec![Ok(Message::Text(json.into()))]);
    ws_half_reader(stream, tx, addr).await;

    // The authenticate message is ignored, but stream ending triggers disconnect.
    let event = rx.recv().await.unwrap();
    assert!(matches!(event, BrokerEvent::Disconnect { addr: a, .. } if a == addr));
}

#[tokio::test]
async fn ws_half_reader_handles_multiple_messages_until_disconnect() {
    let (tx, mut rx) = mpsc::unbounded_channel::<BrokerEvent>();
    let addr = test_addr();

    let messages = vec![
        Ok(Message::Text(
            r#"{"type":"send","username":"a","message":"first"}"#.into(),
        )),
        Ok(Message::Text(
            r#"{"type":"send","username":"a","message":"second"}"#.into(),
        )),
        Ok(Message::Text(r#"{"type":"logout","message":"bye"}"#.into())),
    ];

    let stream = stream::iter(messages);
    ws_half_reader(stream, tx, addr).await;

    let BrokerEvent::Broadcast { str_message, .. } = rx.recv().await.unwrap() else {
        panic!("expected Broadcast");
    };
    assert_eq!(str_message, "first");

    let BrokerEvent::Broadcast { str_message, .. } = rx.recv().await.unwrap() else {
        panic!("expected Broadcast");
    };
    assert_eq!(str_message, "second");

    let BrokerEvent::Disconnect { .. } = rx.recv().await.unwrap() else {
        panic!("expected Disconnect");
    };

    assert!(rx.try_recv().is_err());
}

// Writer integration tests
// Uses tokio::join! to avoid race conditions between sender and writer futures.
// When going out of scope in the sender, the tx channel used to send the msg
// is dropped. Because of that, depending on the execution runtime, it might
// happen that the writer receives nothing.
// Waiting for both futures using join avoid this.
// The explicit drop(tx) is needed because the async block keeps tx alive
// in its state machine until the future is fully dropped. Without explicit
// drop, the writer's recv() never returns None and the while let loop in
// ws_half_writer never finishes.

#[tokio::test]
async fn ws_half_writer_integration_chat_message() {
    let (sink, mut receiver) = futures_mpsc::unbounded::<Message>();
    let (tx, rx) = mpsc::unbounded_channel::<BrokerToClientMsg>();
    let sender_addr = test_addr();

    let writer = ws_half_writer(sink, rx);
    let sender = async {
        tx.send(BrokerToClientMsg::ChatMessage {
            sender: sender_addr,
            sender_name: "alice".to_string(),
            text: "hello!".to_string(),
            timestamp: "17/06/2026 18:30:00".to_string(),
        })
        .unwrap();
        drop(tx);
    };
    let recv = async {
        let msg = receiver.next().await.unwrap();
        match msg {
            Message::Text(text) => {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(parsed["type"], "chat");
                assert_eq!(parsed["sender"], "alice");
                assert_eq!(parsed["message"], "hello!");
                assert_eq!(parsed["timestamp"], "17/06/2026 18:30:00");
            }
            other => panic!("expected Text message, got {other:?}"),
        }
    };

    tokio::join!(writer, sender, recv);
}

#[tokio::test]
async fn ws_half_writer_integration_notification() {
    let (sink, mut receiver) = futures_mpsc::unbounded::<Message>();
    let (tx, rx) = mpsc::unbounded_channel::<BrokerToClientMsg>();

    let writer = ws_half_writer(sink, rx);
    let sender = async {
        tx.send(BrokerToClientMsg::Response(BrokerRsp::Broadcast {
            status: true,
        }))
        .unwrap();
        drop(tx);
    };
    let recv = async {
        let msg = receiver.next().await.unwrap();
        match msg {
            Message::Text(text) => {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(parsed["type"], "notification");
                assert_eq!(parsed["value"], "Message sent");
                assert!(parsed["timestamp"].is_string());
            }
            other => panic!("expected Text message, got {other:?}"),
        }
    };

    tokio::join!(writer, sender, recv);
}

#[tokio::test]
async fn ws_half_writer_integration_user_left_notification() {
    let (sink, mut receiver) = futures_mpsc::unbounded::<Message>();
    let (tx, rx) = mpsc::unbounded_channel::<BrokerToClientMsg>();

    let writer = ws_half_writer(sink, rx);
    let sender = async {
        tx.send(BrokerToClientMsg::Notification {
            text: "alice has left the room".to_string(),
            timestamp: "17/06/2026 18:30:00".to_string(),
        })
        .unwrap();
        drop(tx);
    };
    // Receiving notification that Alice has left the room.
    let recv = async {
        let msg = receiver.next().await.unwrap();
        match msg {
            Message::Text(text) => {
                let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
                assert_eq!(parsed["type"], "notification");
                assert_eq!(parsed["value"], "alice has left the room");
                assert_eq!(parsed["timestamp"], "17/06/2026 18:30:00");
            }
            other => panic!("expected Text message, got {other:?}"),
        }
    };

    tokio::join!(writer, sender, recv);
}

#[tokio::test]
async fn ws_half_writer_integration_shutdown() {
    let (sink, mut receiver) = futures_mpsc::unbounded::<Message>();
    let (_, rx) = mpsc::unbounded_channel::<BrokerToClientMsg>();

    let writer = ws_half_writer(sink, rx);
    let sender = async {};
    let recv = async {
        assert!(receiver.next().await.is_none());
    };

    tokio::join!(writer, sender, recv);
}
