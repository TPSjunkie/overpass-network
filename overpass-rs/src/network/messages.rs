// src/network/messages.rs

use js_sys::Date;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Represents a message sent between nodes in the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// A message sent by a node to request a list of peers.
    PeerDiscoveryRequest,
    /// A message sent by a node to respond to a peer discovery request.
    PeerDiscoveryResponse(Vec<[u8; 32]>),
    /// A message sent by a node to request a list of nodes that are currently in the network.
    NodeDiscoveryRequest,
    /// A message sent by a node to respond to a node discovery request.
    NodeDiscoveryResponse(Vec<[u8; 32]>),
    /// A message sent by a node to request the current state of the network.
    NetworkStateRequest,
    /// A message sent by a node to respond to a network state request.
    NetworkStateResponse(NetworkState),
    /// A message sent by a node to request the current state of a specific node.
    NodeStateRequest([u8; 32]),
    /// A message sent by a node to respond to a node state request.
    NodeStateResponse(NodeState),
    /// A message sent by a node to notify a peer of a suspension.
    NodeSuspended([u8; 32]),
    /// A message sent by a node to request the current state of a specific channel.
    ChannelStateRequest([u8; 32]),
    /// A message sent by a node to respond to a channel state request.
    ChannelStateResponse(ChannelState),
    /// A message sent by a node to request the current state of a specific storage node.
    StorageNodeStateRequest([u8; 32]),
    /// A message sent by a node to respond to a storage node state request.
    StorageNodeStateResponse(StorageNodeState),
    /// A message sent by a node to request the current state of a specific storage node.
    StorageNodeStateUpdateRequest([u8; 32]),
    /// A message sent by a node to respond to a storage node state update request.
    StorageNodeStateUpdateResponse(StorageNodeStateUpdate),
    /// A message used to request a challenge from a node.
    ChallengeRequest([u8; 32]),
    /// A message used to respond to a challenge request.
    ChallengeResponse(crate::network::NetworkMessage),
}

#[wasm_bindgen]
pub fn create_network_message(message_type: &str) -> JsValue {
    let message = match message_type {
        "PeerDiscoveryRequest" => NetworkMessage::PeerDiscoveryRequest,
        // Add other variants here
        _ => return JsValue::NULL,
    };
    serde_wasm_bindgen::to_value(&message).unwrap_or(JsValue::NULL)
}
/// Represents the current state of the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct NetworkState {
    /// The current timestamp.
    pub timestamp: f64,
    /// The current number of nodes in the network.
    pub node_count: u32,
    /// The current number of channels in the network.
    pub channel_count: u32,
    /// The current number of storage nodes in the network.
    pub storage_node_count: u32,
}

#[wasm_bindgen]
impl NetworkState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            timestamp: Date::now(),
            node_count: 0,
            channel_count: 0,
            storage_node_count: 0,
        }
    }
}

/// Represents the current state of a specific node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct StorageNodeStateUpdate {
    pub node_id: Vec<u8>,
}
pub struct StorageNodeState {
    /// The node's ID.
    pub node_id: [u8; 32],
    /// The node's current balance.
    pub balance: u64,
    /// The node's current stake.
    pub stake: u64,
    /// The node's current timestamp.
    pub timestamp: f64,
    /// The node's current status.
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum NodeStatus {
    Online,
    Offline,
    Syncing,
    Error,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct NodeState {
    #[wasm_bindgen(skip)]
    pub node_id: [u8; 32],
    pub timestamp: f64,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct ChannelState {
    #[wasm_bindgen(skip)]
    pub channel_id: [u8; 32],
    pub timestamp: f64,
    pub status: ChannelStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub enum ChannelStatus {
    Open,
    Closed,
    Pending,
}
