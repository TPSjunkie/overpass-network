// ./src/core/hierarchy/intermediate/intermediate_contract.rs

// This module provides an implementation of the IntermediateContract struct, which is used to manage the state of the Overpass Network.
// It includes methods to handle state updates, channel requests, channel closures, and rebalances.

use crate::core::hierarchy::client::wallet_extension::wallet_extension_types::ChannelClosureRequest;
use crate::core::hierarchy::intermediate::destination_contract::DestinationContract;
use crate::core::hierarchy::intermediate::intermediate_contract_types::IntermediateContract;
use crate::core::hierarchy::intermediate::sparse_merkle_tree_i::SparseMerkleTreeI;
use crate::core::storage_node::storage_node_contract::StorageNode;
use crate::core::zkps::plonky2::Plonky2System;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::time::{Duration, SystemTime};

impl<RebalanceRequest> std::fmt::Debug for IntermediateContract<RebalanceRequest>
where
    RebalanceRequest: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntermediateContract")
            .field("auto_rebalance", &self.auto_rebalance)
            .field("battery_charge_rate", &self.battery_charge_rate)
            .field("battery_discharge_rate", &self.battery_discharge_rate)
            .field("battery_level", &self.battery_level)
            .field("battery_wait_time", &self.battery_wait_time)
            .field("challenge_interval", &self.challenge_interval)
            .field("challenge_threshold", &self.challenge_threshold)
            .field("closing_channels", &self.closing_channels)
            .field("destination_contract", &self.destination_contract)
            .field("intermediate_tree", &"SparseMerkleTreeI")
            .field("last_sync", &self.last_sync)
            .field("max_channel_density", &self.max_channel_density)
            .field("max_storage_nodes", &self.max_storage_nodes)
            .field(
                "max_storage_node_batch_size",
                &self.max_storage_node_batch_size,
            )
            .field("max_updates_per_batch", &self.max_updates_per_batch)
            .field("min_storage_nodes", &self.min_storage_nodes)
            .field("state_update_interval", &self.state_update_interval)
            .field("storage_nodes", &"StorageNode")
            .field("rebalance_queue", &self.rebalance_queue)
            .field("tree_manager", &"TreeManager")
            .field("zk_verifier", &"Plonky2System")
            .finish()
    }
}
