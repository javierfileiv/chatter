use super::*;
use tokio::sync::mpsc::UnboundedReceiver;

fn fake_client(
    addr: SocketAddr,
    user: &str,
    room: &str,
) -> (BrokerClient, mpsc::UnboundedReceiver<BrokerToClientMsg>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (
        BrokerClient {
            str_id: user.to_string(),
            addr,
            room_name: room.to_string(),
            broker_to_client: tx,
        },
        rx,
    )
}

fn test_timestamp() -> String {
    "17/06/2026 18:30:00".to_string()
}
async fn check_response(rx_from_client: &mut UnboundedReceiver<BrokerToClientMsg>) {
    let response_from_broker =
        tokio::time::timeout(std::time::Duration::from_millis(100), rx_from_client.recv()).await;

    match response_from_broker {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::AddedToBroker { status }))) => {
            assert!(status, "Broker has not connected client");
            println!("AddedToBroker Treated");
        }
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::JoinRoom { status, created }))) => {
            assert!(status, "Broker has not joined a room");
            if created {
                println!("Room created");
            } else {
                println!("Room joined");
            }
            println!("JoinRoom Treated");
        }
        Ok(Some(BrokerToClientMsg::ChatMessage {
            sender,
            sender_name,
            text,
            timestamp,
        })) => {
            println!(
                "Received message from {} (addr:{}) at {}: {}",
                sender_name, sender, timestamp, text
            );
        }
        Ok(Some(BrokerToClientMsg::UserLogoutNtf { text, timestamp })) => {
            println!("Received logout notification at {}: {}", timestamp, text);
        }
        Ok(None) => {
            panic!("Closed mpsc.");
        }
        Err(e) => {
            panic!("Issue with broker {}.", e);
        }
    }
}

#[tokio::test]
async fn test_broker_connection_success() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");
    let result = tx_broker.send(BrokerEvent::AddUserToBroker {
        client,
        timestamp: test_timestamp(),
    });

    assert!(
        result.is_ok(),
        "The Broker should have received the AddedToBroker event"
    );
    check_response(&mut rx).await;
}

#[tokio::test]
async fn test_broker_broadcast_success() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx).await;

    let result = tx_broker.send(BrokerEvent::Broadcast {
        sender_addr: addr,
        str_message: "Hello World".into(),
        timestamp: test_timestamp(),
    });

    assert!(
        result.is_ok(),
        "The Broker should have received the Broadcast event"
    );
    // Sender receives their own ChatMessage (broadcast goes to everyone in room)
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    match resp {
        Ok(Some(BrokerToClientMsg::ChatMessage {
            sender_name, text, ..
        })) => {
            assert_eq!(sender_name, "Alice");
            assert_eq!(text, "Hello World");
        }
        other => panic!("Alice expected own ChatMessage, got {:?}", other),
    }
    // Nothing else
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    assert!(resp.is_err(), "No second message expected for sender");
}

#[tokio::test]
async fn broadcast_to_multiple_clients_in_same_room() {
    let tx_broker = init();
    let addr_alice: SocketAddr = "127.0.0.1:6001".parse().unwrap();
    let addr_bob: SocketAddr = "127.0.0.1:6002".parse().unwrap();

    let (alice, mut rx_alice) = fake_client(addr_alice, "Alice", "games");
    let (bob, mut rx_bob) = fake_client(addr_bob, "Bob", "games");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client: alice,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx_alice).await;

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client: bob,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx_bob).await;

    tx_broker
        .send(BrokerEvent::Broadcast {
            sender_addr: addr_alice,
            str_message: "Hi everyone!".into(),
            timestamp: test_timestamp(),
        })
        .unwrap();

    // Alice receives her own ChatMessage (broadcast goes to everyone in room)
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx_alice.recv()).await;
    match resp {
        Ok(Some(BrokerToClientMsg::ChatMessage {
            sender_name, text, ..
        })) => {
            assert_eq!(sender_name, "Alice");
            assert_eq!(text.to_string(), "Hi everyone!");
        }
        other => panic!("Alice expected own ChatMessage, got {:?}", other),
    }
    // Nothing else for Alice
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx_alice.recv()).await;
    assert!(resp.is_err(), "No second message for Alice");

    // Bob should receive Alice's chat message
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx_bob.recv()).await;
    match resp {
        Ok(Some(BrokerToClientMsg::ChatMessage {
            sender_name, text, ..
        })) => {
            assert_eq!(sender_name, "Alice");
            assert_eq!(text.to_string(), "Hi everyone!");
        }
        other => panic!("Bob expected chat message from Alice, got {:?}", other),
    }
}

#[tokio::test]
async fn broadcast_only_sender_in_room() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:6003".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Solo", "empty");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx).await;

    tx_broker
        .send(BrokerEvent::Broadcast {
            sender_addr: addr,
            str_message: "Nobody here".into(),
            timestamp: test_timestamp(),
        })
        .unwrap();

    // Solo receives own ChatMessage (broadcast goes to everyone in room)
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    match resp {
        Ok(Some(BrokerToClientMsg::ChatMessage {
            sender_name, text, ..
        })) => {
            assert_eq!(sender_name, "Solo");
            assert_eq!(text.to_string(), "Nobody here");
        }
        other => panic!("Solo expected own ChatMessage, got {:?}", other),
    }
    // Nothing else
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    assert!(resp.is_err(), "No second message for Solo");
}

#[tokio::test]
async fn broadcast_does_not_cross_rooms() {
    let tx_broker = init();
    let addr_alice: SocketAddr = "127.0.0.1:6004".parse().unwrap();
    let addr_bob: SocketAddr = "127.0.0.1:6005".parse().unwrap();

    let (alice, mut rx_alice) = fake_client(addr_alice, "Alice", "room_a");
    let (bob, mut rx_bob) = fake_client(addr_bob, "Bob", "room_b");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client: alice,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx_alice).await;

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client: bob,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx_bob).await;

    tx_broker
        .send(BrokerEvent::Broadcast {
            sender_addr: addr_alice,
            str_message: "Wrong room!".into(),
            timestamp: test_timestamp(),
        })
        .unwrap();

    // Alice receives own ChatMessage (broadcast goes to everyone in room)
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx_alice.recv()).await;
    match resp {
        Ok(Some(BrokerToClientMsg::ChatMessage { .. })) => {}
        other => panic!("Alice expected own ChatMessage, got {:?}", other),
    }
    // Nothing else for Alice
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx_alice.recv()).await;
    assert!(resp.is_err(), "No second message for Alice");

    // Bob should NOT receive anything
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx_bob.recv()).await;
    assert!(
        resp.is_err(),
        "Bob should not receive messages from a different room"
    );
}

#[tokio::test]
async fn join_room_first_connect_creates_room_success() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:5000".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx).await;

    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: addr,
            room_name: "lounge".into(),
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx).await;
}

#[tokio::test]
async fn join_room_first_connect_duplicate_client_fails() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:5001".parse().unwrap();
    let (client1, mut rx1) = fake_client(addr, "Alice", "games");
    let (client2, mut rx2) = fake_client(addr, "Alice", "other");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client: client1,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx1).await;

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client: client2,
            timestamp: test_timestamp(),
        })
        .unwrap();

    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx2.recv()).await;

    match resp {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::AddedToBroker { status }))) => {
            assert!(!status, "Duplicate FirstConnect should fail");
        }
        other => panic!("Unexpected response on rx2: {:?}", other),
    }
}

#[tokio::test]
async fn join_room_room_move_to_new_room_success() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:5002".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx).await;

    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: addr,
            room_name: "lounge".into(),
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx).await;
}

#[tokio::test]
async fn join_room_room_move_to_existing_room_success() {
    let tx_broker = init();
    let addr_a: SocketAddr = "127.0.0.1:5003".parse().unwrap();
    let addr_b: SocketAddr = "127.0.0.1:5004".parse().unwrap();

    let (client_a, mut rx_a) = fake_client(addr_a, "Alice", "games");
    let (client_b, mut rx_b) = fake_client(addr_b, "Bob", "lounge");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client: client_a,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx_a).await;

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client: client_b,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx_b).await;

    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: addr_b,
            room_name: "games".into(),
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx_b).await;
}

#[tokio::test]
async fn join_room_room_move_unknown_addr_fails() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:5005".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx).await;

    let unknown: SocketAddr = "127.0.0.1:9999".parse().unwrap();
    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: unknown,
            room_name: "games".into(),
            timestamp: test_timestamp(),
        })
        .unwrap();

    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;

    assert!(resp.is_err(), "No response should be sent for unknown addr");
}

#[tokio::test]
async fn join_room_room_move_same_room_fails() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:5006".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");

    tx_broker
        .send(BrokerEvent::AddUserToBroker {
            client,
            timestamp: test_timestamp(),
        })
        .unwrap();
    check_response(&mut rx).await;

    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: addr,
            room_name: "games".into(),
            timestamp: test_timestamp(),
        })
        .unwrap();

    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;

    match resp {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::JoinRoom { status, created }))) => {
            assert!(!status, "Same room move should fail");
            assert!(!created);
        }
        other => panic!("Unexpected response: {:?}", other),
    }
}

// TryFrom conversion tests

mod tryfrom_tests {
    use super::*;
    use common::ws_messages::ServerMessage;

    #[test]
    fn tryfrom_broker_connected_success() {
        let msg = BrokerToClientMsg::Response(BrokerRsp::AddedToBroker { status: true });
        let result = ServerMessage::try_from(msg);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(matches!(
            result,
            ServerMessage::Notification { ref value, .. } if value == "Connected"
        ));
    }

    #[test]
    fn tryfrom_broker_connected_failure() {
        let msg = BrokerToClientMsg::Response(BrokerRsp::AddedToBroker { status: false });
        let result = ServerMessage::try_from(msg).unwrap();
        assert!(matches!(
            result,
            ServerMessage::Error { ref value } if value == "Connection failed"
        ));
    }

    #[test]
    fn tryfrom_broker_joinroom_success_created() {
        let msg = BrokerToClientMsg::Response(BrokerRsp::JoinRoom {
            status: true,
            created: true,
        });
        let result = ServerMessage::try_from(msg).unwrap();
        assert!(matches!(
            result,
            ServerMessage::Notification { ref value, .. } if value == "Room created"
        ));
    }

    #[test]
    fn tryfrom_broker_joinroom_success_joined() {
        let msg = BrokerToClientMsg::Response(BrokerRsp::JoinRoom {
            status: true,
            created: false,
        });
        let result = ServerMessage::try_from(msg).unwrap();
        assert!(matches!(
            result,
            ServerMessage::Notification { ref value, .. } if value == "Room joined"
        ));
    }

    #[test]
    fn tryfrom_broker_joinroom_failure() {
        let msg = BrokerToClientMsg::Response(BrokerRsp::JoinRoom {
            status: false,
            created: false,
        });
        let result = ServerMessage::try_from(msg).unwrap();
        assert!(matches!(
            result,
            ServerMessage::Error { ref value } if value == "Join room failed"
        ));
    }

    #[test]
    fn tryfrom_broker_chat_message() {
        let msg = BrokerToClientMsg::ChatMessage {
            sender: "127.0.0.1:8080".parse().unwrap(),
            sender_name: "alice".to_string(),
            text: "hello".to_string(),
            timestamp: "17/06/2026 18:30:00".to_string(),
        };
        let result = ServerMessage::try_from(msg).unwrap();
        assert!(matches!(
            result,
            ServerMessage::Chat { ref sender, ref message, ref timestamp }
            if sender == "alice" && message == "hello" && timestamp == "17/06/2026 18:30:00"
        ));
    }

    #[test]
    fn tryfrom_broker_notification() {
        let msg = BrokerToClientMsg::UserLogoutNtf {
            text: "user left".to_string(),
            timestamp: "17/06/2026 18:30:00".to_string(),
        };
        let result = ServerMessage::try_from(msg).unwrap();
        assert!(matches!(
            result,
            ServerMessage::UserLogoutNtf { ref value, ref timestamp }
            if value == "user left" && timestamp == "17/06/2026 18:30:00"
        ));
    }
}
