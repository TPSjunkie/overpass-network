// src/core/storage_node/storage_node_contract.rs

/// Storage node Smart Contract. This module contains the main storage node contract
/// that is responsible for managing the storage node's state and performing various
/// operations such as storing and retrieving data, as well as verifying the proofs
/// provided by the storage nodes. The contract is designed to be used in a WASM
/// environment, and provides methods for interacting with the storage node's
/// components, such as the battery, plonky2, consistency, distribution, and Overpass OP code ov_ops, is used to make calls to the contract with the op codes. like these codes:
/// 

/// Wallet Sparse Merkle Tree Management(WASM)
/// - This Group of OP codes is for the wallets storage network side,
///  which is responsible for managing the wallet's Sparse Merkle Tree. 
/// Transactions finalized instantly though so if you catch the byzantine 
/// actor at least as long as it's before, the channel is closed, but it's 
/// always checked on the closure of channel by the smart contract anyways, 
/// if there has been any suspicious activity, so if they get slashed, you 
/// can only spend 50% of what's in your balance per transaction so allows 
/// for a pretty good safeguard that's the most lesson you can continue to 
/// spend 50% over and over but you can't in one transaction send out more 
/// than 50% of your balance that's just the rule and then it's fair for everyone 
/// anyways so that's how it works so these codes are to deal with initiate initiate
///  initiating the wallet contracts you have to make a call to the intermediate 
/// notes and they will generate the wallets for the user and those wallet contracts can generate the channels.
/// All levels of state, intermediate and route included have a redundant group of nodes assigned to them based on intermediate contract level anyways, let's get down to the business here and implement this module

use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::battery::BatteryChargingSystem;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use crate::core::zkps::zkp::VirtualCell;
use crate::core::zkps::plonky2::{Plonky2System, Plonky2SystemHandle}; 
use crate::core::storage_node::replication::consistency::ConsistencyManager;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::storage_node::replication::verification::ResponseManager;
use crate::core::types::ovp_ops::*;
use futures::lock::Mutex;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::core::zkps::{circuit_builder::ZkCircuitBuilder, zkp_interface::ProofGenerator};

/// Configuration for the storage node
#[derive(Debug, Clone, Default)]
pub struct StorageNodeConfig {
    pub battery_config: BatteryConfig,
    pub sync_config: SyncConfig,
    pub epidemic_protocol_config: EpidemicProtocolConfig,
    pub network_config: NetworkConfig,
    pub node_id: [u8; 32],
    pub fee: i64,
    pub whitelist: HashSet<[u8; 32]>,
}

impl StorageNodeConfig {
    pub fn new(
        battery_config: BatteryConfig,
        sync_config: SyncConfig,
        epidemic_protocol_config: EpidemicProtocolConfig,
        network_config: NetworkConfig,
        node_id: [u8; 32],
        fee: i64,
        whitelist: HashSet<[u8; 32]>,
    ) -> Self {
        Self {
            battery_config,
            sync_config,
            epidemic_protocol_config,
            network_config,
            node_id,
            fee,
            whitelist,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BatteryConfig {
    pub max_charging_threshold: u64,
    pub max_charging_interval: u64,
}

#[derive(Debug, Clone, Default)]
pub struct SyncConfig {
    pub max_synchronization_boost: u64,
    pub min_synchronization_boost: u64,
    pub synchronization_boost_interval: u64,
}

#[derive(Debug, Clone, Default)]
pub struct EpidemicProtocolConfig {
    pub max_propagation_threshold: u64,
    pub max_propagation_interval: u64,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkConfig {
    pub max_peer_count: u64,
    pub min_peer_count: u64,
    pub max_peer_count_increase: u64,
    pub min_peer_count_decrease: u64,
    pub peer_count_increase_interval: u64,
    pub peer_count_decrease_interval: u64,
}

/// Storage node contract
pub struct StorageNode {
    pub node_id: [u8; 32],
    pub fee: i64,
    pub whitelist: HashSet<[u8; 32]>,

    pub battery_system: Arc<Mutex<BatteryChargingSystem>>,
    pub plonky2_system: Arc<Mutex<Plonky2System>>,
    pub consistency_manager: Arc<Mutex<ConsistencyManager>>,
    pub distribution_manager: Arc<Mutex<DistributionManager>>,
    pub response_manager: Arc<Mutex<ResponseManager>>,

    pub stored_bocs: Arc<Mutex<HashMap<[u8; 32], BOC>>>,
    pub stored_proofs: Arc<Mutex<HashMap<[u8; 32], ZkProof>>>,

    pub virtual_cells: Arc<Mutex<HashMap<VirtualCell, MerkleNode>>>,
    pub current_virtual_cell: Arc<Mutex<VirtualCell>>,
    pub current_virtual_cell_count: Arc<Mutex<usize>>,
}

impl StorageNode {
    pub fn new(
        node_id: [u8; 32],
        fee: i64,
        config: StorageNodeConfig,
        stored_bocs: HashMap<[u8; 32], BOC>,
    ) -> Result<Self, SystemError> {
        let stored_bocs = Arc::new(Mutex::new(stored_bocs));
        let stored_proofs = Arc::new(Mutex::new(HashMap::new()));

        let plonky2_system = Arc::new(Mutex::new(Plonky2System::new(config.network_config)?));

        let battery_system = Arc::new(Mutex::new(BatteryChargingSystem::new(config.battery_config)));

        let consistency_manager = Arc::new(Mutex::new(ConsistencyManager::new(
            Arc::clone(&plonky2_system),
        )));

        let distribution_manager = Arc::new(Mutex::new(DistributionManager::new(
            plonky2_system.lock().unwrap().into(),
            config.epidemic_protocol_config,
        )));

        let response_manager = Arc::new(Mutex::new(ResponseManager::create(
            node_id,
            config.sync_config,
            Arc::clone(&plonky2_system),
        )?));

        let virtual_cells = Arc::new(Mutex::new(HashMap::new()));
        let current_virtual_cell = Arc::new(Mutex::new(VirtualCell::new(Column::default(), 0)));
        let current_virtual_cell_count = Arc::new(Mutex::new(0));

        Ok(Self {
            node_id,
            fee,
            whitelist: HashSet::new(),
            battery_system,
            plonky2_system,
            consistency_manager,
            distribution_manager,
            response_manager,
            stored_bocs,
            stored_proofs,
            virtual_cells,
            current_virtual_cell,
            current_virtual_cell_count,
        })        
    }}

pub struct Storage {
    proof_generator: ProofGenerator,
}

impl Storage {
    pub fn new() -> Self {
        let proof_generator = ProofGenerator::try_new().expect("Failed to initialize proof generator");
        Self { proof_generator }
    }

    pub fn generate_and_store_proof(
        &self,
        old_balance: u64,
        new_balance: u64,
        amount: u64,
    ) -> Result<(), String> {
        let proof = self
            .proof_generator
            .generate_state_transition_proof(old_balance, new_balance, amount, None)
            .map_err(|e| format!("Proof generation failed: {}", e))?;

        // Store proof in storage layer (e.g., database, cache)
        self.store_proof_in_db(proof)?;
        Ok(())
    }

    fn store_proof_in_db(&self, proof: JsValue) -> Result<(), String> {
        // Placeholder for DB interaction logic
        Ok(())
    }
}
