// src/network/sync.rs

use crate::network::protocol::Message;
use std::sync::mpsc;

/// Represents the synchronization state.
pub struct SyncState {
    // Define synchronization state fields here
}

/// Manages synchronization between peers.
pub struct Sync {
    pub state: SyncState,
    pub sender: mpsc::Sender<Message>,
    pub receiver: mpsc::Receiver<Message>,
}

impl Sync {
    /// Creates a new `Sync` instance.
    pub fn new(state: SyncState) -> Self {
        let (tx, rx) = mpsc::channel();
        Sync {
            state,
            sender: tx,
            receiver: rx,
        }
    }

    /// Sends a message to the sync channel.
    pub fn send_message(&self, message: Message) {
        let _ = self.sender.send(message);
    }

    /// Receives a message from the sync channel.
    pub fn receive_message(&mut self) -> Option<Message> {
        self.receiver.recv().ok()
    }
}
