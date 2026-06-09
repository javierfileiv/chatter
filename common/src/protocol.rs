/// # Protocol
///
/// https://www.carolinemorton.co.uk/blog/rust-error-handling-anyhow-thiserror/
///
/// Common protocol definitions for the chat application.
/// Represents the current connection state of a client.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientState {
    /// Client is not connected to any server.
    Disconnected,
    /// Client is in the process of establishing a connection.
    Connecting,
    /// Client has successfully authenticated and is connected.
    Authenticated,
}

/// Represents a client/user in the chat server system.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Client {
    /// Unique identifier for the client (0 = unassigned).
    uid: i32,
    /// The client's username for display and login.
    username: String,
    /// The client's password (stored in plain text - TODO: hash this).
    password: String,
    /// Current connection state of the client.
    state: ClientState,
}

// Client struct implementation.
impl Client {
    /// Creates a new Client instance with the given username and password.
    /// The new client starts in the Disconnected state with uid = 0.
    pub fn new(username: &str, password: &str) -> Self {
        Client {
            uid: 0,
            username: username.to_string(),
            password: password.to_string(),
            state: ClientState::Disconnected,
        }
    }
}

/// Message types that can be sent between client and server.
#[derive(Debug, Clone)]
pub enum Message {
    /// Login request containing client credentials.
    Login(Client),
    /// Request to disconnect from the server.
    Disconnect(Client),
    /// Request to join a room.
    JoinRoom,
    /// Request to leave a room.
    LeaveRoom,
    /// Request to list all available rooms.
    ListRooms,
    /// Text message to be sent to other clients.
    SendMessage(String),
}

mod errors {
    /// Message types that can be sent between client and server.
    #[derive(Debug, Clone)]
    pub enum ServerError {
        InvalidCredentials(String),
        AlreadyLoggedIn(String),
        ConnectionFailed(String),
        ServerFull(String),
        TextFormattingError(String),
        UnknownError(String),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_user() {
        let user = Client::new("john_doe", "secret_password");
        assert_eq!(user.username, "john_doe");
        assert_eq!(user.password, "secret_password");
    }

    #[test]
    fn create_another_user() {
        let user = Client::new("jane_doe", "another_password");

        assert_eq!(user.username, "jane_doe");
        assert_eq!(user.password, "another_password");
    }
}
