// ./src/common/types/dag_boc.rs

/// DAG BOC representation
/// Represents a DAG BOC with a Merkle tree root
/// Contains the root hash and the data of the DAG BOC
/// Used for storing and retrieving DAG data
/// Implements the DagBOC trait
/// Contains methods for serializing and deserializing the DAG BOC
use crate::common::error::client_errors::SystemError;
use crate::common::error::client_errors::SystemErrorType;
use crate::common::types::ops::OpCode;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::collections::HashMap;

/// Channel Cell
/// Represents a channel cell with a balance and nonce
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelCell {
    pub balance: u64,
    pub nonce: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DAGBOC {
    pub dag_cells: Vec<Vec<u8>>,
    pub references: Vec<Vec<u8>>,
    pub roots: Vec<Vec<u8>>,
    pub hash: Option<[u8; 32]>,
    pub state_mapping: HashMap<Vec<u8>, u32>,
}

impl DAGBOC {
    /// Create a new DAG BOC
    pub fn new() -> Self {
        DAGBOC {
            dag_cells: Vec::new(),
            references: Vec::new(),
            roots: Vec::new(),
            hash: None,
            state_mapping: HashMap::new(),
        }
    }

    pub fn add_cell(&mut self, cell: Cell<Vec<u8>>) -> Result<u32, SystemError> {
        let id = self.dag_cells.len() as u32;
        self.dag_cells.push(cell.into_inner());
        Ok(id)
    }

    pub fn update_state_mapping(&mut self, key: Vec<u8>, value: u32) -> Result<(), SystemError> {
        self.state_mapping.insert(key, value);
        Ok(())
    }

    pub fn process_op_code(&mut self, op_code: OpCode) -> Result<(), SystemError> {
        match op_code {
            OpCode::Add { cell } => {
                self.add_cell(Cell::new(cell))?;
            }
            OpCode::SetCode {
                code,
                new_code,
                new_data: _,
                new_libraries: _,
                new_version: _,
            } => {
                // Implement code update logic here
                if let Some(index) = self.dag_cells.iter().position(|c| c == &code) {
                    self.dag_cells[index] = new_code;
                    // You might want to update other fields as well
                }
            }
            OpCode::SetData { cell, new_data } => {
                if let Some(index) = self.dag_cells.iter().position(|c| c == &cell) {
                    self.dag_cells[index] = new_data;
                }
            }
            OpCode::SetLibraries {
                cell,
                new_libraries,
            } => {
                if let Some(index) = self.dag_cells.iter().position(|c| c == &cell) {
                    self.dag_cells[index] = new_libraries;
                }
            }
            OpCode::SetVersion { cell, new_version } => {
                if let Some(index) = self.dag_cells.iter().position(|c| c == &cell) {
                    self.dag_cells[index] = new_version.to_le_bytes().to_vec();
                }
            }
            OpCode::AddReference { from, to } => {
                self.references
                    .push(vec![from.to_le_bytes().to_vec(), to.to_le_bytes().to_vec()].concat());
            }
            OpCode::SetRoot { index } => {
                if let Some(cell) = self.dag_cells.get(index as usize) {
                    self.roots.push(cell.clone());
                }
            }
            OpCode::Remove { cell } => {
                if let Some(index) = self.dag_cells.iter().position(|c| c == &cell) {
                    self.dag_cells.remove(index);
                }
            }
            OpCode::Update { cell } => {
                if let Some(index) = self.dag_cells.iter().position(|c| c == &cell) {
                    self.dag_cells[index] = cell;
                }
            }
            OpCode::RemoveReference { from, to } => {
                if let Some(index) = self.references.iter().position(|r| {
                    let from_ref = u32::from_le_bytes(r[0..4].try_into().unwrap());
                    let to_ref = u32::from_le_bytes(r[4..8].try_into().unwrap());
                    from_ref == from && to_ref == to
                }) {
                    self.references.remove(index);
                }
            }
            _ => {
                return Err(SystemError::new(
                    SystemErrorType::InvalidOperation,
                    "Unsupported operation".to_string(),
                ));
            }
        }
        Ok(())
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

        for cell in &self.dag_cells {
            hasher.update(cell);
        }

        for reference in &self.references {
            hasher.update(reference);
        }

        for root in &self.roots {
            hasher.update(root);
        }

        hasher.finalize().into()
    }

    pub fn with_dag_cells(mut self, dag_cells: Vec<Vec<u8>>) -> Self {
        self.dag_cells = dag_cells;
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
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dag_boc_new() {
        let dag_boc = DAGBOC::new();
        assert!(dag_boc.dag_cells.is_empty());
        assert!(dag_boc.references.is_empty());
        assert!(dag_boc.roots.is_empty());
        assert!(dag_boc.hash.is_none());
    }

    #[test]
    fn test_dag_boc_add_cell() {
        let mut dag_boc = DAGBOC::new();
        let cell = Cell::new(vec![1, 2, 3]);
        let id = dag_boc.add_cell(cell).unwrap();
        assert_eq!(dag_boc.dag_cells.len(), 1);
        assert_eq!(dag_boc.dag_cells[0], vec![1, 2, 3]);
        assert_eq!(id, 0);
    }

    #[test]
    fn test_dag_boc_update_state_mapping() {
        let mut dag_boc = DAGBOC::new();
        let key = "key".as_bytes().to_vec();
        let value = 0;
        dag_boc.update_state_mapping(key.clone(), value).unwrap();
        assert_eq!(dag_boc.state_mapping.get(&key).unwrap(), &value);
    }

    #[test]
    fn test_dag_boc_process_op_code() {
        let mut dag_boc = DAGBOC::new();
        let op_code = OpCode::Add {
            cell: vec![1, 2, 3],
        };
        dag_boc.process_op_code(op_code).unwrap();
        assert_eq!(dag_boc.dag_cells.len(), 1);
        assert_eq!(dag_boc.dag_cells[0], vec![1, 2, 3]);
    }

    #[test]
    fn test_dag_boc_serialization() {
        let dag_boc = DAGBOC::new()
            .with_dag_cells(vec![vec![1, 2, 3]])
            .with_references(vec![vec![4, 5, 6]])
            .with_roots(vec![vec![7, 8, 9]]);

        let serialized = dag_boc.serialize().unwrap();
        let deserialized = DAGBOC::deserialize(&serialized).unwrap();

        assert_eq!(dag_boc, deserialized);
    }

    #[test]
    fn test_compute_hash() {
        let dag_boc = DAGBOC::new()
            .with_dag_cells(vec![vec![1, 2, 3]])
            .with_references(vec![vec![4, 5, 6]])
            .with_roots(vec![vec![7, 8, 9]]);

        let hash = dag_boc.compute_hash();
        assert_ne!(hash, [0u8; 32]);
    }
}
