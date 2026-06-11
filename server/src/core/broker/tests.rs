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
            room: room.to_string(),
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

    let result = tx_broker.send(BrokerEvent::Connect {
        client: client.clone(),
    });

    assert!(
        result.is_ok(),
        "The Broker should have received the Connected event"
    );
    check_response(&mut rx).await;

    let test_message = Message::Text("Hello World".into());
    let result = tx_broker.send(BrokerEvent::Broadcast {
        sender_client: client,
        message: test_message,
    });

    assert!(
        result.is_ok(),
        "The Broker should have received the Broadcast event"
    );
    check_response(&mut rx).await;
}
