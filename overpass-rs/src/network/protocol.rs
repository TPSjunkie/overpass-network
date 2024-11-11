// src/network/protocol.rs

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Enum representing different types of network messages.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum NetworkMessage {
    TextMessage(String),
    Join,
    Leave,
    Ping,
    Pong,
    NodeResume,
    NodeSuspension,
    Error(String),
    Acknowledgment,
    FileTransfer {
        filename: String,
        data: Vec<u8>,
    },
    UserStatus {
        username: String,
        status: UserStatus,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum UserStatus {
    Online,
    Away,
    Busy,
    Offline,
}

/// Struct representing a message with sender information.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub sender: SocketAddr,
    pub content: NetworkMessage,
    pub timestamp: u64,
    pub message_id: u32,
}

impl Message {
    /// Creates a new message with the given sender and content.
    pub fn new(sender: SocketAddr, content: NetworkMessage) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        static mut MESSAGE_COUNTER: u32 = 0;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let message_id = unsafe {
            MESSAGE_COUNTER += 1;
            MESSAGE_COUNTER
        };

        Message {
            sender,
            content,
            timestamp,
            message_id,
        }
    }

    /// Serializes the message to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    /// Deserializes a message from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(bytes)
    }

    /// Gets the age of the message in seconds
    pub fn age(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.timestamp
    }

    /// Checks if the message is expired (older than the given seconds)
    pub fn is_expired(&self, max_age_secs: u64) -> bool {
        self.age() > max_age_secs
    }

    /// Creates a response message to this message
    pub fn create_response(&self, sender: SocketAddr, content: NetworkMessage) -> Self {
        Message::new(sender, content)
    }

    /// Creates an acknowledgment response
    pub fn create_ack(&self, sender: SocketAddr) -> Self {
        self.create_response(sender, NetworkMessage::Acknowledgment)
    }

    /// Creates an error response
    pub fn create_error(&self, sender: SocketAddr, error: String) -> Self {
        self.create_response(sender, NetworkMessage::Error(error))
    }
}
