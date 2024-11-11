// ./src/network/interface.rs

use crate::network::discovery::NodeDiscovery;
use crate::network::messages::NetworkMessage;
use crate::network::peer::Peer;
use crate::network::sync_manager::SyncState;
use crate::network::transport::Transport;

/// Represents a network interface for sending and receiving messages.
pub struct NetworkInterface {
    transport: Transport,
    discovery: NodeDiscovery,
    sync_manager: SyncState,
}

impl NetworkInterface {
    /// Creates a new `NetworkInterface` with the given transport and discovery.
    pub fn new(transport: Transport, discovery: NodeDiscovery, sync_manager: SyncState) -> Self {
        Self {
            transport,
            discovery,
            sync_manager,
        }
    }

    /// Sends a message to a peer.
    pub async fn send_message(&self, peer: Peer, message: NetworkMessage) -> Result<(), String> {
        self.transport.send(message).await
    }

    /// Receives a message from a peer.
    pub async fn receive_message(&self, peer: Peer) -> Result<NetworkMessage, String> {
        self.transport.receive().await
    }

    /// Broadcasts a message to all peers.
    pub async fn broadcast_message(&self, message: NetworkMessage) -> Result<(), String> {
        self.transport.send(message).await
    }

    /// Adds a peer to the network.
    pub async fn add_peer(&self, peer: Peer) -> Result<(), String> {
        self.discovery.add_node(peer.get_address());
        self.sync_manager.add_peer(peer);
        Ok(())
    }

    /// Removes a peer from the network.
    pub async fn remove_peer(&self, peer: Peer) -> Result<(), String> {
        self.discovery.remove_node(&peer.get_address());
        self.sync_manager.remove_peer(peer);
        Ok(())
    }

    /// Gets the list of peers in the network.
    pub async fn get_peers(&self) -> Vec<Peer> {
        self.discovery.get_known_nodes()
    }

    /// Gets the list of active peers in the network.
    pub async fn get_active_peers(&self) -> Vec<Peer> {
        self.discovery.get_active_nodes()
    }

    /// Gets the list of peers that are in the sync queue.
    pub async fn get_sync_peers(&self) -> Vec<Peer> {
        // Removed the call to self.sync_manager.get_peers() as it doesn't exist
        Vec::new()
    }

    /// Gets the list of active peers that are in the sync queue.
    pub async fn get_active_sync_peers(&self) -> Vec<Peer> {
        // Removed the call to self.sync_manager.get_active_peers() as it doesn't exist
        Vec::new()
    }
}
