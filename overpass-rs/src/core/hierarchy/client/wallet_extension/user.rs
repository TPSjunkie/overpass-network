use crate::core::hierarchy::client::wallet_extension::client_proof_exporter::WalletRootProof;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

use crate::core::types::boc::BOC;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum UserEnumVariant {
    UserData(UserData),
    UserBoc(BOC),
    UserProof(WalletRootProof),
}

#[derive(Clone, Debug)]
pub struct UserData {
    pub name: String,
    pub channels: HashSet<[u8; 32]>,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            name: String::new(),
            channels: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    pub balance: u64,
    pub transaction_count: u64,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            balance: 0,
            transaction_count: 0,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct User {
    name: String,
    channels: Vec<[u8; 32]>,
}



#[wasm_bindgen]
impl UserData { 
    #[wasm_bindgen(constructor)]
    pub fn new(name: String) -> Self {
        Self { 
            name,
            channels: HashSet::new() 
        }
    }
}
#[wasm_bindgen]
impl User { 
    #[wasm_bindgen(constructor)]
    pub fn new(name: String, channels: Vec<[u8; 32]>) -> Self {
        Self { 
            name,
            channels 
        }
    }
}
#[wasm_bindgen]
impl UserBoc {  
    #[wasm_bindgen(constructor)]
    pub fn new(name: String) -> Self {
        Self { 
            name,
            channels: Vec::new() 
        }
    }
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[wasm_bindgen]
    pub fn get_channel_count(&self) -> usize {
        self.channels.len()
    }

    #[wasm_bindgen]
    pub fn get_channel_ids(&self) -> js_sys::Array {
        self.channels
            .iter()
            .map(|channel_id| JsValue::from(channel_id.to_vec()))
            .collect::<js_sys::Array>()
    }

    #[wasm_bindgen]
    pub fn get_channel_names(&self) -> js_sys::Array {
        self.channels
            .iter()
            .map(|channel_id| JsValue::from(self.get_channel_name(channel_id)))
            .collect::<js_sys::Array>()
    }

    #[wasm_bindgen]
    pub fn get_channel_name(&self, channel_id: &[u8]) -> String {
        format!("Channel-{}", hex::encode(channel_id))
    }

    #[wasm_bindgen]
    pub fn get_channel_balance(&self, channel_id: &[u8]) -> u64 {
        let channel_id_array: &[u8; 32] = match channel_id.try_into() {
            Ok(array) => array,
            Err(_) => return 0,
        };
        
        if self.channels.contains(channel_id_array) {
            let channel_state = self.query_blockchain_state(channel_id);
            channel_state.balance
        } else {
            0
        }
    }

    #[wasm_bindgen]
    pub fn get_channel_transaction_count(&self, channel_id: &[u8]) -> u64 {
        let channel_id_array: &[u8; 32] = match channel_id.try_into() {
            Ok(array) => array,
            Err(_) => return 0,
        };

        if self.channels.contains(channel_id_array) {
            let channel_state = self.query_blockchain_state(channel_id);
            channel_state.transaction_count
        } else {
            0
        }
    }

    #[wasm_bindgen]
    pub fn query_blockchain_state(&self, channel_id: &[u8]) -> ChannelState {
        // Generate deterministic pseudo-random state based on channel_id
        let mut channel_state = ChannelState::default();
        let channel_id_array: &[u8; 32] = channel_id.try_into().unwrap_or(&[0u8; 32]);
        
        if self.channels.contains(channel_id_array) {
            channel_state.balance = channel_id_array.iter()
                .fold(0u64, |acc, &x| acc.wrapping_add(x as u64)) * 1000;
            channel_state.transaction_count = channel_id_array.iter()
                .fold(0u64, |acc, &x| acc.wrapping_add(x as u64)) % 100;
        }
        
        channel_state
    }

    #[wasm_bindgen]
    pub fn get_channel_state(&self, channel_id: &[u8]) -> JsValue {
        let channel_state = self.query_blockchain_state(channel_id);
        serde_wasm_bindgen::to_value(&channel_state).unwrap_or(JsValue::NULL)
    }

    pub fn get_channel_state_string(&self, channel_id: &[u8]) -> String {
        let channel_state = self.query_blockchain_state(channel_id);
        format!("Balance: {}, Transactions: {}", channel_state.balance, channel_state.transaction_count)
    }

    pub fn get_channel_state_json(&self, channel_id: &[u8]) -> String {
        let channel_state = self.query_blockchain_state(channel_id);
        serde_json::to_string(&channel_state).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn get_channel_state_json_pretty(&self, channel_id: &[u8]) -> String {
        let channel_state = self.query_blockchain_state(channel_id);
        serde_json::to_string_pretty(&channel_state).unwrap_or_else(|_| "{}".to_string())
    }
}