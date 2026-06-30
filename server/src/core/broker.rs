#[cfg(test)]
mod broker_tests;

use chrono::Local;
use common::ws_messages::ServerMessage;
use log::{error, info, warn};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::sync::mpsc;

/// Internal broker client
#[derive(Debug, Clone)]
pub struct BrokerClient {
    /// Client username
    pub str_id: String,
    /// Address of the connected client
    pub addr: SocketAddr,
    /// Room the client has joined to
    pub room_name: String,
    /// Channel for sending messages to the client
    pub broker_to_client: mpsc::UnboundedSender<BrokerToClientMsg>,
}

impl BrokerClient {
    pub fn send_response(&self, rsp: BrokerRsp) {
        if let Err(e) = self
            .broker_to_client
            .send(BrokerToClientMsg::new_response(rsp))
        {
            error!(
                "Failed to send response to client: {} ( addr {}): {}",
                self.str_id, self.addr, e
            );
        }
    }
    pub fn send_message(&self, sender_client: &BrokerClient, msg: &str, timestamp: &str) {
        info!(
            "Broadcasting message from {} at {}",
            sender_client.str_id, timestamp
        );
        if let Err(e) = self
            .broker_to_client
            .send(BrokerToClientMsg::new_chat_message(
                sender_client.addr,
                sender_client.str_id.clone(),
                msg.to_owned(),
                timestamp.to_owned(),
            ))
        {
            error!(
                "Failed to send broadcast from client ({} (addr:{})) to client ({} (addr:{}) : {}",
                sender_client.str_id, sender_client.addr, self.str_id, self.addr, e
            );
        }
    }
    pub fn send_notification(&self, text: &str, timestamp: &str) {
        if let Err(e) = self.broker_to_client.send(BrokerToClientMsg::Notification {
            text: text.to_owned(),
            timestamp: timestamp.to_owned(),
        }) {
            error!(
                "Failed to send notification to client: {} (addr {}): {}",
                self.str_id, self.addr, e
            );
        }
    }
}
/// Internal broker events used by clients
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum BrokerEvent {
    /// Event when user is added to broker list
    AddUserToBroker {
        client: BrokerClient,
        /// Timestamp when the message was received
        timestamp: String,
    },
    /// Event for when a client disconnects
    Disconnect {
        /// Address of the disconnected client
        addr: SocketAddr,
        /// Timestamp when the message was received
        timestamp: String,
    },
    /// Event for broadcasting a message to all clients
    Broadcast {
        /// Sender address
        sender_addr: SocketAddr,
        /// Message to broadcast in String format
        str_message: String,
        /// Timestamp when the message was received
        timestamp: String,
    },
    /// Event for joining or createing a new room
    JoinRoom {
        /// Sender address
        sender_addr: SocketAddr,
        /// Name of the room to join/create
        room_name: String,
        /// Timestamp when the message was received
        timestamp: String,
    },
}

/// Internal broker response to client commands
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum BrokerRsp {
    /// Response for successful connection to broker
    AddedToBroker {
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
        /// Sender address
        sender: SocketAddr,
        /// Sender user name
        sender_name: String,
        /// Tungstenite Message for WebSocket
        text: String,
        /// Timestamp when the message was received
        timestamp: String,
    },
    Notification {
        text: String,
        timestamp: String,
    },
}

impl BrokerToClientMsg {
    pub fn new_response(rsp: BrokerRsp) -> Self {
        Self::Response(rsp)
    }

    pub fn new_chat_message(
        sender: SocketAddr,
        sender_name: String,
        text: String,
        timestamp: String,
    ) -> Self {
        Self::ChatMessage {
            sender,
            sender_name,
            text,
            timestamp,
        }
    }
}

impl TryFrom<BrokerToClientMsg> for ServerMessage {
    type Error = String;

    fn try_from(msg: BrokerToClientMsg) -> Result<Self, String> {
        let timestamp = Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
        match msg {
            BrokerToClientMsg::Response(BrokerRsp::AddedToBroker { status }) => {
                if status {
                    Ok(ServerMessage::Notification {
                        value: "Connected".to_string(),
                        timestamp,
                    })
                } else {
                    Ok(ServerMessage::Error {
                        value: "Connection failed".to_string(),
                    })
                }
            }
            BrokerToClientMsg::Response(BrokerRsp::JoinRoom { status, created }) => {
                if status {
                    let action = if created { "created" } else { "joined" };
                    Ok(ServerMessage::Notification {
                        value: format!("Room {}", action),
                        timestamp,
                    })
                } else {
                    Ok(ServerMessage::Error {
                        value: "Join room failed".to_string(),
                    })
                }
            }
            BrokerToClientMsg::ChatMessage {
                sender_name,
                text,
                timestamp,
                ..
            } => Ok(ServerMessage::Chat {
                sender: sender_name,
                message: text,
                timestamp,
            }),
            BrokerToClientMsg::Notification { text, timestamp } => {
                Ok(ServerMessage::Notification {
                    value: text,
                    timestamp,
                })
            }
        }
    }
}

impl TryFrom<ServerMessage> for BrokerToClientMsg {
    type Error = String;

    fn try_from(msg: ServerMessage) -> Result<Self, Self::Error> {
        match msg {
            ServerMessage::Chat {
                sender,
                message,
                timestamp,
            } => Ok(BrokerToClientMsg::ChatMessage {
                sender: SocketAddr::from(([0, 0, 0, 0], 0)),
                sender_name: sender,
                text: message,
                timestamp,
            }),
            ServerMessage::Notification { value, timestamp } => {
                Ok(BrokerToClientMsg::Notification {
                    text: value,
                    timestamp,
                })
            }
            ServerMessage::Error { value } => Ok(BrokerToClientMsg::Notification {
                text: format!("Error: {}", value),
                timestamp: String::new(),
            }),
            _ => Err("Unsupported ServerMessage variant for broker".to_string()),
        }
    }
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
                client.str_id, client.addr
            );

            if ctx.clients.contains_key(&client.addr) {
                warn!(
                    "Client {} existed in broker, invalid FirstConnect",
                    client.str_id
                );
                return (false, false);
            }

            let addr = client.addr;
            info!(
                "Adding new client {} (addr {}) in broker",
                client.str_id, addr
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
                info!("Client {} already in room {}", client.str_id, room_name);
                return (false, false);
            }
            info!(
                "Preparing to shift client {} from room {} to room{}",
                client.str_id, client.room_name, room_name
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

/// Add a client in the broker internal list
fn add_user_to_broker(ctx: &mut Broker, client: BrokerClient) -> bool {
    info!(
        "Adding client: {} (addr:{}) to room: {}",
        client.str_id, client.addr, client.room_name
    );
    let user_name = client.str_id.clone();
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
            BrokerEvent::AddUserToBroker {
                client: user,
                timestamp,
            } => {
                info!(
                    "Broker: Adding user {} ({}) to broker at {}",
                    user.str_id, user.addr, timestamp
                );

                let rsp_channel = user.broker_to_client.clone();
                let status = add_user_to_broker(&mut ctx, user);
                if let Err(e) =
                    rsp_channel.send(BrokerToClientMsg::new_response(BrokerRsp::AddedToBroker {
                        status,
                    }))
                {
                    error!("Failed to send AddedToBroker reply to client: {}", e);
                }
            }
            BrokerEvent::Broadcast {
                sender_addr,
                str_message,
                timestamp,
            } => {
                if let Some(sender_client) = ctx.clients.get(&sender_addr) {
                    if let Some(addr_list) = ctx.rooms.get(&sender_client.room_name) {
                        for &addr in addr_list.iter() {
                            let broadcast_client = ctx.clients.get(&addr).unwrap();
                            broadcast_client.send_message(sender_client, &str_message, &timestamp);
                        }
                    } else {
                        error!(
                            "This can't happen, room/addr_list was checked on Connect/JoinRoom events"
                        );
                    }
                }
            }
            BrokerEvent::JoinRoom {
                sender_addr,
                room_name,
                timestamp,
            } => {
                info!(
                    "Broker: JoinRoom: {} {} at {}",
                    sender_addr, room_name, timestamp
                );
                let (status, created) =
                    join_room(&mut ctx, JoinRoomType::RoomMove(sender_addr), room_name);
                if let Some(client) = ctx.clients.get(&sender_addr) {
                    client.send_response(BrokerRsp::JoinRoom { status, created });
                } else {
                    warn!("JoinRoom: no client at {sender_addr}, dropping response");
                }
            }
            BrokerEvent::Disconnect { addr, timestamp } => {
                info!("Broker: Disconnect {addr} received at {timestamp}");

                // Get client info before removing.
                let disconnecting_client = match ctx.clients.remove(&addr) {
                    Some(client) => {
                        info!(
                            "Removed client {} (addr {}) from broker",
                            client.str_id, addr
                        );
                        client
                    }
                    None => {
                        warn!("Disconnect: client at {addr} not found in broker");
                        return;
                    }
                };

                let username = disconnecting_client.str_id;
                let room_name = disconnecting_client.room_name.clone();

                // Remove client from their room's address list.
                if let Some(addr_list) = ctx.rooms.get_mut(&room_name) {
                    addr_list.retain(|a| *a != addr);

                    // Notify remaining clients in the room.
                    let notification = format!("{} has left the room", username);
                    for &room_addr in addr_list.iter() {
                        if let Some(room_client) = ctx.clients.get(&room_addr) {
                            room_client.send_notification(&notification, &timestamp);
                        }
                    }

                    // Remove empty rooms just in case!
                    if addr_list.is_empty() {
                        ctx.rooms.remove(&room_name);
                        info!("Room '{}' is now empty, removing it", room_name);
                    }
                }
            }
        }
    }

    info!("Broker closed")
}
