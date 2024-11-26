// ./src/core/state/state_oc_data.rs
use serde::{Deserialize, Serialize};
/// Represents the state of an Overpass Channel
/// This includes the balance, nonce, and sequence number of the channel
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PrivateChannelState {
    pub balance: u64,
    pub nonce: u64,
    pub sequence_number: u64,
    pub merkle_root: [u8; 32],
    pub channel_id: [u8; 32],
    pub wallet_id: [u8; 32],
}

impl Default for PrivateChannelState {
    fn default() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            sequence_number: 0,
            merkle_root: [0; 32],
            channel_id: [0; 32],
            wallet_id: [0; 32],
        }
    }
}

// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PublicChannelState {
    pub balance: u64,
    pub nonce: u64,
    pub sequence_number: u64,
    pub merkle_root: [u8; 32],
    pub channel_id: [u8; 32],
    pub wallet_id: [u8; 32],
}

impl Default for PublicChannelState {
    fn default() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            sequence_number: 0,
            merkle_root: [0; 32],
            channel_id: [0; 32],
            wallet_id: [0; 32],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PrivateChannelStateUpdate {
    pub balance: u64,
    pub nonce: u64,
    pub sequence_number: u64,
    pub merkle_root: [u8; 32],
    pub channel_id: [u8; 32],
    pub wallet_id: [u8; 32],
}

impl Default for PrivateChannelStateUpdate {
    fn default() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            sequence_number: 0,
            merkle_root: [0; 32],
            channel_id: [0; 32],
            wallet_id: [0; 32],
        }
    }
}
