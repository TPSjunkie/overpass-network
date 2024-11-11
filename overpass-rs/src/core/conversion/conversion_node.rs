// ./src/types/node/conversion_node.rs

use crate::core::storage_node::storage_node::StorageNode;
use crate::core::types::ovp_types::OMResult;
use crate::SparseMerkleTree;
/// Conversion for Node to Sparse Merkle Tree.
impl<RootTree, IntermediateTreeManager> StorageNode<RootTree, IntermediateTreeManager> {
    pub fn to_sparse_merkle_tree(&self) -> OMResult<SparseMerkleTree> {
        let mut smt = SparseMerkleTree::new();
        let mut current_node = Some(self.clone());

        while let Some(node) = current_node {
            smt.nodes.push(node.clone());
            current_node = node.left.as_deref().cloned();
        }

        Ok(smt)
    }
}
