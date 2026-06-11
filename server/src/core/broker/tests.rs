#[cfg(test)]
use crate::core::broker::*;
use tokio::sync::mpsc::UnboundedReceiver;

async fn test_response(rx_from_client: &mut UnboundedReceiver<BrokerToClientMsg>) {
    let response_from_broker =
        tokio::time::timeout(std::time::Duration::from_millis(100), rx_from_client.recv()).await;

    match response_from_broker {
        Ok(Some(BrokerToClientMsg::Response(BrokerRsp::Connect { status }))) => {
            assert!(status, "Broker has not connect client");
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
async fn test_broker_connection() {
    let tx_broker = init();

    // Create a fake client with its own reception channel
    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let (tx_to_client, mut rx_from_client) = mpsc::unbounded_channel();
    let username = "Alice".to_string();
    let room = "games".to_string();

    let broker_client = BrokerClient {
        user: username,
        addr,
        room,
        broker_to_client: tx_to_client,
    };

    let result = tx_broker.send(BrokerEvent::Connect {
        client: broker_client,
    });

    assert!(
        result.is_ok(),
        "The Broker should have received the Connected event"
    );
    test_response(&mut rx_from_client).await;
}

#[ignore = "reason: Broadcast not implemented yet and todo! makes it panic"]
#[tokio::test]
async fn test_broker_broadcast() {
    let tx_broker = init();
    let addr_alice: SocketAddr = "127.0.0.1:1111".parse().unwrap();
    let (tx_to_client, mut rx_from_client) = mpsc::unbounded_channel();
    let room = "games".to_string();

    let alice = BrokerClient {
        user: "Alice".to_string(),
        addr: addr_alice,
        broker_to_client: tx_to_client,
        room,
    };

    // Connect Alice
    let result = tx_broker.send(BrokerEvent::Connect {
        client: alice.clone(),
    });
    assert!(
        result.is_ok(),
        "The Broker should have received the Connected event"
    );
    test_response(&mut rx_from_client).await;

    // Simulate sending a Broadcast message
    let test_message = Message::Text("Hello World".into());
    let result = tx_broker.send(BrokerEvent::Broadcast {
        sender_client: alice,
        message: test_message,
    });

    assert!(
        result.is_ok(),
        "The Broker should have received the Broadcast event"
    );
    test_response(&mut rx_from_client).await;
}
