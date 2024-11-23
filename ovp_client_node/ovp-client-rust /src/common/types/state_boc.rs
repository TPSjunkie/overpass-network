use crate::common::error::client_errors::{SystemError, SystemErrorType};
use serde::{Deserialize, Serialize};
use serde::ser::SerializeStruct;
use sha2::{Digest, Sha256};

/// Represents a state init object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateInit {
    /// The code of the contract.
    pub code: Option<Vec<u8>>,
    /// The data of the contract.
    pub data: Option<Vec<u8>>,
    /// The library of the contract.
    pub library: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq)]
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

    /// state_init 
    pub fn set_state_init(&mut self, state_init: StateInit) {
        self.state_init = state_init;
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

    pub fn serialize_to_vec(&self) -> Result<Vec<u8>, SystemError> {
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

impl Serialize for STATEBOC {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("STATEBOC", 4)?;
        state.serialize_field("state_cells", &self.state_cells)?;
        state.serialize_field("references", &self.references)?;
        state.serialize_field("roots", &self.roots)?;
        state.serialize_field("hash", &self.hash)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for STATEBOC {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StateBocHelper {
            state_cells: Vec<Vec<u8>>,
            references: Vec<Vec<u8>>,
            roots: Vec<Vec<u8>>,
            hash: Option<[u8; 32]>,
        }

        let helper = StateBocHelper::deserialize(deserializer)?;
        
        Ok(STATEBOC {
            state_cells: helper.state_cells,
            references: helper.references,
            roots: helper.roots,
            hash: helper.hash,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Cell;

impl Cell {
    pub fn new(
        _get_data: Vec<u8>,
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
    }
}