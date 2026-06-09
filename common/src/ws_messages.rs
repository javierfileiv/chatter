use serde::{Deserialize, Serialize};

/// Socket message to authenticate and register a new client
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticateUser {
    /// The client's username for login
    username: String,
    /// The client's password
    password: String,
    /// The client's room name to join or create if it doesn't exist
    room_name: String,
}

/// Socket message to broadcast content
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendMessage {
    /// The client's username for display
    username: String,
    /// The message to send to the room.
    message: String,
}

/// Socket message to logout client
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Logout {
    /// The message to send to the room.
    message: String,
}
