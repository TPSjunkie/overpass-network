// ./src/types/tree/conversion_tree.rs
use crate::core::types::ovp_types::OMResult;
use crate::SparseMerkleTree;

/// Conversion for Sparse Merkle Tree to Node.
impl SparseMerkleTree {
    pub fn to_node(&self) -> OMResult<SparseMerkleTree> {
        let mut nodes = Vec::new();
        let mut current_node = self.clone();

        while !current_node.nodes.is_empty() {
            nodes.push(current_node.clone());
            current_node = current_node.nodes.pop().unwrap();
        }

        Ok(nodes.pop().unwrap())
    }
}
