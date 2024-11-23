// ./src/common/types/state_state_boc.rs

/// State STATEBOC representation
/// Represents a STATEBOC with a Merkle tree root
/// Contains the root hash and the data of the STATEBOC
/// Used for storing and retrieving state data
/// Implements the StateSTATEBOC trait
/// Contains methods for serializing and deserializing the STATEBOC
use crate::common::error::client_errors::{SystemError, SystemErrorType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct STATEBOC {
    pub state_cells: Vec<Vec<u8>>,
    pub references: Vec<Vec<u8>>,
    pub roots: Vec<Vec<u8>>,
    pub hash: Option<[u8; 32]>,
}

impl STATEBOC {
    pub fn new() -> Self {
        Self {
            state_cells: Vec::new(),
            references: Vec::new(),
            roots: Vec::new(),
            hash: None,
        }
    }

    pub fn add_cell(&mut self, _cell: Cell) {}

    pub fn add_root(&mut self, _index: usize) {}

    pub fn with_hash(mut self, hash: [u8; 32]) -> Self {
        self.hash = Some(hash);
        self
    }

    pub fn with_state_cells(mut self, state_cells: Vec<Vec<u8>>) -> Self {
        self.state_cells = state_cells;
        self
    }

    pub fn with_references(mut self, references: Vec<Vec<u8>>) -> Self {
        self.references = references;
        self
    }

    pub fn with_roots(mut self, roots: Vec<Vec<u8>>) -> Self {
        self.roots = roots;
        self
    }

    pub fn hash(&self) -> [u8; 32] {
        self.hash.unwrap_or_else(|| [0u8; 32])
    }

    pub fn state_cells(&self) -> &Vec<Vec<u8>> {
        &self.state_cells
    }

    pub fn references(&self) -> &Vec<Vec<u8>> {
        &self.references
    }

    pub fn roots(&self) -> &Vec<Vec<u8>> {
        &self.roots
    }

    pub fn set_hash(&mut self, hash: [u8; 32]) {
        self.hash = Some(hash);
    }

    pub fn set_state_cells(&mut self, state_cells: Vec<Vec<u8>>) {
        self.state_cells = state_cells;
    }

    pub fn set_references(&mut self, references: Vec<Vec<u8>>) {
        self.references = references;
    }

    pub fn set_roots(&mut self, roots: Vec<Vec<u8>>) {
        self.roots = roots;
    }

    pub fn serialize(&self) -> Result<Vec<u8>, SystemError> {
        bincode::serialize(self)
            .map_err(|e| SystemError::new(SystemErrorType::SerializationError, e.to_string()))
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, SystemError> {
        bincode::deserialize(data)
            .map_err(|e| SystemError::new(SystemErrorType::SerializationError, e.to_string()))
    }

    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();

        for state_cell in &self.state_cells {
            hasher.update(state_cell);
        }

        for reference in &self.references {
            hasher.update(reference);
        }

        for root in &self.roots {
            hasher.update(root);
        }

        hasher.finalize().into()
    }
}

impl Default for STATEBOC {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Cell;

impl Cell {
    pub fn new(
        _data: Vec<u8>,
        _references: Vec<usize>,
        _cell_type: CellType,
        _merkle_hash: [u8; 32],
        _proof: Option<Vec<u8>>,
    ) -> Self {
        Cell
    }
}

#[derive(Debug, Clone)]
pub enum CellType {
    Ordinary,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_boc_new() {
        let state_boc = STATEBOC::new();
        assert!(state_boc.state_cells.is_empty());
        assert!(state_boc.references.is_empty());
        assert!(state_boc.roots.is_empty());
        assert!(state_boc.hash.is_none());
    }

    #[test]
    fn test_state_boc_builder() {
        let state_cells = vec![vec![1, 2, 3]];
        let references = vec![vec![4, 5, 6]];
        let roots = vec![vec![7, 8, 9]];
        let hash = [0u8; 32];

        let state_boc = STATEBOC::new()
            .with_state_cells(state_cells.clone())
            .with_references(references.clone())
            .with_roots(roots.clone())
            .with_hash(hash);

        assert_eq!(state_boc.state_cells(), &state_cells);
        assert_eq!(state_boc.references(), &references);
        assert_eq!(state_boc.roots(), &roots);
        assert_eq!(state_boc.hash(), hash);
    }

    #[test]
    fn test_state_boc_setter() {
        let mut state_boc = STATEBOC::new();
        let state_cells = vec![vec![1, 2, 3]];
        let references = vec![vec![4, 5, 6]];
        let roots = vec![vec![7, 8, 9]];
        let hash = [0u8; 32];

        state_boc.set_state_cells(state_cells.clone());
        state_boc.set_references(references.clone());
        state_boc.set_roots(roots.clone());
        state_boc.set_hash(hash);

        assert_eq!(state_boc.state_cells(), &state_cells);
        assert_eq!(state_boc.references(), &references);
        assert_eq!(state_boc.roots(), &roots);
        assert_eq!(state_boc.hash(), hash);
    }

    #[test]
    fn test_state_boc_serialization() {
        let state_boc = STATEBOC::new()
            .with_state_cells(vec![vec![1, 2, 3]])
            .with_references(vec![vec![4, 5, 6]])
            .with_roots(vec![vec![7, 8, 9]]);

        let serialized = state_boc.serialize().unwrap();
        let deserialized = STATEBOC::deserialize(&serialized).unwrap();

        assert_eq!(state_boc, deserialized);
    }

    #[test]
    fn test_compute_hash() {
        let state_boc = STATEBOC::new()
            .with_state_cells(vec![vec![1, 2, 3]])
            .with_references(vec![vec![4, 5, 6]])
            .with_roots(vec![vec![7, 8, 9]]);

        let hash = state_boc.compute_hash();
        assert_ne!(hash, [0u8; 32]);
    }
}
