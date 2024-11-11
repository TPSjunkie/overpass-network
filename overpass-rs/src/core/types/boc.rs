use crate::core::error::errors::{BocError, SystemError, SystemErrorType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub data: Vec<u8>,
    pub references: Vec<usize>,
    pub cell_type: CellType,
    pub merkle_hash: [u8; 32],
    pub proof: Option<Vec<u8>>, // Added field for proof data
}

// root_cell struct
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RootCell {
    pub capacity: u64,
    pub lock_script: Script,
    pub type_script: Script,
    pub data: Vec<u8>,
}

// script struct
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub code_hash: H256,
    pub hash_type: ScriptHashType,
    pub args: Vec<u8>,
    // pub hash_type: ScriptHashType,
}

// script_hash_type enum
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ScriptHashType {
    Data,
    Type,
}

// h256 struct
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct H256(pub [u8; 32]);

pub fn repr_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}
// Hash

pub fn hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

// hash_pair
pub fn hash_pair(left: &[u8], right: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

// hash_value
pub fn hash_value(value: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(value);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

// root_hash
pub fn root_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

// root cell hash
pub fn root_cell_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

// get_references function
pub fn get_refs(cells: &[Cell]) -> Vec<usize> {
    let mut references = Vec::new();
    for (index, cell) in cells.iter().enumerate() {
        for reference in cell.references.iter() {
            if !references.contains(reference) {
                references.push(*reference);
            }
        }
    }
    references
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    Ordinary,
    PrunedBranch,
    LibraryReference,
    MerkleProof,
    MerkleUpdate,
}

impl Cell {
    /// Creates a new `Cell` with all fields specified.
    pub fn new(
        data: Vec<u8>,
        references: Vec<usize>,
        cell_type: CellType,
        merkle_hash: [u8; 32],
        proof: Option<Vec<u8>>,
    ) -> Self {
        Self {
            data,
            references,
            cell_type,
            merkle_hash,
            proof,
        }
    }

    /// Creates a new `Cell` with only data specified. Other fields are set to default values.
    pub fn with_data(data: Vec<u8>) -> Self {
        Self {
            data,
            references: Vec::new(),
            cell_type: CellType::Ordinary,
            merkle_hash: [0u8; 32],
            proof: None,
        }
    }

    /// Creates a new `Cell` from data.
    pub fn from_data(data: Vec<u8>) -> Self {
        Self::with_data(data)
    }

    /// Returns a reference to the cell's data.
    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    /// Calculates and updates the Merkle hash of the cell.
    pub fn update_merkle_hash(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(&self.data);
        for &ref_idx in &self.references {
            hasher.update(ref_idx.to_le_bytes());
        }
        self.merkle_hash = hasher.finalize().into();
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            references: Vec::new(),
            cell_type: CellType::Ordinary,
            merkle_hash: [0u8; 32],
            proof: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BOC {
    pub cells: Vec<Cell>,
    pub roots: Vec<usize>,
}

impl BOC {
    /// Creates a new, empty `BOC`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a `Cell` to the `BOC` and returns its index.
    pub fn add_cell(&mut self, cell: Cell) -> usize {
        let index = self.cells.len();
        self.cells.push(cell);
        index
    }

    /// Adds a root index to the `BOC`.
    pub fn add_root(&mut self, index: usize) {
        self.roots.push(index);
    }

    /// Retrieves a reference to a `Cell` by index.
    pub fn get_cell(&self, index: usize) -> Option<&Cell> {
        self.cells.get(index)
    }

    /// Retrieves a mutable reference to a `Cell` by index.
    pub fn get_cell_mut(&mut self, index: usize) -> Option<&mut Cell> {
        self.cells.get_mut(index)
    }

    /// Retrieves a reference to the root `Cell`, if any.
    pub fn get_root_cell(&self) -> Option<&Cell> {
        self.roots
            .first()
            .and_then(|&root_idx| self.get_cell(root_idx))
    }

    /// Retrieves a mutable reference to the root `Cell`, if any.
    pub fn get_root_cell_mut(&mut self) -> Option<&mut Cell> {
        if let Some(&root_idx) = self.roots.first() {
            self.get_cell_mut(root_idx)
        } else {
            None
        }
    }

    /// Returns the number of cells in the `BOC`.
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Returns the number of roots in the `BOC`.
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    /// Checks if the `BOC` is empty.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Clears all cells and roots from the `BOC`.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.roots.clear();
    }

    /// Sets the data for the root cell.
    pub fn set_data(&mut self, data: &[u8]) -> Result<(), SystemError> {
        if let Some(root_cell) = self.get_root_cell_mut() {
            root_cell.data = data.to_vec();
            root_cell.update_merkle_hash();
            Ok(())
        } else {
            Err(SystemError::new(
                SystemErrorType::NoRootCell,
                "No root cell defined".to_string(),
            ))
        }
    }

    /// Sets the Merkle root hash for the root cell.
    pub fn set_merkle_root(&mut self, merkle_root: [u8; 32]) -> Result<(), SystemError> {
        if let Some(root_cell) = self.get_root_cell_mut() {
            root_cell.merkle_hash = merkle_root;
            Ok(())
        } else {
            Err(SystemError::new(
                SystemErrorType::InvalidOperation,
                "No root cell defined".to_string(),
            ))
        }
    }

    /// Sets the proof data for the root cell.
    pub fn set_proof(&mut self, proof: &[u8]) -> Result<(), SystemError> {
        if let Some(root_cell) = self.get_root_cell_mut() {
            root_cell.proof = Some(proof.to_vec());
            Ok(())
        } else {
            Err(SystemError::new(
                SystemErrorType::NoRootCell,
                "No root cell defined".to_string(),
            ))
        }
    }

    /// Retrieves the Merkle root hash of the `BOC`.
    pub fn root_hash(&self) -> Result<[u8; 32], SystemError> {
        if let Some(root_cell) = self.get_root_cell() {
            Ok(root_cell.merkle_hash)
        } else {
            Err(SystemError::new(
                SystemErrorType::NoRootCell,
                "No root cell defined".to_string(),
            ))
        }
    }

    /// Serializes the `BOC` into a JSON byte vector.
    pub fn serialize(&self) -> Result<Vec<u8>, BocError> {
        serde_json::to_vec(self).map_err(|e| BocError::SerializationError(e.to_string()))
    }
    /// Deserializes a `BOC` from a JSON byte vector.
    /// Returns an error if the byte vector is not valid JSON.
    pub fn deserialize(data: &[u8]) -> Result<Self, BocError> {
        serde_json::from_slice(data).map_err(|e| BocError::DeserializationError(e.to_string()))
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_creation() {
        let data = vec![1, 2, 3];
        let refs = vec![0, 1];
        let hash = [0u8; 32];

        let cell = Cell::new(data.clone(), refs.clone(), CellType::Ordinary, hash, None);

        assert_eq!(cell.data, data);
        assert_eq!(cell.references, refs);
        assert!(matches!(cell.cell_type, CellType::Ordinary));
        assert_eq!(cell.merkle_hash, hash);
        assert!(cell.proof.is_none());
    }

    #[test]
    fn test_cell_with_data() {
        let data = vec![1, 2, 3];
        let mut cell = Cell::with_data(data.clone());

        assert_eq!(cell.data, data);
        assert!(cell.references.is_empty());
        assert!(matches!(cell.cell_type, CellType::Ordinary));

        cell.update_merkle_hash();
        assert_ne!(cell.merkle_hash, [0u8; 32]);
    }

    #[test]
    fn test_boc_operations() {
        let mut boc = BOC::new();
        assert!(boc.is_empty());

        let cell1 = Cell::with_data(vec![1, 2, 3]);

        let idx1 = boc.add_cell(cell1);
        boc.add_root(idx1);

        assert_eq!(boc.cell_count(), 2);
        assert_eq!(boc.root_count(), 1);

        let root_cell = boc.get_root_cell().unwrap();
        assert_eq!(root_cell.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_boc_clear() {
        let mut boc = BOC::new();
        let cell = Cell::with_data(vec![1, 2, 3]);
        let idx = boc.add_cell(cell);
        boc.add_root(idx);

        assert!(!boc.is_empty());
        boc.clear();
        assert!(boc.is_empty());
        assert_eq!(boc.root_count(), 0);
    }

    #[test]
    fn test_boc_set_data() {
        let mut boc = BOC::new();
        let cell = Cell::with_data(vec![1, 2, 3]);
        let idx = boc.add_cell(cell);
        boc.add_root(idx);

        let new_data = vec![7, 8, 9];
        boc.set_data(&new_data).unwrap();

        let root_cell = boc.get_root_cell().unwrap();
        assert_eq!(root_cell.data, new_data);
    }

    #[test]
    fn test_boc_set_merkle_root() {
        let mut boc = BOC::new();
        let cell = Cell::with_data(vec![1, 2, 3]);
        let idx = boc.add_cell(cell);
        boc.add_root(idx);

        let new_merkle_root = [42u8; 32];
        boc.set_merkle_root(new_merkle_root).unwrap();

        let root_cell = boc.get_root_cell().unwrap();
        assert_eq!(root_cell.merkle_hash, new_merkle_root);
    }

    #[test]
    fn test_boc_set_proof() {
        let mut boc = BOC::new();
        let cell = Cell::with_data(vec![1, 2, 3]);
        let idx = boc.add_cell(cell);
        boc.add_root(idx);

        let proof_data = vec![10, 20, 30];
        boc.set_proof(&proof_data).unwrap();

        let root_cell = boc.get_root_cell().unwrap();
        assert_eq!(root_cell.proof.as_ref().unwrap(), &proof_data);
    }

    #[test]
    fn test_boc_root_hash() {
        let mut boc = BOC::new();
        let mut cell = Cell::with_data(vec![1, 2, 3]);
        cell.update_merkle_hash();
        let idx = boc.add_cell(cell);
        boc.add_root(idx);

        let root_hash = boc.root_hash().unwrap();
        assert_eq!(root_hash, boc.get_root_cell().unwrap().merkle_hash);
    }

    #[test]
    fn test_boc_serialize_deserialize() {
        let mut boc = BOC::new();
        let mut cell = Cell::with_data(vec![10, 20, 30]);
        cell.update_merkle_hash();
        let idx = boc.add_cell(cell);
        boc.add_root(idx);

        let serialized = boc.serialize().unwrap();
        let deserialized = BOC::deserialize(&serialized).unwrap();

        assert_eq!(boc.cells.len(), deserialized.cells.len());
        assert_eq!(boc.roots.len(), deserialized.roots.len());
        assert_eq!(boc.cells[0].data, deserialized.cells[0].data);
        assert_eq!(boc.cells[0].merkle_hash, deserialized.cells[0].merkle_hash);
    }

    #[test]
    fn test_boc_no_root_cell_error() {
        let mut boc = BOC::new();

        let result = boc.set_data(&[1, 2, 3]);
        assert!(matches!(
            result,
            Err(SystemError {
                error_type: SystemErrorType::NoRootCell,
                ..
            })
        ));
    }

    #[test]
    fn test_system_error_display() {
        let error = SystemError::new(
            SystemErrorType::InvalidTransaction,
            "Transaction validation failed".to_string(),
        );
        assert_eq!(
            error.to_string(),
            "Invalid transaction: Transaction validation failed"
        );
    }
}
