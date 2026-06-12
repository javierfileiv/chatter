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
async fn check_response(rx_from_client: &mut UnboundedReceiver<BrokerToClientMsg>) {
    let response_from_broker =
        tokio::time::timeout(std::time::Duration::from_millis(100), rx_from_client.recv()).await;

    match response_from_broker {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::Connect { status }))) => {
            assert!(status, "Broker has not connected client");
            println!("Connect Treated");
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
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::Broadcast { status }))) => {
            assert!(status, "Broker has not broadcasted message");
            println!("Broadcast Treated");
        }
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::Disconnect { status }))) => {
            println!("Disconnect Treated (status: {})", status);
        }
        Ok(Some(BrokerToClientMsg::ChatMessage {
            sender,
            sender_name,
            text,
        })) => {
            println!(
                "Received message from {} (addr:{}): {}",
                sender_name, sender, text
            );
        }
        Ok(Some(BrokerToClientMsg::Notification(text))) => {
            println!("Received notification: {}", text);
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
    let result = tx_broker.send(BrokerEvent::Connect { client });

    assert!(
        result.is_ok(),
        "The Broker should have received the Connected event"
    );
    check_response(&mut rx).await;
}

#[tokio::test]
async fn test_broker_broadcast_success() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");

    tx_broker.send(BrokerEvent::Connect { client }).unwrap();
    check_response(&mut rx).await;

    let result = tx_broker.send(BrokerEvent::Broadcast {
        sender_addr: addr,
        str_message: "Hello World".into(),
    });

    assert!(
        result.is_ok(),
        "The Broker should have received the Broadcast event"
    );
    check_response(&mut rx).await;
}

#[tokio::test]
async fn broadcast_to_multiple_clients_in_same_room() {
    let tx_broker = init();
    let addr_alice: SocketAddr = "127.0.0.1:6001".parse().unwrap();
    let addr_bob: SocketAddr = "127.0.0.1:6002".parse().unwrap();

    let (alice, mut rx_alice) = fake_client(addr_alice, "Alice", "games");
    let (bob, mut rx_bob) = fake_client(addr_bob, "Bob", "games");

    tx_broker
        .send(BrokerEvent::Connect { client: alice })
        .unwrap();
    check_response(&mut rx_alice).await;

    tx_broker
        .send(BrokerEvent::Connect { client: bob })
        .unwrap();
    check_response(&mut rx_bob).await;

    tx_broker
        .send(BrokerEvent::Broadcast {
            sender_addr: addr_alice,
            str_message: "Hi everyone!".into(),
        })
        .unwrap();

    // Alice should receive broadcast ack
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx_alice.recv()).await;
    match resp {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::Broadcast { status }))) => {
            assert!(status, "Broadcast ack should be true");
        }
        other => panic!("Alice expected broadcast ack, got {:?}", other),
    }

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

    tx_broker.send(BrokerEvent::Connect { client }).unwrap();
    check_response(&mut rx).await;

    tx_broker
        .send(BrokerEvent::Broadcast {
            sender_addr: addr,
            str_message: "Nobody here".into(),
        })
        .unwrap();

    // Sender should receive broadcast ack
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    match resp {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::Broadcast { status }))) => {
            assert!(
                status,
                "Broadcast ack should be true even with no other clients"
            );
        }
        other => panic!("Expected broadcast ack, got {:?}", other),
    }

    // No more messages should arrive (sender doesn't get their own message)
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await;
    assert!(
        resp.is_err(),
        "No second message expected, sender should not receive own broadcast"
    );
}

#[tokio::test]
async fn broadcast_does_not_cross_rooms() {
    let tx_broker = init();
    let addr_alice: SocketAddr = "127.0.0.1:6004".parse().unwrap();
    let addr_bob: SocketAddr = "127.0.0.1:6005".parse().unwrap();

    let (alice, mut rx_alice) = fake_client(addr_alice, "Alice", "room_a");
    let (bob, mut rx_bob) = fake_client(addr_bob, "Bob", "room_b");

    tx_broker
        .send(BrokerEvent::Connect { client: alice })
        .unwrap();
    check_response(&mut rx_alice).await;

    tx_broker
        .send(BrokerEvent::Connect { client: bob })
        .unwrap();
    check_response(&mut rx_bob).await;

    tx_broker
        .send(BrokerEvent::Broadcast {
            sender_addr: addr_alice,
            str_message: "Wrong room!".into(),
        })
        .unwrap();

    // Alice gets ack
    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx_alice.recv()).await;
    match resp {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::Broadcast { status }))) => {
            assert!(status);
        }
        other => panic!("Alice expected broadcast ack, got {:?}", other),
    }

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

    tx_broker.send(BrokerEvent::Connect { client }).unwrap();
    check_response(&mut rx).await;

    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: addr,
            room_name: "lounge".into(),
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
        .send(BrokerEvent::Connect { client: client1 })
        .unwrap();
    check_response(&mut rx1).await;

    tx_broker
        .send(BrokerEvent::Connect { client: client2 })
        .unwrap();

    let resp = tokio::time::timeout(std::time::Duration::from_millis(100), rx2.recv()).await;

    match resp {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::Connect { status }))) => {
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

    tx_broker.send(BrokerEvent::Connect { client }).unwrap();
    check_response(&mut rx).await;

    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: addr,
            room_name: "lounge".into(),
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
        .send(BrokerEvent::Connect { client: client_a })
        .unwrap();
    check_response(&mut rx_a).await;

    tx_broker
        .send(BrokerEvent::Connect { client: client_b })
        .unwrap();
    check_response(&mut rx_b).await;

    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: addr_b,
            room_name: "games".into(),
        })
        .unwrap();
    check_response(&mut rx_b).await;
}

#[tokio::test]
async fn join_room_room_move_unknown_addr_fails() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:5005".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");

    tx_broker.send(BrokerEvent::Connect { client }).unwrap();
    check_response(&mut rx).await;

    let unknown: SocketAddr = "127.0.0.1:9999".parse().unwrap();
    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: unknown,
            room_name: "games".into(),
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

    tx_broker.send(BrokerEvent::Connect { client }).unwrap();
    check_response(&mut rx).await;

    tx_broker
        .send(BrokerEvent::JoinRoom {
            sender_addr: addr,
            room_name: "games".into(),
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
