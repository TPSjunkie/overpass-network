// ./src/core/storage_node/replication/consistency.rs

use crate::core::error::{SystemError, SystemErrorType};
use crate::core::storage_node::storage_node_contract::StorageNode;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use std::sync::Arc;

pub struct ConsistencyManager {
    storage_node: Arc<StorageNode>,
    metrics: ConsistencyMetrics,
    verify_boc: bool,
    verify_proof: bool,
}

impl ConsistencyManager {
    pub fn new(storage_node: Arc<StorageNode>) -> Self {
        Self {
            storage_node,
            metrics: ConsistencyMetrics::default(),
            verify_boc: true,
            verify_proof: true,
        }
    }
    
    // Verify the BOC consistency
    pub async fn verify_boc_consistency(&self, boc: BOC) -> Result<(), SystemError> {
        if self.verify_boc {
            self.storage_node
                .verify_boc_consistency(&boc)
                .await
                .map_err(|e| SystemError::new(SystemErrorType::InvalidAmount, e.to_string()))?;
            self.metrics.verify_boc += 1;
        }
        Ok(())
    }

    // Verify the Proof consistency
    pub async fn verify_proof_consistency(&self, proof: ZkProof) -> Result<(), SystemError> {
        if self.verify_proof {
            self.storage_node
                .as_ref()
                .verify_proof_consistency(&proof)
                .await
                .map_err(|e| SystemError::new(SystemErrorType::InvalidAmount, e.to_string()))?;
            self.metrics.verify_proof += 1;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ConsistencyMetrics {
    pub verify_boc: u64,
    pub verify_proof: u64,
}    

impl Default for ConsistencyMetrics {
    fn default() -> Self {
        Self {
            verify_boc: 0,
            verify_proof: 0,
        }
    }
}