use log::info;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

/// Internal broker client
#[derive(Debug, Clone)]
pub struct BrokerClient {
    /// Client username
    pub user: String,
    /// Address of the connected client
    pub addr: SocketAddr,
    /// Room the client is connected to
    pub room: String,
    /// Channel for sending messages to the client
    pub broker_to_client: mpsc::UnboundedSender<BrokerToClientMsg>,
}

/// Internal broker events used by clients
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum BrokerEvent {
    /// Event for when a client connects
    Connect { client: BrokerClient },
    /// Event for when a client disconnects
    Disconnect {
        /// Address of the disconnected client
        addr: SocketAddr,
    },
    /// Event for broadcasting a message to all clients
    Broadcast {
        /// Sender Client
        sender_client: BrokerClient,
        /// Message to broadcast
        message: Message,
    },
    /// Event for joining or createing a new room
    JoinRoom {
        /// Sender Client
        sender_client: BrokerClient,
        /// Name of the room to join/create
        room_name: String,
    },
}

/// Internal broker response to client commands
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum BrokerRsp {
    /// Response for when a client connects
    Connect {
        /// Final status
        status: bool,
    },
    /// Response for broadcasting a message to all clients
    Broadcast {
        /// Final status
        status: bool,
    },
    /// Response for creating a new room
    JoinRoom {
        /// Final status
        status: bool,
        /// Was the room created or joined
        created: bool,
    },
}

/// Internal broker to client response message
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum BrokerToClientMsg {
    /// Internal action response
    Response(BrokerRsp),
    /// A chat message from other user
    ChatMessage {
        /// Message status
        status: bool,
        /// Sender address
        sender: SocketAddr,
        /// Message text
        text: String,
    },
    Notification(String),
}

/// Room basic structure
struct Room {
    /// Room name
    name: String,
    /// Map of clients connected to the room
    /// by IP addr and broadcast channel for the broker to send messages
    clients: Vec<BrokerClient>,
}

/// Initialize the broker and return the channel to send events to it
pub fn init() -> mpsc::UnboundedSender<BrokerEvent> {
    let (tx, rx) = mpsc::unbounded_channel::<BrokerEvent>();
    info!("Broker channel created.");
    tokio::spawn(run(rx));
    tx
}

/// Join a room if it exists, otherwise create it
///
/// If the client is present in some room, it will be removed
/// from it and add it to the requested room
///
/// # Returns: tuple of (success, room_created), if room_created == false
/// means the room existed before
fn join_room(_rooms: &mut Vec<Room>, client: BrokerClient, room_name: String) -> (bool, bool) {
    info!("{} joins the room {}", client.addr, room_name);
    (true, false)
}

/// Register a client in the broker internal list
fn register_client(rooms: &mut Vec<Room>, client: BrokerClient) {
    info!(
        "Adding client: {} ({}) to room: {}",
        client.user, client.addr, client.room
    );

    if let Some(room) = rooms.iter_mut().find(|r| r.name == client.room) {
        room.clients.push(client);
    } else {
        let new_room = Room {
            name: client.room.clone(),
            clients: vec![client],
        };
        rooms.push(new_room);
    }
}

/// Run the broker event loop
pub async fn run(mut rx_events: mpsc::UnboundedReceiver<BrokerEvent>) {
    let mut rooms: Vec<Room> = vec![];

    while let Some(event) = rx_events.recv().await {
        match event {
            BrokerEvent::Connect { client } => {
                let reply_channel = client.broker_to_client.clone();
                info!("Broker: Connected: {} {}", client.user, client.addr);

                register_client(&mut rooms, client);

                let rsp = BrokerToClientMsg::Response(BrokerRsp::Connect { status: true });
                let _ = reply_channel.send(rsp);
            }
            BrokerEvent::Broadcast {
                sender_client,
                message,
            } => {
                info!("Broker: Broadcast: {} {}", sender_client.addr, message);

                let _rsp = BrokerToClientMsg::Response(BrokerRsp::Broadcast { status: true });
                todo!("Implement fn to broadcast to all connected clients");
            }
            BrokerEvent::JoinRoom {
                sender_client,
                room_name,
            } => {
                let (status, created) = join_room(&mut rooms, sender_client, room_name);
                let _rsp = BrokerToClientMsg::Response(BrokerRsp::JoinRoom { status, created });
                todo!("Implement fn to join room send Notification message back to broker client");
            }
            BrokerEvent::Disconnect { addr } => {
                info!("Broker: Disconnect {addr} received");
            }
        }
    }

    info!("Broker closed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::broker;
    use tokio::sync::mpsc::UnboundedReceiver;

    async fn test_response(rx_from_client: &mut UnboundedReceiver<broker::BrokerToClientMsg>) {
        let response_from_broker =
            tokio::time::timeout(std::time::Duration::from_millis(100), rx_from_client.recv())
                .await;

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
}
