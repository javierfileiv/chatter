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
            user: user.to_string(),
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
        Ok(Some(BrokerToClientMsg::ChatMessage {
            status,
            sender,
            text,
        })) => {
            assert!(status, "Broker issue sending broadcast message");
            println!("Received message from {}: {}", sender, text);
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

#[ignore = "reason: Broadcast not implemented yet and todo! makes it panic"]
#[tokio::test]
async fn test_broker_broadcast_success() {
    let tx_broker = init();
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let (client, mut rx) = fake_client(addr, "Alice", "games");

    let result = tx_broker.send(BrokerEvent::Connect { client });

    assert!(
        result.is_ok(),
        "The Broker should have received the Connected event"
    );
    check_response(&mut rx).await;

    let test_message = Message::Text("Hello World".into());
    let result = tx_broker.send(BrokerEvent::Broadcast {
        sender_addr: addr,
        message: test_message,
    });

    assert!(
        result.is_ok(),
        "The Broker should have received the Broadcast event"
    );
    check_response(&mut rx).await;
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
