use crate::common::types::ops::OpCode;
use crate::core::client::wallet_extension::channel_manager::ChannelConfig;
use crate::core::client::wallet_extension::user::ChannelState;
use crate::core::client::{
    channel::channel_contract::ChannelContract,
    wallet_extension::wallet_extension_types::WalletExtension,
};
use crate::core::state::sparse_merkle_tree_wasm::SparseMerkleTreeWasm;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
interface ByteArray32 {
    toArray(): Uint8Array;
}
"#;

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ByteArray32Local(#[wasm_bindgen(skip)] pub [u8; 32]);

#[wasm_bindgen]
impl ByteArray32Local {
    #[wasm_bindgen(constructor)]
    pub fn new_from_slice(array: &[u8]) -> Result<ByteArray32Local, JsValue> {
        if array.len() != 32 {
            return Err(JsValue::from_str("Array must be 32 bytes long"));
        }
        let mut result = [0u8; 32];
        result.copy_from_slice(array);
        Ok(ByteArray32Local(result))
    }

    pub fn to_string(&self) -> String {
        hex::encode(self.0.to_vec())
    }

    pub fn from_string(val: &str) -> Result<ByteArray32Local, JsValue> {
        let array = hex::decode(val).map_err(|_| JsValue::from_str("Invalid hex string"))?;
        Self::new_from_slice(&array)
    }
}

impl WalletExtension {
    pub fn new() -> Self {
        Self {
            wallet_id: [0u8; 32],
            channels: HashMap::new(),
            total_locked_balance: 0,
            rebalance_config: Default::default(),
            proof_system: Arc::new(Default::default()),
            state_tree: Arc::new(RwLock::new(SparseMerkleTreeWasm::new())),
            root_hash: [0u8; 32],
            balance: 0,
            encrypted_states: HashMap::new(),
        }
    }

    pub async fn create_channel(
        &mut self,
        sender: [u8; 32],
        _recipient: [u8; 32],
        deposit: u64,
        _config: &ChannelConfig,
    ) -> Result<[u8; 32], JsValue> {
        let channel_id = rand::random::<[u8; 32]>();

        // Create channel contract
        let channel = ChannelContract::new(&hex::encode(sender));

        self.state_tree
            .write()
            .unwrap()
            .update(&channel_id.to_vec(), &channel_id.to_vec());
        self.channels
            .insert(channel_id, Arc::new(RwLock::new(channel)));
        self.total_locked_balance += deposit;

        Ok(channel_id)
    }

    pub async fn update_channel_state(
        &mut self,
        channel_id: [u8; 32],
        new_state: ChannelState,
    ) -> Result<Vec<u8>, JsValue> {
        let channel = self
            .channels
            .get_mut(&channel_id)
            .ok_or_else(|| JsValue::from_str("Channel not found"))?;

        let mut channel = channel.write().unwrap();
        channel.update_balance(new_state.balance);

        Ok(Vec::new())
    }
}
