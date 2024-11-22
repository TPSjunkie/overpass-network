use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::types::boc::BOC;
use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
use crate::core::zkps::proof::ZkProof;
use plonky2::hash::hash_types::RichField;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2_field::extension::Extendable;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAndRetrievalMetrics {
    pub store_boc: u64,
    pub store_proof: u64,
    pub retrieve_boc: u64,
    pub retrieve_proof: u64,
    pub verification_count: u64,
    pub verification_success: u64,
    pub verification_failure: u64,
}

impl Default for StorageAndRetrievalMetrics {
    fn default() -> Self {
        Self {
            store_boc: 0,
            store_proof: 0,
            retrieve_boc: 0,
            retrieve_proof: 0,
            verification_count: 0,
            verification_success: 0,
            verification_failure: 0,
        }
    }
}

pub struct StorageAndRetrievalManager<F: RichField + Extendable<2>, StorageNode> {
    storage_node: Arc<StorageNode>,
    metrics: StorageAndRetrievalMetrics,
    store_boc: bool,
    store_proof: bool,
    retrieve_boc: bool,
    retrieve_proof: bool,
    verify_proof: bool,
    circuit_config: CircuitConfig,
    _marker: std::marker::PhantomData<F>,
}
impl<F: RichField + Extendable<2>, StorageNode> StorageAndRetrievalManager<F, StorageNode> {
    pub fn new(storage_node: Arc<StorageNode>) -> Self {
        Self {
            storage_node,
            metrics: StorageAndRetrievalMetrics::default(),
            store_boc: true,
            store_proof: true,
            retrieve_boc: true,
            retrieve_proof: true,
            verify_proof: true,
            circuit_config: CircuitConfig::default(),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn store_data(&mut self, boc: BOC, proof: ZkProof) -> Result<(), SystemError> {
        if !self.store_boc && !self.store_proof {
            return Err(SystemError::new(
                SystemErrorType::OperationDisabled,
                "Both BOC and proof storage are disabled".to_string(),
            ));
        }

        // Verify the proof before storing if enabled
        if self.verify_proof {
            self.verify_proof(&proof).await?;
        }

        if self.store_boc {
            self.storage_node
                .stored_bocs
                .lock()
                .await
                .insert(boc.hash(), boc.clone());
            self.metrics.store_boc += 1;
        }

        if self.store_proof {
            self.storage_node
                .stored_proofs
                .lock()
                .await
                .insert(proof.hash(), proof);
            self.metrics.store_proof += 1;
        }

        Ok(())
    }

    pub async fn retrieve_data(&self, boc_id: &[u8; 32]) -> Result<BOC, SystemError> {
        if !self.retrieve_boc {
            return Err(SystemError::new(
                SystemErrorType::OperationDisabled,
                "BOC retrieval is disabled".to_string(),
            ));
        }

        let boc = self
            .storage_node
            .stored_bocs
            .lock()
            .await
            .get(boc_id)
            .cloned()
            .ok_or_else(|| {
                SystemError::new(SystemErrorType::StorageError, "BOC not found".to_string())
            })?;

        self.metrics.retrieve_boc += 1;
        Ok(boc)
    }

    pub async fn retrieve_proof(&self, proof_id: &[u8; 32]) -> Result<ZkProof, SystemError> {
        if !self.retrieve_proof {
            return Err(SystemError::new(
                SystemErrorType::OperationDisabled,
                "Proof retrieval is disabled".to_string(),
            ));
        }

        let proof = self
            .storage_node
            .stored_proofs
            .lock()
            .await
            .get(proof_id)
            .cloned()
            .ok_or_else(|| {
                SystemError::new(SystemErrorType::StorageError, "Proof not found".to_string())
            })?;

        self.metrics.retrieve_proof += 1;
        Ok(proof)
    }

    pub async fn verify_proof(&self, proof: &ZkProof) -> Result<bool, SystemError> {
        if !self.verify_proof {
            return Err(SystemError::new(
                SystemErrorType::OperationDisabled,
                "Proof verification is disabled".to_string(),
            ));
        }

        self.metrics.verification_count += 1;

        let mut circuit_builder = ZkCircuitBuilder::<F, 2>::new(self.circuit_config.clone());

        let circuit = circuit_builder
            .build_verification_circuit(proof)
            .map_err(|e| {
                self.metrics.verification_failure += 1;
                SystemError::new(
                    SystemErrorType::CircuitError,
                    format!("Failed to build verification circuit: {:?}", e),
                )
            })?;

        let result = circuit.verify_proof(proof).map_err(|e| {
            self.metrics.verification_failure += 1;
            SystemError::new(
                SystemErrorType::VerificationError,
                format!("Proof verification failed: {:?}", e),
            )
        })?;

        if result {
            self.metrics.verification_success += 1;
        } else {
            self.metrics.verification_failure += 1;
        }

        Ok(result)
    }

    // Configuration methods
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

    pub fn set_circuit_config(&mut self, config: CircuitConfig) {
        self.circuit_config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage_node::storage_node_config::EpidemicProtocolConfig;
    use crate::core::storage_node::storage_node_config::NetworkConfig;
    use crate::core::storage_node::storage_node_config::StorageNodeConfig;

    use crate::core::storage_node::battery::charging::BatteryConfig;
    use crate::core::storage_node::epidemic::sync::SyncConfig;
    use crate::core::storage_node::storage_node_contract::StorageAndRetrievalManager;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use std::collections::HashSet;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    type F = GoldilocksField;

    async fn setup_storage_and_retrieval<StorageNode: Default>(
    ) -> StorageAndRetrievalManager<F, StorageNode> {
        let storage_node = Arc::new(StorageNode::default());

        StorageAndRetrievalManager::new(storage_node)
    }
    async fn create_test_data() -> (BOC, ZkProof) {
        // Function body goes here
        let boc = BOC::new();
        let proof = ZkProof::default();
        (boc, proof)
    }

    #[wasm_bindgen_test]
    async fn test_storage_and_retrieval() {
        let mut manager = setup_storage_and_retrieval().await;
        let (boc, proof) = create_test_data().await;

        // Test storage
        let result = manager.store_data(boc.clone(), proof.clone()).await;
        assert!(result.is_ok());

        // Test retrieval
        let retrieved_boc = manager.retrieve_data(&boc.hash()).await;
        assert!(retrieved_boc.is_ok());
        assert_eq!(retrieved_boc.unwrap().hash(), boc.hash());

        let retrieved_proof = manager.retrieve_proof(&proof.hash()).await;
        assert!(retrieved_proof.is_ok());
        assert_eq!(retrieved_proof.unwrap().hash(), proof.hash());
    }

    #[wasm_bindgen_test]
    async fn test_error_handling() {
        let mut manager = setup_storage_and_retrieval().await;

        // Test disabled operations
        manager.set_store_boc(false);
        manager.set_store_proof(false);

        let (boc, proof) = create_test_data().await;
        let result = manager.store_data(boc, proof).await;
        assert!(result.is_err());

        // Test not found errors
        let result = manager.retrieve_data(&[1u8; 32]).await;
        assert!(result.is_err());

        let result = manager.retrieve_proof(&[1u8; 32]).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn test_verification() {
        let manager = setup_storage_and_retrieval().await;
        let (_, proof) = create_test_data().await;

        let result = manager.verify_proof(&proof).await;
        assert!(result.is_ok());

        let metrics = manager.get_metrics();
        assert!(metrics.verification_count > 0);
        assert!(metrics.verification_success > 0);
    }

    #[wasm_bindgen_test]
    async fn test_metrics() {
        let mut manager = setup_storage_and_retrieval().await;
        let (boc, proof) = create_test_data().await;

        manager
            .store_data(boc.clone(), proof.clone())
            .await
            .unwrap();
        manager.retrieve_data(&boc.hash()).await.unwrap();
        manager.retrieve_proof(&proof.hash()).await.unwrap();
        manager.verify_proof(&proof).await.unwrap();

        let metrics = manager.get_metrics();
        assert_eq!(metrics.store_boc, 1);
        assert_eq!(metrics.store_proof, 1);
        assert_eq!(metrics.retrieve_boc, 1);
        assert_eq!(metrics.retrieve_proof, 1);
        assert!(metrics.verification_count > 0);
    }
}
