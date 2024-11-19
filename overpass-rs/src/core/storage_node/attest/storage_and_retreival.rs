// ./src/core/storage_node/verification/storage_and_retrieval.rs

/// Storage and Retrieval Verification
/// This module is the entry point for accessing the storage nodes' storage and retrieval capabilities.
/// It provides methods for storing and retrieving data, as well as verifying the proofs provided by the storage nodes. 
/// The storage note contract module uses OP codes, which will tell it to access this module for storage and retrieval. 
/// The OP codes are defined in the `ov_ops` module.
/// The information is stored in the form of a BOC (Bag of Cells) and a proof. 
/// The BOC contains the data and the proof contains the information about the data, such as the public inputs, the merkle root, and the timestamp.     
/// The storage and retrieval verification module is responsible for verifying the proofs provided by the storage nodes.
/// The verification process involves checking the validity of the proof and the consistency of the data.

use crate::core::error::errors::{SystemError, SystemErrorType, CellError};
use crate::core::types::boc::BOC;
use crate::core::types::ovp_ops::*;
use crate::core::zkps::plonky2::Plonky2SystemHandle;
use crate::core::zkps::proof::ZkProof;
use crate::core::hierarchy::intermediate::sparse_merkle_tree_i::MerkleNode;
use crate::core::hierarchy::root::sparse_merkle_tree_r::RootTreeManagerTrait;
use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
use crate::core::zkps::zkp::VirtualCell;
use crate::core::storage_node::storage_node_contract::{StorageNode, StorageNodeConfig};
use crate::core::storage_node::replication::state::StateManager;
use crate::core::storage_node::replication::consistency::ConsistencyValidator;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::storage_node::replication::verification::VerificationManager;

/// Storage and Retrieval Metrics
/// This struct contains metrics related to storage and retrieval.  
/// It includes metrics for storing, retrieving, and verifying data.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct StorageAndRetrievalMetrics {
        pub store_boc: u64,
        pub store_proof: u64,
        pub retrieve_boc: u64,
        pub retrieve_proof: u64,
    }

    impl Default for StorageAndRetrievalMetrics {
        fn default() -> Self {
            Self {
                store_boc: 0,
                store_proof: 0,
                retrieve_boc: 0,
                retrieve_proof: 0,
            }
        }
    }
    /// Storage and Retrieval Manager
    ///
    /// This struct represents the storage and retrieval manager. It contains the logic for storing and retrieving data, as well as verifying the proofs provided by the storage nodes.
    pub struct StorageAndRetrievalManager {
        storage_node: Arc<StorageNode>,
        metrics: StorageAndRetrievalMetrics,
        store_boc: bool,
        store_proof: bool,
        retrieve_boc: bool,
        retrieve_proof: bool,
        verify_proof: bool,
    }
    impl StorageAndRetrievalManager {
        pub fn new(storage_node: Arc<StorageNode>) -> Self {
            Self {
                storage_node,
                metrics: StorageAndRetrievalMetrics {
                    store_boc: 0,
                    store_proof: 0,
                    retrieve_boc: 0,
                    retrieve_proof: 0,
                },
                store_boc: true,
                store_proof: true,
                retrieve_boc: true,
                retrieve_proof: true,
                verify_proof: true,
            }
        }
    
        // Stores the data in the storage node.
        pub async fn store_data(&mut self, boc: BOC, proof: ZkProof) -> Result<(), SystemError> {
            if self.store_boc || self.store_proof {
                self.storage_node
                    .as_ref()
                    .store_update(boc, proof)
                    .await
                    .map_err(|e| SystemError::new(SystemErrorType::InvalidAmount, e.to_string()))?;
                
                if self.store_boc {
                    self.metrics.store_boc += 1;
                }
                if self.store_proof {
                    self.metrics.store_proof += 1;
                }
            }
            Ok(())
        }    // Retrieves the data from the storage node.
        pub async fn retrieve_data(&self, boc_id: &[u8; 32]) -> Result<BOC, SystemError> {
            if self.retrieve_boc {
                self.storage_node
                    .as_ref()
                    .retrieve_boc(boc_id)
                    .await
                    .map_err(|e| SystemError::new(SystemErrorType::InvalidAmount, e.to_string()))
            } else {
                Err(SystemError::new(
                    SystemErrorType::InvalidAmount,
                    "Storage and retrieval verification is disabled".to_string(),
                ))
            }
        }
    
        // Retrieves the proof from the storage node.
        pub async fn retrieve_proof(&self, proof_id: &[u8; 32]) -> Result<ZkProof, SystemError> {
            if self.retrieve_proof {
                self.storage_node
                    .as_ref()
                    .retrieve_proof(proof_id)
                    .await
                    .map_err(|e| SystemError::new(SystemErrorType::InvalidAmount, e.to_string()))
            } else {
                Err(SystemError::new(
                    SystemErrorType::InvalidAmount,
                    "Storage and retrieval verification is disabled".to_string(),
                ))
            }
        }
        // Verifies the proof
        pub async fn verify_proof(&self, proof: &ZkProof) -> Result<bool, SystemError> {
            if self.verify_proof {
                let mut builder = ZkCircuitBuilder::<ark_bn254::Bn254, 2>::new(ProofGenerator::try_new().unwrap());
                let circuit = builder.build_circuit().unwrap();
                let mut pw = PartialWitness::new();
                fill_proof_witness(pw, &circuit, proof).unwrap();
                let proof = circuit.prove(pw).unwrap();
                match proof.verify() {
                    Ok(true) => Ok(true),
                    Ok(false) => Err(SystemError::new(
                        SystemErrorType::InvalidProof,
                        "Proof verification failed".to_string(),
                    )),
                    Err(e) => Err(SystemError::new(
                        SystemErrorType::VerificationError,
                        format!("Error during proof verification: {:?}", e)
                    )),
                }
            } else {
                Err(SystemError::new(
                    SystemErrorType::InvalidAmount,
                    "Storage and retrieval verification is disabled".to_string(),
                ))
            }
        }   
        // Returns the metrics for the storage and retrieval manager.
        pub fn get_metrics(&self) -> StorageAndRetrievalMetrics {
            self.metrics.clone()
        }
        // Sets the metrics for the storage and retrieval manager.
        pub fn set_metrics(&mut self, metrics: StorageAndRetrievalMetrics) {
            self.metrics = metrics;
        }   
        // Sets the storage and retrieval verification flags.
        pub fn set_verify_proof(&mut self, verify_proof: bool) {
            self.verify_proof = verify_proof;
        }   
        // Sets the storage and retrieval verification flags.
        pub fn set_store_boc(&mut self, store_boc: bool) {
            self.store_boc = store_boc;
        }   
        // Sets the storage and retrieval verification flags.
        pub fn set_store_proof(&mut self, store_proof: bool) {
            self.store_proof = store_proof;
        }
        // Sets the storage and retrieval verification flags.
        pub fn set_retrieve_boc(&mut self, retrieve_boc: bool) {
            self.retrieve_boc = retrieve_boc;
        }
        // Sets the storage and retrieval verification flags.
        pub fn set_retrieve_proof(&mut self, retrieve_proof: bool) {
            self.retrieve_proof = retrieve_proof;
        }
        // Sets the storage and retrieval verification flags.
        pub fn set_verify_proof(&mut self, verify_proof: bool) {
            self.verify_proof = verify_proof;
        }
        // Sets the storage and retrieval verification flags.
        pub fn set_verify_proof(&mut self, verify_proof: bool) {
            self.verify_proof = verify_proof;
        }   
        // Sets the storage and retrieval verification flags.
        pub fn set_retrieve_proof(&mut self, retrieve_proof: bool) {
            self.retrieve_proof = retrieve_proof;
        }
        // Sets the storage and retrieval verification flags.
        pub fn set_store_proof(&mut self, store_proof: bool) {
            self.store_proof = store_proof;
        }
        // Sets the storage and retrieval verification flags.
        pub fn set_store_boc(&mut self, store_boc: bool) {
            self.store_boc = store_boc;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;    
        use crate::core::storage_node::storage_node_contract::StorageNode;
        use crate::core::zkps::plonky2::Plonky2SystemHandle;
        use crate::core::zkps::proof::ZkProof;
        use crate::core::types::boc::BOC;
        use crate::core::types::ovp_ops::*;
        use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
        use crate::core::zkps::zkp::VirtualCell;
        use crate::core::storage_node::replication::state::StateManager;
        use crate::core::storage_node::replication::consistency::ConsistencyValidator;
        use crate::core::storage_node::replication::distribution::DistributionManager;
        use crate::core::storage_node::replication::verification::VerificationManager;
        use wasm_bindgen_test::*;
        wasm_bindgen_test_configure!(run_in_browser);
        async fn setup_storage_and_retrieval() -> StorageAndRetrievalManager {
            let storage_node = Arc::new(StorageNode::new(                
                [0u8; 32],
                0,
                StorageNodeConfig {
                    battery_config: crate::core::storage_node::storage_node_contract::BatteryConfig::default(),
                    sync_config: crate::core::storage_node::storage_node_contract::SyncConfig::default(),
                    epidemic_protocol_config: crate::core::storage_node::storage_node_contract::EpidemicProtocolConfig::default(),
                    network_config: crate::core::storage_node::storage_node_contract::NetworkConfig::default(),
                    node_id: [0u8; 32],
                    fee: 0,
                    whitelist: HashSet::new(),
                },
                HashMap::new()
            ).await.unwrap());
            let state_manager = StateManager::new().unwrap();
            let consistency_validator = ConsistencyValidator::new();
            let distribution_manager = DistributionManager::new();
            let verification_manager = VerificationManager::new(
                &state_manager,
                &consistency_validator,
                &distribution_manager,
            );
            StorageAndRetrievalManager::new(storage_node)
        }
        #[wasm_bindgen_test]
        async fn test_store_data() {
            let manager = setup_storage_and_retrieval().await;
            let boc = BOC {
                cells: vec![],
                references: vec![],
            };
            let proof = ZkProof {
                proof_data: vec![],
                public_inputs: vec![],
                merkle_root: vec![],
                timestamp: 0,
            };
            let result = manager.store_data(boc, proof).await;
            assert!(result.is_ok());
        }
        #[wasm_bindgen_test]
        async fn test_retrieve_data() {
            let manager = setup_storage_and_retrieval().await;
            let boc_id = [0u8; 32];
            let result = manager.retrieve_data(&boc_id).await;
            assert!(result.is_ok());
        }
        #[wasm_bindgen_test]
        async fn test_retrieve_proof() {
            let manager = setup_storage_and_retrieval().await;
            let proof_id = [0u8; 32];
            let result = manager.retrieve_proof(&proof_id).await;
            assert!(result.is_ok());
        }
        #[wasm_bindgen_test]
        async fn test_verify_proof() {
            let manager = setup_storage_and_retrieval().await;
            let proof = ZkProof {
                proof_data: vec![],
                public_inputs: vec![],
                merkle_root: vec![],
                timestamp: 0,
            };
            let result = manager.verify_proof(&proof).await;
            assert!(result.is_ok());
        }   

            let manager = setup_storage_and_retrieval().await;
            let proof_id = [0u8; 32];
            let result = manager.verify_proof(&proof_id).await;
            assert!(result.is_ok());
        }   
        #[wasm_bindgen_test]
        async fn test_retrieve_proof() {
            let manager = setup_storage_and_retrieval().await;
            let proof_id = [0u8; 32];
            let result = manager.retrieve_proof(&proof_id).await;
            assert!(result.is_ok());
        }
    #[wasm_bindgen_test]
    async fn test_verify_proof() {
            let manager = setup_storage_and_retrieval().await;
            let proof = ZkProof {
                proof_data: vec![],
                public_inputs: vec![],
                merkle_root: vec![],
                timestamp: 0,
            };
            let result = manager.verify_proof(&proof).await;
            assert!(result.is_ok());
        }   
        #[wasm_bindgen_test]
        async fn test_retrieve_proof() {
            let manager = setup_storage_and_retrieval().await;
            let proof_id = [0u8; 32];
            let result = manager.retrieve_proof(&proof_id).await;
            assert!(result.is_ok());
        }   
        #[wasm_bindgen_test]
        async fn test_retrieve_proof() {
            let manager = setup_storage_and_retrieval().await;
            let proof_id = [0u8; 32];
            let result = manager.retrieve_proof(&proof_id).await;
            assert!(result.is_ok());
            
       }
       #[wasm_bindgen_test]
       async fn test_store_data() {
           let manager = setup_storage_and_retrieval().await;
           let boc = BOC {
               cells: vec![],
               references: vec![],
           };
           let proof = ZkProof {
               proof_data: vec![],
               public_inputs: vec![],
               merkle_root: vec![],
               timestamp: 0,
           };
           let result = manager.store_data(boc, proof).await;
           assert!(result.is_ok());
       }    
       #[wasm_bindgen_test]
       async fn test_retrieve_data() {
           let manager = setup_storage_and_retrieval().await;
           let boc_id = [0u8; 32];
           let result = manager.retrieve_data(&boc_id).await;
           assert!(result.is_ok());
       }
       #[wasm_bindgen_test]
       async fn test_retrieve_proof() {
           let manager = setup_storage_and_retrieval().await;
           let proof_id = [0u8; 32];
           let result = manager.retrieve_proof(&proof_id).await;
           assert!(result.is_ok());
       }
       #[wasm_bindgen_test]
       async fn test_verify_proof() {
           let manager = setup_storage_and_retrieval().await;
           let proof = ZkProof {
               proof_data: vec![],
               public_inputs: vec![],
               merkle_root: vec![],
               timestamp: 0,
           };
           let result = manager.verify_proof(&proof).await;
           assert!(result.is_ok());
       }    
       #[wasm_bindgen_test]
       async fn test_retrieve_proof() {
           let manager = setup_storage_and_retrieval().await;
           let proof_id = [0u8; 32];
           let result = manager.retrieve_proof(&proof_id).await;
           assert!(result.is_ok());
       }    
       #[wasm_bindgen_test]
       async fn test_retrieve_proof() {
           let manager = setup_storage_and_retrieval().await;
           let proof_id = [0u8; 32];
           let result = manager.retrieve_proof(&proof_id).await;
           assert!(result.is_ok());
           let manager = setup_storage_and_retrieval().await;
           let proof_id = [0u8; 32];
           let result = manager.retrieve_proof(&proof_id).await;
           assert!(result.is_ok());    
        }
