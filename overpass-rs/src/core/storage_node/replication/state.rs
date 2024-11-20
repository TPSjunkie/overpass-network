use parking_lot::RwLock;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::types::boc::BOC;
use crate::core::hierarchy::intermediate::sparse_merkle_tree_i::MerkleNode;
use plonky2::{field::goldilocks_field::GoldilocksField, hash::hash_types::RichField};

use plonky2::hash::poseidon::PoseidonHash;


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateType {
    Wallet,
    Intermediate, 
    Root,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    InvalidStateType,
    InvalidStateHash,
    StateNotFound,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationState {
    pub replication_state: ReplicationState,
    pub consistency_state: ConsistencyState,
    pub distribution_state: DistributionState,
}

pub struct StateManager {
    // Remove wallet_tree
    wallet_proofs: RwLock<HashMap<[u8; 32], Vec<[u8; 32]>>>, // wallet_id -> proof path
    intermediate_trees: RwLock<HashMap<[u8; 32], MerkleNode>>,
    root_tree: RwLock<MerkleNode>,
    state_hashes: RwLock<HashMap<[u8; 32], StateType>>,
    state_updates: RwLock<HashMap<[u8; 32], u64>>,
    metrics: RwLock<StateMetrics>,
}
impl StateManager {
    pub fn new() -> Result<Self, SystemError> {
        let metrics = StateMetrics::default();
        Ok(Self {
            wallet_proofs: RwLock::new(HashMap::new()),
            intermediate_trees: RwLock::new(HashMap::new()),
            root_tree: RwLock::new(MerkleNode {
                hash: None,
                data: None,
                left: None,
                right: None,
                virtual_cell: None,
                value: None,
                is_leaf: false,
                is_virtual: false,
                is_empty: true,
            }),
            state_hashes: RwLock::new(HashMap::new()),
            state_updates: RwLock::new(HashMap::new()),
            metrics: RwLock::new(metrics),
        })
    }
}

impl StateManager {
    pub async fn update_wallet_state(
        &self,
        wallet_id: [u8; 32],
        state: BOC,
        proof: Vec<[u8; 32]>,
    ) -> Result<[u8; 32], SystemError> {
        let state_hash = self.compute_state_hash(&state);
        
        // Store just the proof path
        self.wallet_proofs.write().insert(wallet_id, proof);
        self.state_hashes.write().insert(state_hash, StateType::Wallet);
        
        let mut metrics = self.metrics.write();
        metrics.wallet_states += 1;
        metrics.total_states += 1;
        metrics.state_updates += 1;
        
        Ok(state_hash)
    }
    pub async fn update_intermediate_state(
        &self,
        contract_id: [u8; 32],
        state: BOC,
    ) -> Result<[u8; 32], SystemError> {
        let state_hash = self.compute_state_hash(&state);
        let new_node = MerkleNode {
            hash: Some(state_hash),
            data: Some(state.serialize().map_err(|e| SystemError::new(SystemErrorType::VerificationError, e.to_string()))?),
            left: None,
            right: None,
            virtual_cell: None,
            value: None,
            is_leaf: true,
            is_virtual: false,
            is_empty: false,
        };
        self.intermediate_trees
            .write()
            .insert(contract_id, new_node);

        self.state_hashes
            .write()
            .insert(state_hash, StateType::Intermediate);
        *self.state_updates.write().entry(state_hash).or_insert(0) += 1;

        let mut metrics = self.metrics.write();
        metrics.intermediate_states += 1;
        metrics.total_states += 1;
        metrics.state_updates += 1;

        Ok(state_hash)
    }
    pub async fn update_root_state(
        &self,
        state: BOC,
    ) -> Result<[u8; 32], SystemError> {
        let state_hash = self.compute_state_hash(&state);
        let new_node = MerkleNode {
            hash: Some(state_hash),
            data: Some(state.serialize().map_err(|e| SystemError::new(SystemErrorType::VerificationError, e.to_string()))?),
            left: None,
            right: None,
            virtual_cell: None,
            value: None,
            is_leaf: true,
            is_virtual: false,
            is_empty: false,
        };
        let mut root_tree = self.root_tree.write().await;
        self.update_merkle_tree(&mut root_tree, state_hash, new_node)?;
        self.state_hashes.write().await.insert(state_hash, StateType::Root);
        *self.state_updates.write().await.entry(state_hash).or_insert(0) += 1;

        let mut metrics = self.metrics.write().await;
        metrics.total_states += 1;
        metrics.state_updates += 1;

        Ok(state_hash)
    }


    fn compute_state_hash(&self, state: &BOC) -> [u8; 32] { 
        let mut hasher = Sha256::new();
        let bytes = bincode::serialize(state).unwrap_or_default();
        hasher.update(&bytes);
        hasher.finalize().into()
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
                    break;
                }
                if let Some(ref mut left) = current.left {
                    current = left;
                }
            } else {
                if current.right.is_none() {
                    current.right = Some(Box::new(node));
                    break;
                }
                if let Some(ref mut right) = current.right {
                    current = right;
                }
            }
        }

        self.metrics.write().merkle_updates += 1;
        Ok(())
    }

    pub async fn update_wallet_state_with_proof(
        &self,
        wallet_id: [u8; 32],
        state: BOC,
    ) -> Result<[u8; 32], SystemError> {
        let mut wallet_tree = self.wallet_tree.write().await;
        let state_hash = self.compute_state_hash(&state);
        let new_node = MerkleNode::from_data(state_hash, state);
        self.update_merkle_tree(&mut wallet_tree, wallet_id, new_node)?;

        self.state_hashes.write().await.insert(state_hash, StateType::Wallet);
        *self.state_updates.write().await.entry(state_hash).or_insert(0) += 1;

        let mut metrics = self.metrics.write().await;
        metrics.wallet_states += 1;
        metrics.total_states += 1;
        metrics.state_updates += 1;

        Ok(state_hash)
    }
    pub async fn update_root_state(&self, state: BOC) -> Result<[u8; 32], SystemError> {
        let mut root_tree = self.root_tree.write().await;
        let state_hash = self.compute_state_hash(&state);
        let new_node = MerkleNode::from_data(state_hash, state);
        self.update_merkle_tree(&mut root_tree, state_hash, new_node)?;

        self.state_hashes
            .write()
            .await
            .insert(state_hash, StateType::Root);
        *self
            .state_updates
            .write()
            .await
            .entry(state_hash)
            .or_insert(0) += 1;

        let mut metrics = self.metrics.write().await;
        metrics.total_states += 1;
        metrics.state_updates += 1;

        Ok(state_hash)
    }    pub async fn update_intermediate_state(
        &self,
        contract_id: [u8; 32],
        state: BOC,
    ) -> Result<[u8; 32], SystemError> {
        let mut intermediate_trees = self.intermediate_trees.write();
        let state_hash = self.compute_state_hash(&state);
        let new_node = MerkleNode::from_data(state_hash, state);
        intermediate_trees.insert(contract_id, new_node);
        
        self.state_hashes.write().insert(state_hash, StateType::Intermediate);
        *self.state_updates.write().entry(state_hash).or_insert(0) += 1;
        
        let mut metrics = self.metrics.write();
        metrics.intermediate_states += 1;
        metrics.total_states += 1;
        metrics.state_updates += 1;
        
        Ok(state_hash)
    }

    pub async fn update_root_state(&self, state: BOC) -> Result<[u8; 32], SystemError> {
        let mut root_tree = self.root_tree.write();
        let state_hash = self.compute_state_hash(&state);
        let new_node = MerkleNode::from_data(state_hash, state);
        *root_tree = new_node;
        
        self.state_hashes.write().insert(state_hash, StateType::Root);
        *self.state_updates.write().entry(state_hash).or_insert(0) += 1;
        
        let mut metrics = self.metrics.write();
        metrics.total_states += 1;
        metrics.state_updates += 1;
        
        Ok(state_hash)
    }

    pub fn generate_proof(&self, state_hash: [u8; 32]) -> Result<Vec<[u8; 32]>, SystemError> {
        let state_type = self.state_hashes.read().get(&state_hash).ok_or_else(|| 
            SystemError::new(SystemErrorType::InvalidInput, "State hash not found".to_string())
        )?;
        
        let proof = match state_type {
            StateType::Wallet => self.generate_wallet_proof(state_hash)?,
            StateType::Intermediate => self.generate_intermediate_proof(state_hash)?,
            StateType::Root => vec![state_hash],
        };
        
        Ok(proof)
    }

    fn generate_wallet_proof(&self, state_hash: [u8; 32]) -> Result<Vec<[u8; 32]>, SystemError> {
        let wallet_tree = self.wallet_tree.read();
        let mut proof = Vec::new();
        self.collect_proof_nodes(&wallet_tree, state_hash, &mut proof)?;
        Ok(proof)
    }

    fn generate_intermediate_proof(&self, state_hash: [u8; 32]) -> Result<Vec<[u8; 32]>, SystemError> {
        let intermediate_trees = self.intermediate_trees.read();
        let mut proof = Vec::new();
        
        for tree in intermediate_trees.values() {
            if self.collect_proof_nodes(tree, state_hash, &mut proof)? {
                break;
            }
        }
        
        Ok(proof)
    }

    fn collect_proof_nodes(
        &self,
        node: &MerkleNode,
        target_hash: [u8; 32],
        proof: &mut Vec<[u8; 32]>,
    ) -> Result<bool, SystemError> {
        if node.hash() == Some(target_hash) {
            proof.push(target_hash);
            return Ok(true);
        }

        if let Some(ref left) = node.left {
            if self.collect_proof_nodes(left, target_hash, proof)? {
                if let Some(hash) = node.hash() {
                    proof.push(hash);
                }
                return Ok(true);
            }
        }

        if let Some(ref right) = node.right {
            if self.collect_proof_nodes(right, target_hash, proof)? {
                if let Some(hash) = node.hash() {
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
        }
    }

    #[wasm_bindgen_test]
    async fn test_wallet_state_updates() {
        let manager = StateManager::new().unwrap();
        let wallet_id = [1u8; 32];
        let state = create_test_boc();
        
        let hash = manager.update_wallet_state(wallet_id, state).await.unwrap();
        assert!(manager.verify_state(hash));
        assert_eq!(manager.get_state_type(hash), Some(StateType::Wallet));
    }

    #[wasm_bindgen_test]
    async fn test_intermediate_state_updates() {
        let manager = StateManager::new().unwrap();
        let contract_id = [2u8; 32];
        let state = create_test_boc();
        
        let hash = manager.update_intermediate_state(contract_id, state).await.unwrap();
        assert!(manager.verify_state(hash));
        assert_eq!(manager.get_state_type(hash), Some(StateType::Intermediate));
    }

    #[wasm_bindgen_test]
    async fn test_root_state_updates() {
        let manager = StateManager::new().unwrap();
        let state = create_test_boc();
        
        let hash = manager.update_root_state(state).await.unwrap();
        assert!(manager.verify_state(hash));
        assert_eq!(manager.get_state_type(hash), Some(StateType::Root));
    }

    #[wasm_bindgen_test]
    async fn test_proof_generation() {
        let manager = StateManager::new().unwrap();
        let wallet_id = [3u8; 32];
        let state = create_test_boc();
        
        let hash = manager.update_wallet_state(wallet_id, state).await.unwrap();
        let proof = manager.generate_proof(hash).unwrap();
        assert!(!proof.is_empty());
    }

    #[wasm_bindgen_test]
    async fn test_metrics() {
        let manager = StateManager::new().unwrap();
        let state = create_test_boc();
        
        manager.update_wallet_state([1u8; 32], state.clone()).await.unwrap();
        manager.update_intermediate_state([2u8; 32], state.clone()).await.unwrap();
        manager.update_root_state(state).await.unwrap();
        
        let metrics = manager.get_metrics();
        assert_eq!(metrics.total_states, 3);
        assert_eq!(metrics.wallet_states, 1);
        assert_eq!(metrics.intermediate_states, 1);
    }
}