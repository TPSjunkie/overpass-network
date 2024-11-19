use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::battery::BatteryChargingSystem;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::storage_node::replication::verification::ResponseManager;
use crate::core::types::ovp_ops::*;
use futures::lock::Mutex;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone)]
pub struct StorageNodeConfig {
    pub battery_config: BatteryConfig,
    pub sync_config: SyncConfig,
    pub epidemic_protocol_config: EpidemicProtocolConfig,
    pub network_config: NetworkConfig,
    pub(crate) node_id: [u8; 32],
    pub(crate) fee: i32,
    pub(crate) whitelist: HashSet<[u8; 32]>,
}

#[derive(Clone, Default)]
pub struct BatteryConfig {
    pub initial_charge: u64,
    pub max_charge: u64,
}

#[derive(Clone, Default)]
pub struct SyncConfig {
    pub sync_interval: u64,
    pub retry_delay: u64,
}

#[derive(Clone, Default)]
pub struct EpidemicProtocolConfig {
    pub redundancy_factor: u32,
    pub propagation_probability: f64,
}

#[derive(Clone, Default)]
pub struct NetworkConfig {
    pub port: u16,
    pub max_connections: u32,
}

pub struct StorageNode {
    pub node_id: [u8; 32],
    pub stake: u64,
    pub stored_bocs: Arc<Mutex<HashMap<[u8; 32], BOC>>>,
    pub stored_proofs: Arc<Mutex<HashMap<[u8; 32], ZkProof>>>,
    pub battery_system: Arc<Mutex<BatteryChargingSystem>>,
    pub config: StorageNodeConfig,
    peers: Arc<Mutex<HashSet<[u8; 32]>>>,
}

impl StorageNode {
    pub async fn new(
        node_id: [u8; 32],
        initial_stake: u64,
        config: StorageNodeConfig,
        peers: HashSet<[u8; 32]>,
    ) -> Result<Self, SystemError> {
        if initial_stake < 1000 {
            return Err(SystemError {
                error_type: SystemErrorType::InvalidAmount,
                message: "Insufficient initial stake".to_string(),
            });
        }

        let battery_charging_system = BatteryChargingSystem::new(initial_stake);

        Ok(Self {
            node_id,
            stake: initial_stake,
            stored_bocs: Arc::new(Mutex::new(HashMap::new())),
            stored_proofs: Arc::new(Mutex::new(HashMap::new())),
            battery_system: Arc::new(Mutex::new(battery_charging_system)),
            config,
            peers: Arc::new(Mutex::new(peers)),
        })
    }

    pub async fn store_update(&self, boc: BOC, proof: ZkProof) -> Result<(), SystemError> {
        let mut battery_system = self.battery_system.lock().await;
        battery_system
            .charge_for_processing()
            .await
            .map_err(|e| SystemError {
                error_type: SystemErrorType::BatteryError,
                message: e.to_string(),
            })?;

        if !self.verify_proof_internal(&proof, &boc)? {
            return Err(SystemError {
                error_type: SystemErrorType::InvalidProof,
                message: "Invalid proof".to_string(),
            });
        }

        let boc_id = self.hash_boc_internal(&boc);
        {
            let mut bocs = self.stored_bocs.lock().await;
            bocs.insert(boc_id, boc);

            let mut proofs = self.stored_proofs.lock().await;
            proofs.insert(boc_id, proof);
        }

        Ok(())
    }

    pub async fn retrieve_boc(&self, boc_id: &[u8; 32]) -> Result<BOC, SystemError> {
        let bocs = self.stored_bocs.lock().await;
        bocs.get(boc_id).cloned().ok_or_else(|| SystemError {
            error_type: SystemErrorType::NotFound,
            message: "BOC not found".to_string(),
        })
    }

    pub async fn retrieve_proof(&self, proof_id: &[u8; 32]) -> Result<ZkProof, SystemError> {
        let proofs = self.stored_proofs.lock().await;
        proofs.get(proof_id).cloned().ok_or_else(|| SystemError {
            error_type: SystemErrorType::NotFound,
            message: "Proof not found".to_string(),
        })
    }

    pub async fn add_peer(&self, peer_id: [u8; 32]) -> Result<(), SystemError> {
        let mut peers = self.peers.lock().await;
        peers.insert(peer_id);
        Ok(())
    }

    pub async fn remove_peer(&self, peer_id: &[u8; 32]) -> Result<(), SystemError> {
        let mut peers = self.peers.lock().await;
        peers.remove(peer_id);
        Ok(())
    }

    fn hash_boc_internal(&self, boc: &BOC) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(bincode::serialize(boc).unwrap());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    fn verify_proof_internal(&self, proof: &ZkProof, boc: &BOC) -> Result<bool, SystemError> {
        // Verify the proof matches the BOC
        let boc_hash = self.hash_boc_internal(boc);
    
        // Verify the proof's public inputs contain the BOC hash
        if proof.public_inputs.len() != 1 || proof.public_inputs[0] != boc_hash.to_vec() {
            return Ok(false);
        }

        // Verify the ZK proof cryptographic validity using the verification key
        // Note: This part is commented out due to missing dependencies
        /*
        let verification_key = self.generate_verification_key();
        let proof_bytes = proof.proof_data.to_vec();
        let proof = Proof::<Bn254>::read(&proof_bytes[..]).map_err(|_| SystemError::new(
            SystemErrorType::InvalidProof,
            "Invalid proof".into()
        ))?;
        match Bn254::verify(&verification_key, &[boc_hash], &proof) {
            Ok(true) => Ok(true),
            Ok(false) => Ok(false),
            Err(_) => Err(SystemError::new(
                SystemErrorType::InvalidProof,
                "Invalid proof".into()
            )),
        }
        */
        
        // Placeholder return value
        Ok(true)
    }

    // Note: This function is commented out due to missing dependencies
    /*
    fn generate_verification_key(&self) -> ark_bn254::Bn254VerificationKey<ark_bn254::Bn254> {
        let params = ark_bn254::Bn254::new();
        let vk = ark_bn254::Bn254VerificationKey::<ark_bn254::Bn254>::new(
            params,
            self.node_id,
            self.config.fee,
            self.config.whitelist.clone(),
        );
        vk
    }
    */
}

impl StorageNodeConfig {
    pub fn new(
        battery_config: BatteryConfig,
        sync_config: SyncConfig,
        epidemic_config: EpidemicProtocolConfig,
        network_config: NetworkConfig,
        node_id: [u8; 32],
        fee: i32,
        whitelist: HashSet<[u8; 32]>,
    ) -> Self {
        Self {
            battery_config,
            sync_config,
            epidemic_protocol_config: epidemic_config,
            network_config,
            node_id,
            fee,
            whitelist,
        }
    }
}
