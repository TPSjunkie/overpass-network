use std::sync::Arc;
use parking_lot::RwLock;
use hashbrown::HashMap;
use serde::{Serialize, Deserialize};

use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::types::boc::BOC;

// State types for hierarchy
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateType {
    Wallet,         // Individual wallet states
    Intermediate,   // Intermediate contract states
    Root,          // Global root state
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

/// Sparse Merkle Tree node for state management
#[derive(Debug, Clone)]
pub struct MerkleNode {
    pub hash: [u8; 32],
    pub left: Option<Box<MerkleNode>>,
    pub right: Option<Box<MerkleNode>>,
    pub state_type: StateType,
    pub data: Option<BOC>,
}

impl MerkleNode {
    pub fn new(hash: [u8; 32], state_type: StateType) -> Self {
        Self {
            hash,
            left: None,
            right: None,
            state_type,
            data: None,
        }
    }

    pub fn new_with_data(hash: [u8; 32], state_type: StateType, data: BOC) -> Self {
        Self {
            hash,
            left: None,
            right: None,
            state_type,
            data: Some(data),
        }
    }
}

pub struct StateManager {
    // Hierarchical Merkle trees
    wallet_tree: RwLock<MerkleNode>,
    intermediate_trees: RwLock<HashMap<[u8; 32], MerkleNode>>,
    root_tree: RwLock<MerkleNode>,
    
    // State tracking
    state_hashes: RwLock<HashMap<[u8; 32], StateType>>,
    state_updates: RwLock<HashMap<[u8; 32], u64>>, // hash -> update count
    
    // Metrics
    metrics: RwLock<StateMetrics>,
}

impl StateManager {
    pub fn new() -> Result<Self, SystemError> {
        // Initialize empty Merkle trees for each level
        let wallet_root = MerkleNode::new([0u8; 32], StateType::Wallet);
        let intermediate_root = MerkleNode::new([0u8; 32], StateType::Intermediate);
        let root_tree = MerkleNode::new([0u8; 32], StateType::Root);

        Ok(Self {
            wallet_tree: RwLock::new(wallet_root),
            intermediate_trees: RwLock::new(HashMap::new()),
            root_tree: RwLock::new(root_tree),
            state_hashes: RwLock::new(HashMap::new()),
            state_updates: RwLock::new(HashMap::new()),
            metrics: RwLock::new(StateMetrics::default()),
        })
    }

    // Update wallet state
    pub async fn update_wallet_state(
        &self,
        wallet_id: [u8; 32],
        state: BOC,
    ) -> Result<[u8; 32], SystemError> {
        let mut wallet_tree = self.wallet_tree.write();
        let state_hash = self.compute_state_hash(&state);
        
        // Create new node
        let new_node = MerkleNode::new_with_data(state_hash, StateType::Wallet, state);
        
        // Update tree
        self.update_merkle_tree(&mut wallet_tree, wallet_id, new_node)?;
        
        // Track state
        self.state_hashes.write().insert(state_hash, StateType::Wallet);
        *self.state_updates.write().entry(state_hash).or_insert(0) += 1;
        
        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.wallet_states += 1;
        metrics.total_states += 1;
        metrics.state_updates += 1;
        
        Ok(state_hash)
    }

    // Update intermediate state
    pub async fn update_intermediate_state(
        &self,
        contract_id: [u8; 32],
        state: BOC,
    ) -> Result<[u8; 32], SystemError> {
        let mut intermediate_trees = self.intermediate_trees.write();
        let state_hash = self.compute_state_hash(&state);
        
        // Create new node
        let new_node = MerkleNode::new_with_data(state_hash, StateType::Intermediate, state);
        
        // Update or insert tree
        intermediate_trees.insert(contract_id, new_node);
        
        // Track state
        self.state_hashes.write().insert(state_hash, StateType::Intermediate);
        *self.state_updates.write().entry(state_hash).or_insert(0) += 1;
        
        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.intermediate_states += 1;
        metrics.total_states += 1;
        metrics.state_updates += 1;
        
        Ok(state_hash)
    }

    // Update root state
    pub async fn update_root_state(&self, state: BOC) -> Result<[u8; 32], SystemError> {
        let mut root_tree = self.root_tree.write();
        let state_hash = self.compute_state_hash(&state);
        
        // Create new node
        let new_node = MerkleNode::new_with_data(state_hash, StateType::Root, state);
        *root_tree = new_node;
        
        // Track state
        self.state_hashes.write().insert(state_hash, StateType::Root);
        *self.state_updates.write().entry(state_hash).or_insert(0) += 1;
        
        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.total_states += 1;
        metrics.state_updates += 1;
        
        Ok(state_hash)
    }

    // Update Merkle tree with new node
    fn update_merkle_tree(
        &self,
        tree: &mut MerkleNode,
        path: [u8; 32],
        node: MerkleNode,
    ) -> Result<(), SystemError> {
        let mut current = tree;
        
        // Traverse path to insert node
        for i in 0..256 {
            let bit = (path[i / 8] >> (7 - (i % 8))) & 1;
            
            if bit == 0 {
                if current.left.is_none() {
                    current.left = Some(Box::new(node.clone()));
                    break;
                }
                current = current.left.as_mut().unwrap();
            } else {
                if current.right.is_none() {
                    current.right = Some(Box::new(node.clone()));
                    break;
                }
                current = current.right.as_mut().unwrap();
            }
        }

        // Update metrics
        self.metrics.write().merkle_updates += 1;
        
        Ok(())
    }

    // Generate Merkle proof for state
    pub fn generate_proof(&self, state_hash: [u8; 32]) -> Result<Vec<[u8; 32]>, SystemError> {
        let state_type = self.state_hashes.read().get(&state_hash).ok_or(
            SystemError::new(SystemErrorType::StateNotFound, "State hash not found")
        )?;
        
        let proof = match state_type {
            StateType::Wallet => self.generate_wallet_proof(state_hash)?,
            StateType::Intermediate => self.generate_intermediate_proof(state_hash)?,
            StateType::Root => vec![state_hash],
        };
        
        Ok(proof)
    }

    // Generate proof for wallet state
    fn generate_wallet_proof(&self, state_hash: [u8; 32]) -> Result<Vec<[u8; 32]>, SystemError> {
        let wallet_tree = self.wallet_tree.read();
        let mut proof = Vec::new();
        self.collect_proof_nodes(&wallet_tree, state_hash, &mut proof)?;
        Ok(proof)
    }

    // Generate proof for intermediate state
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

    // Collect proof nodes
    fn collect_proof_nodes(
        &self,
        node: &MerkleNode,
        target_hash: [u8; 32],
        proof: &mut Vec<[u8; 32]>,
    ) -> Result<bool, SystemError> {
        if node.hash == target_hash {
            proof.push(node.hash);
            return Ok(true);
        }

        if let Some(left) = &node.left {
            if self.collect_proof_nodes(left, target_hash, proof)? {
                proof.push(node.hash);
                return Ok(true);
            }
        }

        if let Some(right) = &node.right {
            if self.collect_proof_nodes(right, target_hash, proof)? {
                proof.push(node.hash);
                return Ok(true);
            }
        }

        Ok(false)
    }

    // Verify state exists
    pub fn verify_state(&self, state_hash: [u8; 32]) -> bool {
        self.state_hashes.read().contains_key(&state_hash)
    }

    // Get state type
    pub fn get_state_type(&self, state_hash: [u8; 32]) -> Option<StateType> {
        self.state_hashes.read().get(&state_hash).cloned()
    }

    // Get state metrics
    pub fn get_metrics(&self) -> StateMetrics {
        self.metrics.read().clone()
    }

    // Compute state hash
    fn compute_state_hash(&self, state: &BOC) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&state.serialize());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
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