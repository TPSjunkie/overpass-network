// ./src/core/zkps/transaction_manager.rs

//! Transaction Manager Module
//!
//! This module provides a high-level interface for managing transactions in the Overpass Channels system.
//! It includes functionality for creating, verifying, and managing transactions using the ZK proof system.
//!
//! # Key Structures
//! - `TransactionManager`: High-level interface for managing transactions
//! - `TransactionOCData`: Represents a transaction in the Overpass Channels system
//! - `TransactionType`: Enumeration of different types of transactions
//! - `WalletExtension`: Represents the wallet extension state
//! - `ChannelConfig`: Configuration for managing channels
//! - `ZkProof`: Represents a zero-knowledge proof
//!
//! # Main Functions
//! - `create_channel_transaction`: Creates a new channel transaction
//! - `update_channel_state`: Updates the state of a channel
//! - `verify_proof`: Verifies the validity of a given proof
//!
//! # Usage
//! This module is primarily used by the JavaScript/TypeScript interface to interact with the
//! underlying Rust implementation. It provides a high-level abstraction for managing transactions
//! and proofs in the Overpass Channels system.
//!
use crate::core::client::{transaction::transaction_types::TransactionType, wallet_extension::wallet_extension_contract};
use crate::core::client::wallet_extension::user::ChannelState;
use crate::core::client::transaction::transaction_oc_data::TransactionOCData;
use crate::core::client::channel::channel_contract::ChannelConfig;
use crate::core::client::wallet_extension::wallet_extension_types::WalletExtension;
use crate::core::state::state_manager::StateManager;
use crate::core::zkps::proof::ZkProof;
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;
use serde::Serialize;

#[wasm_bindgen]
#[derive(Clone)]
pub struct TransactionManager {
    state_manager: Arc<RwLock<StateManager>>,
    wallet_extension: Arc<RwLock<WalletExtension>>,
}

#[wasm_bindgen]
impl TransactionManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsValue> {
        Ok(Self {
            state_manager: Arc::new(RwLock::new(StateManager::new())),
            wallet_extension: Arc::new(RwLock::new(WalletExtension::new()?)),
        })
    }

    pub async fn create_channel_transaction(
        &self,
        sender: &[u8],
        recipient: &[u8],
        deposit: u64,
        config: JsValue,
    ) -> Result<JsValue, JsValue> {
        let mut wallet = self.wallet_extension.write()
            .map_err(|e| JsValue::from_str(&format!("Failed to acquire wallet lock: {:?}", e)))?;

        // Convert arrays to fixed length
        let sender_bytes: [u8; 32] = sender.try_into()
            .map_err(|_| JsValue::from_str("Invalid sender address length"))?;
        let recipient_bytes: [u8; 32] = recipient.try_into()
            .map_err(|_| JsValue::from_str("Invalid recipient address length"))?;

        // Convert config to ChannelConfig
        let channel_config = serde_wasm_bindgen::from_value::<ChannelConfig>(config)
            .map_err(|e| JsValue::from_str(&format!("Invalid channel config: {:?}", e)))?;

        // Create channel and get channel ID
        let channel_id = wallet.create_channel(
            wallet_extension_contract::ByteArray32Local(sender_bytes),
            wallet_extension_contract::ByteArray32Local(recipient_bytes),
            deposit,
            &channel_config
        ).await
        .map_err(|e| JsValue::from_str(&format!("Failed to create channel: {:?}", e)))?;

        // Get state transition proof
        let state_manager = self.state_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Failed to acquire state manager lock: {:?}", e)))?;

        // Get current state hash
        let state_hash = wallet.get_root_hash(&channel_id)
            .map_err(|e| JsValue::from_str(&format!("Failed to get state hash: {:?}", e)))?;

        let proof = state_manager.create_state_transition_proof(
            &wallet.root_hash,
            &wallet_extension_contract::ByteArray32Local(channel_id),
            &[0u8; 32],
            &state_hash,
        ).map_err(|e| JsValue::from_str(&format!("Failed to create state transition proof: {:?}", e)))?;

        // Create transaction
        let channel = wallet.channels.get(&channel_id)
            .ok_or_else(|| JsValue::from_str("Channel not found"))?
            .clone();

        #[derive(Serialize)]
        struct TransactionWrapper {
            transaction: TransactionOCData<WalletExtension>
        }

        let transaction = TransactionOCData::new(
            channel_id,
            TransactionType::ChannelOpen,
            self.wallet_extension.clone(),
            channel,
            ZkProof::from(proof),
        );

        let wrapper = TransactionWrapper { transaction };

        // Serialize and return
        serde_wasm_bindgen::to_value(&wrapper)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize transaction: {:?}", e)))
    }

    pub async fn update_channel_state(
        &self,
        channel_id: &[u8],
        new_state: JsValue,
    ) -> Result<JsValue, JsValue> {
        // Convert channel ID
        let channel_id: [u8; 32] = channel_id.try_into()
            .map_err(|_| JsValue::from_str("Invalid channel ID length"))?;

        // Get wallet and channel
        let wallet = self.wallet_extension.read()
            .map_err(|e| JsValue::from_str(&format!("Failed to acquire wallet lock: {:?}", e)))?;

        let channel = wallet.get_channel(&channel_id)
            .ok_or_else(|| JsValue::from_str("Channel not found"))?;

        let channel_state: ChannelState = serde_wasm_bindgen::from_value(new_state)
            .map_err(|e| JsValue::from_str(&format!("Invalid channel state: {:?}", e)))?;

        let state_manager = self.state_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Failed to acquire state manager lock: {:?}", e)))?;

        // Get state transition proof
        let proof = state_manager.create_state_transition_proof(
            &wallet.root_hash,
            &wallet_extension_contract::ByteArray32Local(channel_id),
            &channel.get_state_hash(),
            &channel_state.get_hash(),
        ).map_err(|e| JsValue::from_str(&format!("Failed to create state transition proof: {:?}", e)))?;

        // Create and return transaction
        #[derive(Serialize)]
        struct TransactionWrapper {
            transaction: TransactionOCData<WalletExtension>
        }

        let transaction = TransactionOCData::new(
            channel_id,
            TransactionType::StateUpdate,
            self.wallet_extension.clone(),
            channel.clone(),
            ZkProof::from(proof),
        );

        let wrapper = TransactionWrapper { transaction };

        serde_wasm_bindgen::to_value(&wrapper)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize transaction: {:?}", e)))
    }

    pub async fn verify_proof(
        &self,
        proof_data: &[u8],
        public_inputs: &[u64],
    ) -> Result<bool, JsValue> {
        let proof = ZkProof::new(proof_data.to_vec(), public_inputs.to_vec())
            .map_err(|e| JsValue::from_str(&format!("Invalid proof data: {:?}", e)))?;

        let state_manager = self.state_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Failed to acquire state manager lock: {:?}", e)))?;

        state_manager.verify_proof(&proof)
            .map_err(|e| JsValue::from_str(&format!("Proof verification failed: {:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_create_channel_transaction() {
        // Test implementation...
    }

    #[tokio::test]
    async fn test_update_channel_state() {
        // Test implementation...
    }

    #[tokio::test]
    async fn test_verify_proof() {
        // Test implementation...
    }
}