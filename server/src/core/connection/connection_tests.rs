use super::*;
use futures::stream;
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
        BrokerEvent::Disconnect { addr: a } => {
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
    assert!(matches!(event, BrokerEvent::Disconnect { addr: a } if a == addr));
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
    assert!(matches!(event, BrokerEvent::Disconnect { addr: a } if a == addr));
}

// Receives fake error socket event and verify it ok.
#[tokio::test]
async fn ws_half_reader_sends_disconnect_on_error() {
    let (tx, mut rx) = mpsc::unbounded_channel::<BrokerEvent>();
    let addr = test_addr();

    let stream = stream::iter(vec![Err(Error::ConnectionClosed)]);
    ws_half_reader(stream, tx, addr).await;

    let event = rx.recv().await.unwrap();
    assert!(matches!(event, BrokerEvent::Disconnect { addr: a } if a == addr));
}

// Receives fake authenticate type event when we are not waiting for it and verify we receive nothing.
#[tokio::test]
async fn ws_half_reader_ignores_unexpected_authenticate() {
    let (tx, mut rx) = mpsc::unbounded_channel::<BrokerEvent>();
    let addr = test_addr();
    let json = r#"{"type":"authenticate","username":"a","password":"p","room_name":"r"}"#;

    let stream = stream::iter(vec![Ok(Message::Text(json.into()))]);
    ws_half_reader(stream, tx, addr).await;

    assert!(rx.recv().await.is_none());
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
