// ./src/core/hierarchy/client/wallet_extension/user.rs
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

// This is a struct that represents a user (private)
#[wasm_bindgen]
pub struct User {
    name: String,
    channels: HashSet<[u8; 32]>,
}

impl User {
    pub fn new(name: String, channels: HashSet<[u8; 32]>) -> Self {
        Self { name, channels }
    }
}

#[wasm_bindgen]
impl User {
    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get_channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn get_channel_ids(&self) -> js_sys::Array {
        self.channels.iter().map(|id| js_sys::Uint8Array::from(&id[..]).into()).collect()
    }

    pub fn get_channel_names(&self) -> js_sys::Array {
        self.channels
            .iter()
            .map(|channel_id| self.get_channel_name(channel_id).into())
            .collect()
    }

    pub fn get_channel_name(&self, channel_id: &[u8; 32]) -> String {
        format!("{:?}", channel_id)
    }

    pub fn get_channel_balance(&self, channel_id: &[u8]) -> u64 {
        0 // Placeholder implementation
    }

    pub fn get_channel_transaction_count(&self, channel_id: &[u8]) -> u64 {
        0 // Placeholder implementation
    }
}
// This is a function that returns the name of the user
