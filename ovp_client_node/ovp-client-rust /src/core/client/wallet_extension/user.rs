use crate::common::types::dag_boc::DAGBOC;
use crate::common::types::state_boc::STATEBOC;
use crate::core::client::wallet_extension::client_proof_exporter::WalletRootProof;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::wasm_bindgen;

#[derive(Debug, Clone)]
pub enum UserData {
    User(User),
    UserBoc(STATEBOC),
    UserProof(WalletRootProof),
}

#[wasm_bindgen(js_name = "User")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    name: String,
    channels: Vec<Vec<u8>>,
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
use wasm_bindgen::JsValue;

impl User {
    pub fn new_with_channels(name: String, channels: Vec<Vec<u8>>) -> User {
        User { name, channels }
    }

    pub fn add_channel(&mut self, channel: js_sys::Uint8Array) {
        if channel.length() == 32 {
            let mut array = vec![0u8; 32];
            channel.copy_to(&mut array);
            self.channels.push(array);
        }
    }

    pub fn new_empty(name: String) -> User {
        User {
            name,
            channels: Vec::new(),
        }
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(js_name = "getName"))]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get_channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn get_channel_ids(&self) -> js_sys::Array {
        self.channels
            .iter()
            .map(|channel_id| JsValue::from(channel_id.clone()))
            .collect::<js_sys::Array>()
    }

    pub fn get_channel_names(&self) -> js_sys::Array {
        self.channels
            .iter()
            .map(|channel_id| JsValue::from(self.get_channel_name(channel_id)))
            .collect::<js_sys::Array>()
    }

    pub fn get_channel_name(&self, channel_id: &[u8]) -> String {
        format!("Channel-{}", hex::encode(channel_id))
    }

    pub fn get_channel_balance(&self, channel_id: &[u8]) -> u64 {
        if self.channels.iter().any(|c| c == channel_id) {
            let channel_state = self.query_blockchain_state(channel_id);
            channel_state.balance
        } else {
            0
        }
    }

    pub fn get_channel_transaction_count(&self, channel_id: &[u8]) -> u64 {
        if self.channels.iter().any(|c| c == channel_id) {
            let channel_state = self.query_blockchain_state(channel_id);
            channel_state.transaction_count
        } else {
            0
        }
    }

    pub fn get_channel_state(&self, channel_id: &[u8]) -> JsValue {
        let channel_state = self.query_blockchain_state(channel_id);
        serde_wasm_bindgen::to_value(&channel_state).unwrap_or(JsValue::NULL)
    }

    pub fn get_channel_state_string(&self, channel_id: &[u8]) -> String {
        let channel_state = self.query_blockchain_state(channel_id);
        format!(
            "Balance: {}, Transactions: {}",
            channel_state.balance, channel_state.transaction_count
        )
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

impl User {
    pub fn new(name: String) -> User {
        User {
            name,
            channels: Vec::new(),
        }
    }

    pub fn query_blockchain_state(&self, channel_id: &[u8]) -> ChannelState {
        // Generate deterministic pseudo-random state based on channel_id
        let mut channel_state = ChannelState::default();

        if self.channels.iter().any(|c| c == channel_id) {
            channel_state.balance = channel_id
                .iter()
                .fold(0u64, |acc, &x| acc.wrapping_add(x as u64))
                * 1000;
            channel_state.transaction_count = channel_id
                .iter()
                .fold(0u64, |acc, &x| acc.wrapping_add(x as u64))
                % 100;
        }

        channel_state
    }
}

impl Default for User {
    fn default() -> User {
        User {
            name: String::new(),
            channels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelState {
    pub balance: u64,
    pub transaction_count: u64,
}
