//! WebSocket message protocol definitions
//!
//! This module defines the JSON message types exchanged between client and server
//! over WebSocket connections. All messages are serialized/deserialized using serde.
//!
//! # Protocol Overview
//!
//! Messages use a tagged union format with a `"type"` field:
//!
//! ```json
//! {
//!   "type": "authenticate",
//!   "username": "alice",
//!   "password": "secret",
//!   "room_name": "lobby"
//! }
//! ```
//!
//! # Client → Server Messages ([`ClientMessage`])
//!
//! - `authenticate`: Initial authentication with username, password, and room
//! - `send`: Broadcast a text message to the current room
//! - `join`: Request to join a different room
//! - `logout`: Graceful disconnection
//!
//! # Server → Client Messages ([`ServerMessage`])
//!
//! - `auth_result`: Authentication response with success status
//! - `chat`: Incoming chat message from another user
//! - `notification`: System notifications (room joined, connected, etc.)
//! - `user_logout`: Notification when another user leaves the room
//! - `join_room`: Response to a room join request
//! - `error`: Error messages
//!
//! # Example Flow
//!
//! 1. Client sends `authenticate` message
//! 2. Server responds with `auth_result`
//! 3. Client sends `send` messages to chat
//! 4. Server broadcasts `chat` messages to all users in the room
//! 5. Client sends `logout` to disconnect

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

/// Socket message to join a room
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Room {
    /// The client's username for display
    pub username: String,
    /// Room to join or create
    pub room_name: String,
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
    //Join an existing or create a new room
    #[serde(rename = "join")]
    JoinRoom(Room),
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
    // Server joinroom response
    #[serde(rename = "join_room")]
    JoinRoom {
        success: bool,
        created: bool,
        room_name: String,
    },
    // Server notifies something
    #[serde(rename = "notification")]
    Notification { value: String, timestamp: String },
    // Server notifies user logout in current room
    #[serde(rename = "user_logout")]
    UserLogoutNtf { value: String, timestamp: String },
    // Some error in the server
    #[serde(rename = "error")]
    Error { value: String },
}
