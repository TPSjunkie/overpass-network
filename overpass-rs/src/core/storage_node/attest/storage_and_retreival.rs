// ./src/core/storage_node/verification/storage_and_retrieval.rs

// Storage and Retrieval Verification
// This module is the entry point for accessing the storage nodes' storage and retrieval capabilities.
// It provides methods for storing and retrieving data, as well as verifying the proofs provided by the storage nodes.

use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::storage_node_contract::StorageNode;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use std::sync::Arc;

#[derive(Clone)]
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
}

#[derive(Clone)]
pub struct StorageAndRetrievalMetrics {
    pub store_boc: u64,
    pub store_proof: u64,
    pub retrieve_boc: u64,
    pub retrieve_proof: u64,
}

impl StorageAndRetrievalMetrics {
    pub fn new() -> Self {
        Self {
            store_boc: 0,
            store_proof: 0,
            retrieve_boc: 0,
            retrieve_proof: 0,
        }
    }
}

impl Default for StorageAndRetrievalMetrics {
    fn default() -> Self {
        Self::new()
    }
}
