use crate::core::hierarchy::sparse_merkle_tree_wasm::SparseMerkleTreeWasm;
use crate::core::storage_node::storage_node::StorageNode;
use crate::core::types::ovp_types::SystemError;
use crate::core::types::ovp_types::{WalletRootState, WalletStateUpdate};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

// Define behavior required for Merkle trees
pub trait MerkleTreeBehavior {
    fn get_managed_states(&self) -> Vec<[u8; 32]>;
    fn get_merkle_tree(&self) -> &SparseMerkleTreeWasm;
    fn root(&self) -> [u8; 32];
}

// Type alias for the complex StorageNode type with all needed generic parameters
type StorageNodeType = StorageNode<WalletStateUpdate, WalletRootState>;

pub struct ConsistencyChecker {
    storage_nodes: Vec<StorageNodeType>,
    visited_cells: HashSet<[u8; 32]>,
    max_depth: usize,
}
impl ConsistencyChecker {
    pub fn new(nodes: Vec<StorageNodeType>) -> Self {
        Self {
            storage_nodes: nodes,
            visited_cells: HashSet::new(),
            max_depth: 1024,
        }
    }

    pub fn verify_global_consistency(&mut self) -> Result<(), SystemError> {
        self.verify_redundancy_factor()?;
        self.verify_boc_transitions()?;
        self.verify_merkle_paths()?;
        self.verify_node_overlap()?;
        Ok(())
    }

    fn verify_redundancy_factor(&self) -> Result<(), SystemError> {
        let mut state_coverage = HashSet::new();

        for node in &self.storage_nodes {
            for state in node.get_managed_states() {
                state_coverage.insert(state);
            }
        }

        const MIN_REDUNDANCY: usize = 3;
        for state in state_coverage {
            let copies = self
                .storage_nodes
                .iter()
                .filter(|node| node.manages_state(&state))
                .count();

            if copies < MIN_REDUNDANCY {
                return Err(SystemError::InsufficientRedundancy {
                    state_id: state,
                    copies,
                    required: MIN_REDUNDANCY,
                });
            }
        }

        Ok(())
    }

    fn verify_boc_transitions(&mut self) -> Result<(), SystemError> {
        for node in &self.storage_nodes {
            let bocs = node.get_boc_history();

            for i in 1..bocs.len() {
                let prev_state = bocs[i - 1].get_state();
                let next_state = bocs[i].get_state();

                if !self.is_valid_transition(&prev_state, &next_state) {
                    return Err(SystemError::InvalidStateTransition {
                        from: prev_state,
                        to: next_state,
                        node_id: node.id(),
                    });
                }
            }
        }
        Ok(())
    }

    fn verify_merkle_paths(&self) -> Result<(), SystemError> {
        for node in &self.storage_nodes {
            let smt = node.get_merkle_tree();

            for (leaf, path) in smt.get_all_paths() {
                if !self.verify_merkle_path(&leaf, &path, smt.root()) {
                    return Err(SystemError::InvalidMerklePath {
                        leaf,
                        root: smt.root(),
                        node_id: node.id(),
                    });
                }
            }
        }
        Ok(())
    }

    fn verify_node_overlap(&self) -> Result<(), SystemError> {
        for (i, node1) in self.storage_nodes.iter().enumerate() {
            for node2 in self.storage_nodes.iter().skip(i + 1) {
                let overlap = self.calculate_overlap(node1, node2);

                const MIN_OVERLAP: f64 = 0.3;
                if overlap < MIN_OVERLAP {
                    return Err(SystemError::InsufficientNodeOverlap {
                        node1_id: node1.id(),
                        node2_id: node2.id(),
                        overlap,
                        required: MIN_OVERLAP,
                    });
                }
            }
        }
        Ok(())
    }

    fn is_valid_transition(&self, prev: &[u8], next: &[u8]) -> bool {
        if prev.len() != 32 || next.len() != 32 {
            return false;
        }

        let mut hasher = Sha256::new();
        hasher.update(prev);
        let expected_next = hasher.finalize();

        next == expected_next.as_slice()
    }

    fn verify_merkle_path(&self, leaf: &[u8], path: &[(Vec<u8>, bool)], root: &[u8]) -> bool {
        let mut current = leaf.to_vec();

        for (sibling, is_right) in path {
            let mut hasher = Sha256::new();
            if *is_right {
                hasher.update(sibling);
                hasher.update(current);
            } else {
                hasher.update(current);
                hasher.update(sibling);
            }
            current = hasher.finalize().to_vec();
        }

        current.as_slice() == root
    }

    fn calculate_overlap(&self, node1: &StorageNodeType, node2: &StorageNodeType) -> f64 {
        let states1: HashSet<_> = node1.get_managed_states().into_iter().collect();
        let states2: HashSet<_> = node2.get_managed_states().into_iter().collect();

        let intersection = states1.intersection(&states2).count();
        let union = states1.union(&states2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{hierarchy::IntermediateContract, state::boc::cell_serialization::BOC};

    fn create_test_storage_node(id: [u8; 32]) -> StorageNodeType {
        StorageNode::new(
            id,                            // node_id
            WalletRootState::new(),        // root_tree
            IntermediateContract::new(id), // intermediate_contract
            SparseMerkleTreeWasm::new(),   // storage_tree
            100,                           // battery_capacity
            vec![],                        // initial_peers
        )
    }

    #[test]
    fn test_redundancy_verification() {
        let mut nodes = Vec::new();
        let state1 = [1u8; 32];
        let state2 = [2u8; 32];

        // Create nodes with overlapping states
        for i in 0..3 {
            let mut node = create_test_storage_node([i as u8; 32]);
            node.add_state(state1);
            node.add_state(state2);
            nodes.push(node);
        }

        let checker = ConsistencyChecker::new(nodes);
        assert!(checker.verify_redundancy_factor().is_ok());
    }

    #[test]
    fn test_boc_transition_verification() {
        let mut node = create_test_storage_node([0u8; 32]);

        // Create valid BOC chain
        let mut prev_state = [0u8; 32];
        for _ in 0..5 {
            let mut hasher = Sha256::new();
            hasher.update(&prev_state);
            let next_state = hasher.finalize();

            let boc = BOC::new(prev_state.to_vec(), next_state.to_vec());
            node.add_boc(boc);

            prev_state.copy_from_slice(&next_state);
        }

        let checker = ConsistencyChecker::new(vec![node]);
        assert!(checker.verify_boc_transitions().is_ok());
    }
    #[test]
    fn test_merkle_path_verification() {
        let leaf = vec![1u8; 32];
        let sibling = vec![2u8; 32];

        let mut hasher = Sha256::new();
        hasher.update(&leaf);
        hasher.update(&sibling);
        let root = hasher.finalize();

        let path = vec![(sibling, false)];

        let checker = ConsistencyChecker::new(vec![]);
        assert!(checker.verify_merkle_path(&leaf, &path, &root));
    }

    #[test]
    fn test_node_overlap_verification() {
        let mut node1 = create_test_storage_node([1u8; 32]);
        let mut node2 = create_test_storage_node([2u8; 32]);

        // Create overlapping states
        let shared_states: Vec<[u8; 32]> = (0..5).map(|i| [i as u8; 32]).collect();

        for state in &shared_states {
            node1.add_state(*state);
            node2.add_state(*state);
        }

        // Add unique states to each node
        for i in 5..8 {
            node1.add_state([i as u8; 32]);
        }

        for i in 8..11 {
            node2.add_state([i as u8; 32]);
        }

        let checker = ConsistencyChecker::new(vec![node1, node2]);
        assert!(checker.verify_node_overlap().is_ok());
    }
}
