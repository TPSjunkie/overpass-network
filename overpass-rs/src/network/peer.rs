// src/network/peer.rs

use crate::network::protocol::Message;
use std::net::SocketAddr;
use std::sync::mpsc;

/// Represents a peer in the network.
pub struct Peer {
    pub address: SocketAddr,
    sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
}

impl Peer {
    /// Creates a new peer with the given address.
    pub fn new(address: SocketAddr) -> (Self, mpsc::Sender<Message>, mpsc::Receiver<Message>) {
        let (tx1, rx1) = mpsc::channel();
        let (tx2, rx2) = mpsc::channel();

        let peer = Peer {
            address,
            sender: tx1,
            receiver: rx2,
        };

        (peer, tx2, rx1)
    }

    /// Sends a message to the peer.
    pub async fn send_message(&self, message: Message) -> Result<(), mpsc::SendError<Message>> {
        self.sender.send(message)
    }

    /// Receives a message from the peer.
    pub async fn receive_message(&self) -> Result<Message, mpsc::RecvError> {
        self.receiver.recv()
    }

    /// Returns the peer's address.
    pub fn get_address(&self) -> SocketAddr {
        self.address
    }

    /// Checks if the peer is still connected.
    pub fn is_connected(&self) -> bool {
        self.sender.send(Message::Ping()).is_ok()
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        // Close channels explicitly
        let _ = self.sender.send(Message::Disconnect());

        // Drop the sender and receiver, which will close the channels
        drop(self.sender.clone());
        drop(self.receiver);

        // Log or handle any cleanup errors if needed
        println!("Peer {} disconnected", self.address);
    }
}
