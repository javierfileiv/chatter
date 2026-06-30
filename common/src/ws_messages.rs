use serde::{Deserialize, Serialize};

/// Socket message to authenticate and register a new client
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticateUser {
    /// The client's username for login
    pub username: String,
    /// The client's password
    pub password: String,
    /// The client's room name to join or create if it doesn't exist
    pub room_name: String,
}

/// Socket message to broadcast content
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendMessage {
    /// The client's username for display
    pub username: String,
    /// The message to send to the room
    pub message: String,
}

/// Socket message to logout client
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Logout {
    /// Optional message to broadcast before leaving
    pub message: String,
}

/// Messages the client can send to the server via WebSocket
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum ClientMessage {
    // Authentication msg
    #[serde(rename = "authenticate")]
    Authenticate(AuthenticateUser),
    // Broadcast msg
    #[serde(rename = "send")]
    Broadcast(SendMessage),
    // Logout msg
    #[serde(rename = "logout")]
    Logout(Logout),
}

/// Messages the server can send to the client via WebSocket
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum ServerMessage {
    // Authentication response
    #[serde(rename = "auth_result")]
    AuthResult { success: bool, msg: Option<String> },
    // Broadcast response
    #[serde(rename = "chat")]
    Chat {
        sender: String,
        message: String,
        timestamp: String,
    },
    // Server communicates something
    #[serde(rename = "notification")]
    Notification { value: String, timestamp: String },
    // Some error in the server
    #[serde(rename = "error")]
    Error { value: String },
}
