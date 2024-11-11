// ./src/core/hierarchy/client/wallet_extension/user.rs
use crate::core::types::boc::BOC;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{Blob, Url};

// This is a struct that represents a user (private)
#[wasm_bindgen]
pub struct User {
    pub name: String,
    pub channels: HashSet<[u8; 32]>,
}

impl User {
    pub fn new(name: String, channels: HashSet<[u8; 32]>) -> Self {
        Self { name, channels }
    }
}

#[wasm_bindgen]
impl User {
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_channels(&self) -> HashSet<[u8; 32]> {
        self.channels.clone()
    }

    pub fn get_channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn get_channel_ids(&self) -> Vec<[u8; 32]> {
        self.channels.iter().cloned().collect()
    }

    pub fn get_channel_names(&self) -> Vec<String> {
        self.channels
            .iter()
            .map(|channel_id| self.get_channel_name(channel_id))
            .collect()
    }

    pub fn get_channel_name(&self, channel_id: &[u8; 32]) -> String {
        let channel = self.channels.get(channel_id).unwrap();
        format!("{:?}", channel)
    }

    pub fn get_channel_balance(&self, channel_id: &[u8; 32]) -> u64 {
        let channel = self.channels.get(channel_id).unwrap();
        format!("{:?}", channel)
    }

    pub fn get_channel_transaction_count(&self, channel_id: &[u8; 32]) -> u64 {
        let channel = self.channels.get(channel_id).unwrap();
        format!("{:?}", channel)
    }
}
// This is a function that returns the name of the user
