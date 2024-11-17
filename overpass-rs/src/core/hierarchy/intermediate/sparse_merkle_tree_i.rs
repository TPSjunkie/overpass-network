use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::types::boc::{Cell, CellType, BOC};
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::iop::target::Target;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2_field::types::Field;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Intermediate Tree Trait
pub trait IntermediateTreeManagerTrait {
    fn update_trees(
        &mut self,
        boc: &BOC,
        intermediate_trees: &mut HashMap<[u8; 32], SparseMerkleTreeI>,
        root_trees: &mut HashMap<[u8; 32], SparseMerkleTreeI>,
    ) -> Result<(), SystemError>;
}

type VirtualCell = Target;

/// Merkle Tree Node
pub struct MerkleNode {
    pub left: Option<[u8; 32]>,
    pub right: Option<[u8; 32]>,
    pub hash: Option<[u8; 32]>,
    pub virtual_cell: Option<VirtualCell>,
    pub value: Option<[u8; 32]>,
    pub is_leaf: bool,
    pub is_virtual: bool,
    pub is_empty: bool,
}

/// Sparse Merkle Tree Implementation
pub struct SparseMerkleTreeI {
    circuit_builder: CircuitBuilder<GoldilocksField, 2>,
    root_hash: [u8; 32],
    nodes: HashMap<[u8; 32], MerkleNode>,
    height: usize,
    virtual_cells: HashMap<VirtualCell, MerkleNode>,
    virtual_cell_count: usize,
    current_virtual_cell: VirtualCell,
    current_virtual_cell_count: usize,
}

impl SparseMerkleTreeI {
    /// Create a new Sparse Merkle Tree
    pub fn new() -> Self {
        let config = CircuitConfig::standard_recursion_config();
        let circuit_builder = CircuitBuilder::new(config);

        Self {
            circuit_builder,
            root_hash: [0u8; 32],
            nodes: HashMap::new(),
            height: 256,
            virtual_cells: HashMap::new(),
            virtual_cell_count: 0,
            current_virtual_cell: Target::wire(0, 0),
            current_virtual_cell_count: 0,
        }
    }

    /// Update a leaf in the Merkle tree
    pub fn update(&mut self, key: &[u8], value: &[u8]) -> Result<(), SystemError> {
        let leaf_hash = self.hash_leaf(key, value);
        let path = self.generate_merkle_path(key)?;
        let _value_field = self.hash_to_field(&leaf_hash);
        let _value_cell = self.circuit_builder.add_virtual_public_input();
        let _key_field = self.hash_to_field(&self.hash_leaf(key, &[]));
        let _key_cell = self.circuit_builder.add_virtual_public_input();
        self.add_path_constraints(&path, _key_cell, _value_cell)?;
        self.root_hash = self.calculate_new_root(&path, &leaf_hash)?;
        Ok(())
    }

    pub fn add_virtual_public_input(&mut self) -> Target {
        self.circuit_builder.add_virtual_public_input()
    }

    /// Add constraints to the path in the zk-SNARK circuit
    fn add_path_constraints(
        &mut self,
        path: &[([u8; 32], bool)],
        _key_cell: Target,
        _value_cell: Target,
    ) -> Result<(), SystemError> {
        let mut current = self.circuit_builder.add_virtual_target();

        for (sibling, is_left) in path {
            let _sibling_field = self.hash_to_field(sibling);
            let sibling_cell = self.circuit_builder.add_virtual_public_input();

            let _cells = if *is_left {
                [current, sibling_cell]
            } else {
                [sibling_cell, current]
            };
            current = self.circuit_builder.add_virtual_target();
        }

        let root_cell = self.circuit_builder.add_virtual_public_input();
        let is_equal = self.circuit_builder.is_equal(current, root_cell);
        let bool_target = self.circuit_builder.add_virtual_target();
        self.circuit_builder.connect(is_equal.target, bool_target);

        Ok(())
    }

    /// Generate Merkle path for a given key
    fn generate_merkle_path(&self, key: &[u8]) -> Result<Vec<([u8; 32], bool)>, SystemError> {
        let mut path = Vec::new();
        let mut current = self.root_hash;

        for i in 0..self.height {
            let bit = self.get_bit(key, i);
            let node = self.nodes.get(&current).ok_or(SystemError {
                error_type: SystemErrorType::NotFound,
                message: "Node not found in path".to_string(),
            })?;

            if bit {
                let right_hash = node.right.ok_or_else(|| SystemError {
                    error_type: SystemErrorType::NotFound,
                    message: "Invalid path".to_string(),
                })?;
                path.push((right_hash, true));
                current = right_hash;
            } else {
                let left_hash = node.left.ok_or_else(|| SystemError {
                    error_type: SystemErrorType::NotFound,
                    message: "Invalid path".to_string(),
                })?;
                path.push((left_hash, false));
                current = left_hash;
            }
        }

        Ok(path)
    }

    /// Hash a leaf node
    fn hash_leaf(&self, key: &[u8], value: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(key);
        hasher.update(value);
        hasher.finalize().into()
    }

    /// Calculate the new root hash after updating a leaf
    fn calculate_new_root(
        &self,
        path: &[([u8; 32], bool)],
        leaf_hash: &[u8; 32],
    ) -> Result<[u8; 32], SystemError> {
        let mut current = *leaf_hash;

        for (sibling, is_left) in path.iter().rev() {
            let mut hasher = Sha256::new();
            if *is_left {
                hasher.update(current);
                hasher.update(sibling);
            } else {
                hasher.update(sibling);
                hasher.update(current);
            }
            current = hasher.finalize().into();
        }

        Ok(current)
    }

    /// Extract a bit from the key at a specific index
    fn get_bit(&self, key: &[u8], index: usize) -> bool {
        let byte_index = index / 8;
        let bit_index = 7 - (index % 8);
        if byte_index < key.len() {
            (key[byte_index] >> bit_index) & 1 == 1
        } else {
            false
        }
    }

    /// Convert hash to a field element
    fn hash_to_field(&self, bytes: &[u8; 32]) -> GoldilocksField {
        let mut array = [0u8; 8];
        array.copy_from_slice(&bytes[0..8]);
        let num = u64::from_le_bytes(array);
        GoldilocksField::from_canonical_u64(num)
    }

    /// Return the current root hash of the tree
    pub fn root(&self) -> [u8; 32] {
        self.root_hash
    }

    /// Serialize the tree state to a BOC format
    pub fn serialize_state(&self) -> Result<BOC, SystemError> {
        let mut boc = BOC::new();
        let root_cell = Cell::new(vec![], vec![], CellType::Ordinary, self.root_hash, None);
        let mut node_cells = Vec::new();

        for (hash, node) in &self.nodes {
            let mut node_data = Vec::new();
            if let Some(left) = node.left {
                node_data.extend_from_slice(&left);
            }
            if let Some(right) = node.right {
                node_data.extend_from_slice(&right);
            }
            let cell = Cell::new(
                node_data,
                vec![],
                CellType::Ordinary,
                *hash,
                None,
            );
            node_cells.push(cell);
        }

        Ok(boc)
    }
}