// ./src/types/zkp/conversion_zkp.rs

use crate::core::types::ovp_types::*;
use crate::SparseMerkleTree;

/// Implementation of SparseMerkleTree for the hierarchical structure.
/// Converts the Sparse Merkle Tree zk-SNARK Proof into a privacy-preserving BOC format
/// that encapsulates the root of an intermediate contract (IC).
impl SparseMerkleTree {
    /// Converts the Sparse Merkle Tree zk-SNARK Proof into a BOC (Bag of Cells) format.
    /// Each IC maintains a private SMT, represented by a root, which is the only public data
    /// submitted to the root contract.
    pub fn to_boc(&self, ic_root: &IntermediateContractRoot) -> OMResult<Vec<u8>> {
        let mut boc = Vec::new();

        // Serialize all the node hashes in the proof path for Merkle consistency verification
        for node in &self.nodes {
            boc.extend_from_slice(&node.hash_node()); // Node hash
            boc.push(node.key.len() as u8); // Key length
            boc.push(node.value.len() as u8); // Value length
        }

        // Serialize the IC's root hash securely (to be added to the root contract)
        boc.extend_from_slice(&ic_root.hash); // Intermediate contract root hash

        // Finalize BOC by appending the calculated root hash of the entire path
        boc.extend_from_slice(&self.root_hash); // Final proof root hash for validation

        Ok(boc)
    }
}

/// A data structure for the root of an Intermediate Contract (IC) in the hierarchical structure.
/// This is the only data exposed from the IC's private SMT to the root contract.
#[derive(Clone, Debug)]
pub struct IntermediateContractRoot {
    pub hash: [u8; 32], // Merkle root hash of the IC's managed channels
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SparseMerkleTree;

    #[test]
    fn test_to_boc() {
        let nodes = vec![
            SparseMerkleTree::new("key1".as_bytes(), "value1".as_bytes()),
            SparseMerkleTree::new("key2".as_bytes(), "value2".as_bytes()),
        ];
        let smt = SparseMerkleTree::new(nodes);
        let ic_root = IntermediateContractRoot { hash: [0xAA; 32] }; // Sample IC root hash

        let boc = smt.to_boc(&ic_root).expect("Failed to create BOC");
        assert_eq!(boc.len(), 134); // Expected BOC size based on test values

        // Validate serialized structure
        assert_eq!(&boc[0..32], &smt.nodes[0].hash_node());
        assert_eq!(&boc[34..66], &smt.nodes[1].hash_node());
        assert_eq!(&boc[68..100], &ic_root.hash);
        assert_eq!(&boc[100..132], &smt.root_hash);
    }
}
