use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::hierarchy::intermediate::sparse_merkle_tree_i::MerkleNode;
use crate::core::types::boc::BOC;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateType {
    Wallet,
    Intermediate,
    Root,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMetrics {
    pub total_states: usize,
    pub wallet_states: usize,
    pub intermediate_states: usize,
    pub state_updates: u64,
    pub merkle_updates: u64,
    pub verification_time: f64,
}

impl Default for StateMetrics {
    fn default() -> Self {
        Self {
            total_states: 0,
            wallet_states: 0,
            intermediate_states: 0,
            state_updates: 0,
            merkle_updates: 0,
            verification_time: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationState {
    pub state_hash: [u8; 32],
    pub state_type: StateType,
    pub state_metrics: StateMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyState {
    pub state_hash: [u8; 32],
    pub state_type: StateType,
    pub state_metrics: StateMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionState {
    pub state_hash: [u8; 32],
    pub state_type: StateType,
    pub state_metrics: StateMetrics,
}

pub struct StateManager {
    wallet_proofs: RwLock<HashMap<[u8; 32], Vec<[u8; 32]>>>,
    intermediate_trees: RwLock<HashMap<[u8; 32], MerkleNode>>,
    root_tree: RwLock<MerkleNode>,
    state_hashes: RwLock<HashMap<[u8; 32], StateType>>,
    state_updates: RwLock<HashMap<[u8; 32], u64>>,
    metrics: RwLock<StateMetrics>,
}

impl StateManager {
    fn update_merkle_tree(
        &self,
        tree: &mut MerkleNode,
        path: [u8; 32],
        new_node: MerkleNode,
    ) -> Result<(), SystemError> {
        let mut current = tree;

        for i in 0..256 {
            let bit = (path[i / 8] >> (7 - (i % 8))) & 1;

            match bit {
                0 => {
                    if current.left.is_none() {
                        current.left = Some(Box::new(new_node.clone()));
                        break;
                    }
                    if let Some(ref mut left) = current.left {
                        current = left;
                    }
                }
                _ => {
                    if current.right.is_none() {
                        current.right = Some(Box::new(new_node.clone()));
                        break;
                    }
                    if let Some(ref mut right) = current.right {
                        current = right;
                    }
                }
            }
        }

        self.metrics.write().merkle_updates += 1;
        Ok(())
    }

    pub fn generate_intermediate_proof(
        &self,
        state_hash: [u8; 32],
    ) -> Result<Vec<[u8; 32]>, SystemError> {
        let trees = self.intermediate_trees.read();
        let mut proof = Vec::new();

        for tree in trees.values() {
            if self.collect_proof_nodes(tree, &state_hash, &mut proof)? {
                return Ok(proof);
            }
        }

        Err(SystemError::new(
            SystemErrorType::InvalidInput,
            "State hash not found in intermediate trees".to_string(),
        ))
    }

    fn collect_proof_nodes(
        &self,
        node: &MerkleNode,
        target_hash: &[u8; 32],
        proof: &mut Vec<[u8; 32]>,
    ) -> Result<bool, SystemError> {
        if let Some(hash) = node.hash {
            if hash == *target_hash {
                proof.push(hash);
                return Ok(true);
            }
        }

        if let Some(ref left) = node.left {
            if self.collect_proof_nodes(left, target_hash, proof)? {
                if let Some(hash) = node.hash {
                    proof.push(hash);
                }
                return Ok(true);
            }
        }

        if let Some(ref right) = node.right {
            if self.collect_proof_nodes(right, target_hash, proof)? {
                if let Some(hash) = node.hash {
                    proof.push(hash);
                }
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn get_current_state(&self) -> Result<BOC, SystemError> {
        let root = self.root_tree.read();
        if let Some(hash) = root.hash {
            if let Some(data) = &root.data {
                return BOC::deserialize(data)?;
            }
        }
        Err(SystemError::new(
            SystemErrorType::InvalidState,
            "No current state".to_string(),
        ))
    }

    pub fn compute_state_hash(&self, state: &BOC) -> Result<[u8; 32], SystemError> {
        let mut hasher = Sha256::new();
        let bytes = bincode::serialize(state)
            .map_err(|e| SystemError::new(SystemErrorType::SerializationError, e.to_string()))?;
        hasher.update(&bytes);
        Ok(hasher.finalize().into())
    }

    fn update_merkle_tree(
        &self,
        tree: &mut MerkleNode,
        path: [u8; 32],
        node: MerkleNode,
    ) -> Result<(), SystemError> {
        let mut current = tree;

        for i in 0..256 {
            let bit = (path[i / 8] >> (7 - (i % 8))) & 1;

            if bit == 0 {
                if current.left.is_none() {
                    current.left = Some(Box::new(node));
                    return Ok(());
                }
                current = current.left.as_mut().unwrap();
            } else {
                if current.right.is_none() {
                    current.right = Some(Box::new(node));
                    return Ok(());
                }
                current = current.right.as_mut().unwrap();
            }
        }

        self.metrics.write().merkle_updates += 1;
        Ok(())
    }

    pub fn generate_proof(&self, state_hash: [u8; 32]) -> Result<Vec<[u8; 32]>, SystemError> {
        let binding = self.state_hashes.read();
        let state_type = binding.get(&state_hash).ok_or_else(|| {
            SystemError::new(
                SystemErrorType::InvalidInput,
                "State hash not found".to_string(),
            )
        })?;

        let proof = match state_type {
            StateType::Wallet => {
                // Find the proof path from wallet proofs
                let proofs = self.wallet_proofs.read();
                proofs
                    .values()
                    .find(|p| p.contains(&state_hash))
                    .cloned()
                    .ok_or_else(|| {
                        SystemError::new(
                            SystemErrorType::InvalidInput,
                            "Proof not found for wallet state".to_string(),
                        )
                    })?
            }
            StateType::Intermediate => {
                // Generate intermediate proof
                self.generate_intermediate_proof(state_hash)?
            }
            StateType::Root => vec![state_hash],
        };

        Ok(proof)
    }

    fn generate_intermediate_proof(
        &self,
        state_hash: [u8; 32],
    ) -> Result<Vec<[u8; 32]>, SystemError> {
        let trees = self.intermediate_trees.read();
        let mut proof = Vec::new();

        for tree in trees.values() {
            if self.collect_proof_nodes(tree, state_hash, &mut proof)? {
                break;
            }
        }

        if proof.is_empty() {
            return Err(SystemError::new(
                SystemErrorType::InvalidInput,
                "State hash not found in intermediate trees".to_string(),
            ));
        }

        Ok(proof)
    }

    fn collect_proof_nodes(
        &self,
        node: &MerkleNode,
        target_hash: [u8; 32],
        proof: &mut Vec<[u8; 32]>,
    ) -> Result<bool, SystemError> {
        if let Some(hash) = node.hash {
            if hash == target_hash {
                proof.push(hash);
                return Ok(true);
            }
        }

        if let Some(ref left) = node.left {
            if self.collect_proof_nodes(left, target_hash, proof)? {
                if let Some(hash) = node.hash {
                    proof.push(hash);
                }
                return Ok(true);
            }
        }

        if let Some(ref right) = node.right {
            if self.collect_proof_nodes(right, target_hash, proof)? {
                if let Some(hash) = node.hash {
                    proof.push(hash);
                }
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn verify_state(&self, state_hash: [u8; 32]) -> bool {
        self.state_hashes.read().contains_key(&state_hash)
    }

    pub fn get_state_type(&self, state_hash: [u8; 32]) -> Option<StateType> {
        self.state_hashes.read().get(&state_hash).cloned()
    }

    pub fn get_metrics(&self) -> StateMetrics {
        self.metrics.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn create_test_boc() -> BOC {
        BOC {
            cells: vec![],
            references: vec![],
            roots: vec![],
            hash: todo!(),
        }
    }

    #[wasm_bindgen_test]
    async fn test_wallet_state_updates() {
        let manager = StateManager::new().unwrap();
        let wallet_id = [1u8; 32];
        let state = create_test_boc();
        let proof = vec![[0u8; 32]];

        let hash = manager
            .update_wallet_state(wallet_id, state, proof)
            .unwrap();
        assert!(manager.verify_state(hash));
        assert_eq!(manager.get_state_type(hash), Some(StateType::Wallet));
    }

    #[wasm_bindgen_test]
    async fn test_intermediate_state_updates() {
        let manager = StateManager::new().unwrap();
        let contract_id = [2u8; 32];
        let state = create_test_boc();

        let hash = manager
            .update_intermediate_state(contract_id, state)
            .unwrap();
        assert!(manager.verify_state(hash));
        assert_eq!(manager.get_state_type(hash), Some(StateType::Intermediate));
    }

    #[wasm_bindgen_test]
    async fn test_root_state_updates() {
        let manager = StateManager::new().unwrap();
        let state = create_test_boc();

        let hash = manager.update_root_state(state).unwrap();
        assert!(manager.verify_state(hash));
        assert_eq!(manager.get_state_type(hash), Some(StateType::Root));
    }

    #[wasm_bindgen_test]
    async fn test_proof_generation() {
        let manager = StateManager::new().unwrap();
        let wallet_id = [3u8; 32];
        let state = create_test_boc();
        let proof = vec![[0u8; 32]];

        let hash = manager
            .update_wallet_state(wallet_id, state, proof)
            .unwrap();
        let generated_proof = manager.generate_proof(hash).unwrap();
        assert!(!generated_proof.is_empty());
    }

    #[wasm_bindgen_test]
    async fn test_metrics() {
        let manager = StateManager::new().unwrap();
        let state = create_test_boc();

        manager
            .update_wallet_state([1u8; 32], state.clone(), vec![[0u8; 32]])
            .unwrap();
        manager
            .update_intermediate_state([2u8; 32], state.clone())
            .unwrap();
        manager.update_root_state(state).unwrap();

        let metrics = manager.get_metrics();
        assert_eq!(metrics.total_states, 3);
        assert_eq!(metrics.wallet_states, 1);
        assert_eq!(metrics.intermediate_states, 1);
    }
}
