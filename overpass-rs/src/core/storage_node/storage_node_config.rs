// src/core/storage_node/storage_node_config.rs

use crate::core::error::errors::SystemError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNodeConfig {
    pub battery_config: BatteryConfig,
    pub sync_config: SyncConfig,
    pub epidemic_protocol_config: EpidemicProtocolConfig,
    pub network_config: NetworkConfig,
    pub node_id: [u8; 32],
    pub fee: u64,
    pub whitelist: HashSet<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryConfig {
    pub capacity: u64,
    pub charge_threshold: u64,
    pub discharge_threshold: u64,
    pub max_charge_rate: u64,
    pub max_discharge_rate: u64,
    pub min_charge_rate: u64,
    pub min_discharge_rate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub max_attempts: u32,
    pub retry_timeout: u64,
    pub min_timeout: u64,
    pub max_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpidemicProtocolConfig {
    pub max_propagations: u32,
    pub propagation_timeout: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_peers: u32,
    pub max_nodes: u32,
    pub max_channels: u32,
    pub max_storage_nodes: u32,
    pub max_storage_nodes_per_channel: u32,
    pub min_stake: u64,
    pub min_storage_node_stake: u64,
    pub challenge_interval: u64,
    pub challenge_threshold: u64,
}

impl Default for StorageNodeConfig {
    fn default() -> Self {
        Self {
            battery_config: BatteryConfig::default(),
            sync_config: SyncConfig::default(),
            epidemic_protocol_config: EpidemicProtocolConfig::default(),
            network_config: NetworkConfig::default(),
            node_id: [0u8; 32],
            fee: 0,
            whitelist: HashSet::new(),
        }
    }
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            charge_threshold: 10,
            discharge_threshold: 10,
            max_charge_rate: 10,
            max_discharge_rate: 10,
            min_charge_rate: 10,
            min_discharge_rate: 10,
        }
    }
}
impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            retry_timeout: 1000,
            min_timeout: 1000,
            max_timeout: 1000,
        }
    }
}

impl Default for EpidemicProtocolConfig {
    fn default() -> Self {
        Self {
            max_propagations: 10,
            propagation_timeout: 30000,
            max_retries: 3,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_peers: 10,
            max_nodes: 100,
            max_channels: 10,
            max_storage_nodes: 100,
            max_storage_nodes_per_channel: 10,
            min_stake: 1000,
            min_storage_node_stake: 1000,
            challenge_interval: 60000,
            challenge_threshold: 1000,
        }
    }
}
impl StorageNodeConfig {
    pub fn new(node_id: [u8; 32], fee: u64) -> Result<Self, SystemError> {
        let config = Self {
            battery_config: BatteryConfig::default(),
            sync_config: SyncConfig::default(),
            epidemic_protocol_config: EpidemicProtocolConfig::default(),
            network_config: NetworkConfig::default(),
            node_id,
            fee,
            whitelist: HashSet::new(),
        };

        Ok(config)
    }
}
