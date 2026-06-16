use super::*;

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
