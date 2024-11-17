// ./src/core/hierarchy/client/wallet_extension/user.rs
use std::collections::HashSet;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct User {
    name: String,
    channels: HashSet<[u8; 32]>,
}

#[wasm_bindgen]
impl User {
    pub fn new(name: String, channels: HashSet<[u8; 32]>) -> Self {
        Self { name, channels }
    }

    #[wasm_bindgen(getter)]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn get_channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn get_channel_ids(&self) -> js_sys::Array {
        self.channels
            .iter()
            .map(|channel_id| JsValue::from(channel_id.to_vec()))
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
        let mut balance = 0;
        if let Ok(channel_array) = channel_id.try_into() {
            if self.channels.contains(&channel_array) {
                // Here we would typically query a blockchain or database
                // For now, generate a pseudo-random balance based on channel_id
                balance = channel_id.iter().fold(0u64, |acc, &x| acc.wrapping_add(x as u64)) * 1000;
            }
        }
        balance
    }

    pub fn get_channel_transaction_count(&self, channel_id: &[u8]) -> u64 {
        let mut count = 0;
        if let Ok(channel_array) = channel_id.try_into() {
            if self.channels.contains(&channel_array) {
                // Here we would typically query a blockchain or database
                // For now, generate a pseudo-random transaction count based on channel_id
                count = channel_id.iter().fold(0u64, |acc, &x| acc.wrapping_add(x as u64)) % 100;
            }
        }
        count
    }
}