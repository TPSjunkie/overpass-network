// ./src/core/hierarchy/intermediate/intermediate_contract_types.rs

use crate::core::hierarchy::client::wallet_extension::wallet_extension_types::ChannelClosureRequest;

use crate::core::hierarchy::intermediate::destination_contract::DestinationContract;
use crate::core::hierarchy::intermediate::sparse_merkle_tree_i::SparseMerkleTreeI;
use crate::core::zkps::plonky2::Plonky2System;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::time::{Duration, SystemTime};

type ChannelId = String;
type ChannelIdMap = HashMap<ChannelId, ChannelClosureRequest>;
type ChannelIdMapVec = VecDeque<ChannelId>;

pub trait ByteArray<const N: usize> {
   fn new() -> Self;
   fn from_bytes(bytes: [u8; N]) -> Self;
   fn to_bytes(&self) -> [u8; N];
   fn from_str(s: &str) -> Self;
   fn to_string(&self) -> String;
   fn from_hex(s: &str) -> Self;
   fn to_hex(&self) -> String;
   fn from_base64(s: &str) -> Self;
   fn to_base64(&self) -> String;
   fn zero() -> Self;
   fn one() -> Self;
}

// Remove the recursive type alias
// type IntermediateContract = IntermediateContract<RebalanceRequest>;

// Define RebalanceRequest as a struct instead of a recursive type alias
pub struct RebalanceRequest {
    // Add fields as needed
}

type RebalanceRequestMap = HashMap<ChannelId, RebalanceRequest>;

type IntermediateContractMap = HashMap<ChannelId, IntermediateContract>;
type IntermediateContractMapVec = VecDeque<IntermediateContract>;

pub struct IntermediateContract {
    pub auto_rebalance: bool,
    pub battery_charge_rate: f64,
    pub battery_discharge_rate: f64,
    pub battery_level: f64,
    pub battery_wait_time: Duration,
    pub challenge_interval: Duration,
    pub challenge_threshold: u64,
    pub closing_channels: ChannelIdMap,
    pub destination_contract: DestinationContract,
    pub intermediate_tree: SparseMerkleTreeI,
    pub last_sync: SystemTime,
    pub max_channel_density: u32,
    pub max_storage_nodes: u32,
    pub min_channel_density: u32,
    pub min_storage_nodes: u32,
    pub plonky2_system: Plonky2System,
    pub rebalance_requests: VecDeque<RebalanceRequest>,
    pub rebalance_request_interval: Duration,
    pub rebalance_request_threshold: u64,
    pub rebalance_request_window: Duration,
    pub rebalance_request_window_start: SystemTime,
    pub state_update_interval: Duration,
    pub state_update_threshold: u64,
    pub state_update_window: Duration,
    pub state_update_window_start: SystemTime,
    _phantom: PhantomData<(SparseMerkleTreeI, Plonky2System)>,
}