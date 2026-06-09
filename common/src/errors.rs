//! # Errors
//! Common error definitions for the chat application (client/server).
//!
//! Ref: https://www.carolinemorton.co.uk/blog/rust-error-handling-anyhow-thiserror/
//!
/// Message types that can be sent between client and server.
#[derive(Debug, Clone)]
pub enum _ServerError {
    InvalidCredentials(String),
    AlreadyLoggedIn(String),
    ConnectionFailed(String),
    ServerFull(String),
    TextFormattingError(String),
    UnknownError(String),
}
