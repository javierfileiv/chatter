#[cfg(test)]
mod tests;

use log::{error, info, warn};
use std::collections::HashMap;
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
    /// Room the client has joined to
    pub room_name: String,
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
        /// Sender address
        sender_addr: SocketAddr,
        /// Message to broadcast
        message: Message,
    },
    /// Event for joining or createing a new room
    JoinRoom {
        /// Sender address
        sender_addr: SocketAddr,
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

/// Internal enumeration for join a room
/// Used for an existing user to move into another room or a new user joining a room
enum JoinRoomType {
    FirstConnect(BrokerClient),
    RoomMove(SocketAddr),
}

/// Data structure for Broker context
/// Used to hold rooms/client data
struct Broker {
    // Client hashmap indexed by IP addr
    clients: HashMap<SocketAddr, BrokerClient>,

    // Address ip list indexed by room name
    rooms: HashMap<String, Vec<SocketAddr>>,
}

/// Instantiate a new Broker context
impl Broker {
    fn new() -> Broker {
        Broker {
            clients: HashMap::new(),
            rooms: HashMap::new(),
        }
    }
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
/// # Returns: tuple of (success, room_created), if room_created is false means the room existed before
fn join_room(ctx: &mut Broker, joining_client: JoinRoomType, room_name: String) -> (bool, bool) {
    let client_addr = match joining_client {
        JoinRoomType::FirstConnect(mut client) => {
            info!(
                "Adding user {} (addr {}) to broker",
                client.user, client.addr
            );

            if ctx.clients.contains_key(&client.addr) {
                warn!(
                    "Client {} existed in broker, invalid FirstConnect",
                    client.user
                );
                return (false, false);
            }

            let addr = client.addr;
            info!(
                "Adding new client {} (addr {}) in broker",
                client.user, addr
            );
            client.room_name = room_name.clone();

            ctx.clients.insert(addr, client);
            addr
        }
        JoinRoomType::RoomMove(addr) => {
            let client = match ctx.clients.get_mut(&addr) {
                Some(c) => c,
                None => {
                    warn!("Address {} not exist in Broker, invalid RoomMove", addr);
                    return (false, false);
                }
            };
            if client.room_name == room_name {
                info!("Client {} already in room {}", client.user, room_name);
                return (false, false);
            }
            info!(
                "Preparing to shift client {} from room {} to room{}",
                client.user, client.room_name, room_name
            );
            // Remove client addr from client.room_name
            if let Some(addr_list) = ctx.rooms.get_mut(&client.room_name) {
                addr_list.retain(|a| *a != addr);
            }
            // change room
            client.room_name = room_name.clone();
            // remove addr from room
            addr
        }
    };

    let mut room_created = false;

    ctx.rooms
        .entry(room_name)
        .and_modify(|addr_list| addr_list.push(client_addr))
        .or_insert_with(|| {
            room_created = true;
            vec![client_addr]
        });
    (true, room_created)
}

/// Register a client in the broker internal list
fn register_client(ctx: &mut Broker, client: BrokerClient) -> bool {
    info!(
        "Registering client: {} (addr:{}) to room: {}",
        client.user, client.addr, client.room_name
    );
    let user_name = client.user.clone();
    let room_name = client.room_name.clone();
    let (status, _) = join_room(ctx, JoinRoomType::FirstConnect(client), room_name);

    if !status {
        error!("Error adding new user {}", user_name);
    }
    status
}

/// Run the broker event loop
pub async fn run(mut rx_events: mpsc::UnboundedReceiver<BrokerEvent>) {
    let mut ctx = Broker::new();

    while let Some(event) = rx_events.recv().await {
        match event {
            BrokerEvent::Connect { client } => {
                let reply_channel = client.broker_to_client.clone();
                info!("Broker: Connected: {} {}", client.user, client.addr);

                let status = register_client(&mut ctx, client);
                let rsp = BrokerToClientMsg::Response(BrokerRsp::Connect { status });
                reply_channel.send(rsp).unwrap();
            }
            BrokerEvent::Broadcast {
                sender_addr,
                message,
            } => {
                info!("Broker: Broadcast: {} {}", sender_addr, message);

                let _rsp = BrokerToClientMsg::Response(BrokerRsp::Broadcast { status: true });
                todo!("Implement fn to broadcast to all connected clients");
            }
            BrokerEvent::JoinRoom {
                sender_addr,
                room_name,
            } => {
                let (status, created) =
                    join_room(&mut ctx, JoinRoomType::RoomMove(sender_addr), room_name);
                let rsp = BrokerToClientMsg::Response(BrokerRsp::JoinRoom { status, created });
                if let Some(client) = ctx.clients.get(&sender_addr) {
                    let _ = client.broker_to_client.send(rsp);
                } else {
                    info!("JoinRoom: no client at {sender_addr}, dropping response");
                }
            }
            BrokerEvent::Disconnect { addr } => {
                info!("Broker: Disconnect {addr} received");
            }
        }
    }

    info!("Broker closed")
}
