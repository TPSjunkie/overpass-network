use crate::core::hierarchy::intermediate::sparse_merkle_tree_i::MerkleNode;
use crate::core::error::errors::SystemError;
use crate::core::storage_node::battery::BatteryChargingSystem;
use crate::core::types::boc::BOC;
use crate::core::zkps::circuit_builder::Column;
use crate::core::zkps::proof::ZkProof;
use crate::core::zkps::zkp::VirtualCell;
use crate::core::zkps::plonky2::Plonky2System;
use crate::core::storage_node::replication::consistency::ConsistencyValidator;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::storage_node::attest::response::ResponseManager;
use crate::core::zkps::proof::ProofGenerator;
use futures::lock::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

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

pub struct StorageNode {
    pub node_id: [u8; 32],
    pub fee: i64,
    pub whitelist: HashSet<[u8; 32]>,
    pub battery_system: Arc<Mutex<BatteryChargingSystem>>,
    pub plonky2_system: Arc<Mutex<Plonky2System>>,
    pub consistency_validator: Arc<Mutex<ConsistencyValidator>>,
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
    ) -> Result<Self, SystemError> {
        let stored_bocs = Arc::new(Mutex::new(HashMap::new()));
        let stored_proofs = Arc::new(Mutex::new(HashMap::new()));

        let plonky2_system = Arc::new(Mutex::new(Plonky2System::new(config.network_config)?));

        let battery_system = Arc::new(Mutex::new(BatteryChargingSystem::new(config.battery_config)));

        let consistency_validator = Arc::new(Mutex::new(ConsistencyValidator::new(
            Arc::clone(&plonky2_system),
        )));

        let distribution_manager = Arc::new(Mutex::new(DistributionManager::new(
            config.epidemic_protocol_config.max_propagation_threshold,
            config.epidemic_protocol_config.max_propagation_interval,
        )));

        let response_manager = Arc::new(Mutex::new(ResponseManager::create(
            node_id,
            config.sync_config.max_synchronization_boost,
            config.sync_config.min_synchronization_boost,
            config.sync_config.synchronization_boost_interval,
        )?));

        let virtual_cells = Arc::new(Mutex::new(HashMap::new()));
        let current_virtual_cell = Arc::new(Mutex::new(VirtualCell::new(0, 0)));
        let current_virtual_cell_count = Arc::new(Mutex::new(0));

        Ok(Self {
            node_id,
            fee,
            whitelist: config.whitelist,
            battery_system,
            plonky2_system,
            consistency_validator,
            distribution_manager,
            response_manager,
            stored_bocs,
            stored_proofs,
            virtual_cells,
            current_virtual_cell,
            current_virtual_cell_count,
        })
    }
}
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

        self.store_proof(proof)
    }

    pub fn store_proof(&self, proof: ZkProof) -> Result<(), String> {
        let mut proofs = self.get_stored_proofs();
        proofs.insert(proof.hash(), proof);
        Ok(())
    }

    pub fn get_stored_proofs(&self) -> HashMap<[u8; 32], ZkProof> {
        HashMap::new()
    }
}
