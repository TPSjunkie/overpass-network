use std::sync::Arc;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::hash::hash_types::RichField;
use plonky2_field::extension::Extendable;
use serde::{Serialize, Deserialize};
    
use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
use crate::core::storage_node::storage_node_contract::StorageNode;

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

pub struct StorageAndRetrievalManager<F: RichField + Extendable<2>> {
    storage_node: Arc<StorageNode>,
    metrics: StorageAndRetrievalMetrics,
    store_boc: bool,
    store_proof: bool,
    retrieve_boc: bool,
    retrieve_proof: bool,
    verify_proof: bool,
    _marker: std::marker::PhantomData<F>,
}

impl<F: RichField + Extendable<2>> StorageAndRetrievalManager<F> {
    pub fn new(storage_node: Arc<StorageNode>) -> Self {
        Self {
            storage_node,
            metrics: StorageAndRetrievalMetrics::default(),
            store_boc: true,
            store_proof: true,
            retrieve_boc: true,
            retrieve_proof: true,
            verify_proof: true,
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn store_data(&mut self, boc: BOC, proof: ZkProof) -> Result<(), SystemError> {
        if self.store_boc || self.store_proof {
            self.storage_node
                .store(&boc, &proof)
                .await
                .map_err(|e| SystemError::new(SystemErrorType::StorageError, e.to_string()))?;
            
            if self.store_boc {
                self.metrics.store_boc += 1;
            }
            if self.store_proof {
                self.metrics.store_proof += 1;
            }
        }
        Ok(())
    }

    pub async fn retrieve_data(&self, boc_id: &[u8; 32]) -> Result<BOC, SystemError> {
        if self.retrieve_boc {
            let boc = self.storage_node
    .get_boc(boc_id)
    .await
    .map_err(|e| SystemError::new(SystemErrorType::StorageError, e.to_string()))?;  
    
    self.metrics.retrieve_boc += 1;
    Ok(boc)
    } else {
    Err(SystemError::new(
    SystemErrorType::OperationDisabled,
    "Storage and retrieval retrieval is disabled".to_string(),
    ))
    }
    }   

    pub async fn retrieve_proof(&self, proof_id: &[u8; 32]) -> Result<ZkProof, SystemError> {
        if self.retrieve_proof {
            let proof = self.storage_node
                .get_proof(proof_id)
                .await
                .map_err(|e| SystemError::new(SystemErrorType::StorageError, e.to_string()))?;

            self.metrics.retrieve_proof += 1;
            Ok(proof)
        } else {
            Err(SystemError::new(
                SystemErrorType::OperationDisabled,
                "Storage and retrieval verification is disabled".to_string(),
            ))
        }
    }

    pub async fn verify_proof(&self, proof: &ZkProof) -> Result<bool, SystemError> {
        if self.verify_proof {
            let circuit_proof = ZkCircuitBuilder::new(proof.clone());
            let circuit_proof = circuit_proof.build().map_err(|e| {
                SystemError::new(
                    SystemErrorType::CircuitError,
                    format!("Failed to build circuit: {:?}", e),
                )
            })?;
            let proof_bytes = bincode::serialize(&proof).map_err(|e| {
                SystemError::new(
                    SystemErrorType::SerializationError,
                    format!("Failed to serialize proof: {}", e),
                )
            })?;
            let proof_bytes = proof_bytes.to_vec();
            match circuit_proof.verify() {
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
                SystemErrorType::OperationDisabled,
                "Storage and retrieval verification is disabled".to_string(),
            ))
        }
    }

    pub fn get_metrics(&self) -> StorageAndRetrievalMetrics {
        self.metrics.clone()
    }

    pub fn set_metrics(&mut self, metrics: StorageAndRetrievalMetrics) {
        self.metrics = metrics;
    }

    pub fn set_verify_proof(&mut self, verify_proof: bool) {
        self.verify_proof = verify_proof;
    }

    pub fn set_store_boc(&mut self, store_boc: bool) {
        self.store_boc = store_boc;
    }

    pub fn set_store_proof(&mut self, store_proof: bool) {
        self.store_proof = store_proof;
    }

    pub fn set_retrieve_boc(&mut self, retrieve_boc: bool) {
        self.retrieve_boc = retrieve_boc;
    }

    pub fn set_retrieve_proof(&mut self, retrieve_proof: bool) {
        self.retrieve_proof = retrieve_proof;
    }
}

#[cfg(test)]
mod tests {
    use super::*;    
    use crate::core::storage_node::storage_node_contract::{
        StorageNodeConfig, BatteryConfig, SyncConfig, 
        EpidemicProtocolConfig, NetworkConfig
    };
    use plonky2::field::goldilocks_field::GoldilocksField;
    use std::collections::HashSet;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    type F = GoldilocksField;

    async fn setup_storage_and_retrieval() -> StorageAndRetrievalManager<F> {
        let storage_node = Arc::new(StorageNode::new(                
            [0u8; 32],
            0,
            StorageNodeConfig {
                battery_config: BatteryConfig::default(),
                sync_config: SyncConfig::default(),
                epidemic_protocol_config: EpidemicProtocolConfig::default(),
                network_config: NetworkConfig::default(),
                node_id: [0u8; 32],
                fee: 0,
                whitelist: HashSet::new(),
            },
        ).unwrap());

        StorageAndRetrievalManager::new(storage_node)
    }

    #[wasm_bindgen_test]
    async fn test_store_data() {
        let mut manager = setup_storage_and_retrieval().await;
        let boc = BOC {
            cells: vec![],
            references: vec![],
            roots: vec![],
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
}