use crate::core::error::errors::{BocError, SystemError, SystemErrorType};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub data: Vec<u8>,
    pub references: Vec<usize>,
    pub cell_type: CellType,
    pub merkle_hash: [u8; 32],
    pub proof: Option<Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RootCell {
    pub capacity: u64,
    pub lock_script: Script,
    pub type_script: Script,
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub code_hash: H256,
    pub hash_type: ScriptHashType,
    pub args: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ScriptHashType {
    Data,
    Type,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct H256(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    Ordinary,
    PrunedBranch,
    LibraryReference,
    MerkleProof,
    MerkleUpdate,
}

pub fn repr_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

pub fn hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

pub fn hash_pair(left: &[u8], right: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

pub fn hash_value(value: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(value);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

pub fn root_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

pub fn root_cell_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&hasher.finalize());
    hash
}

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

impl Cell {
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

    pub fn with_data(data: Vec<u8>) -> Self {
        Self {
            data,
            references: Vec::new(),
            cell_type: CellType::Ordinary,
            merkle_hash: [0u8; 32],
            proof: None,
        }
    }

    pub fn from_data(data: Vec<u8>) -> Self {
        Self::with_data(data)
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

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
    pub(crate) references: Vec<usize>,
}
impl BOC {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        // Hash structure elements
        for cell in &self.cells {
            hasher.update(&cell.merkle_hash); // Note: this merkle_hash was computed using Poseidon
            hasher.update(&cell.data);
            for &ref_idx in &cell.references {
                hasher.update(&ref_idx.to_le_bytes());
            }
        }

        // Hash roots and references
        for &root in &self.roots {
            hasher.update(&root.to_le_bytes());
        }
        for &reference in &self.references {
            hasher.update(&reference.to_le_bytes());
        }

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&hasher.finalize());
        hash
    }
}
    pub fn add_cell(&mut self, data: Vec<u8>) -> Result<usize, SystemError> {
        let mut cell = Cell::with_data(data);
        cell.update_merkle_hash();
        let index = self.cells.len();
        self.cells.push(cell);
        Ok(index)
    }

    pub fn add_proof_cell(&mut self, data: Vec<u8>, proof: Vec<u8>) -> Result<usize, SystemError> {
        let mut cell = Cell::new(
            data,
            Vec::new(),
            CellType::MerkleProof,
            [0u8; 32],
            Some(proof),
        );
        cell.update_merkle_hash();
        let index = self.cells.len();
        self.cells.push(cell);
        Ok(index)
    }

    pub fn add_reference(&mut self, from_idx: usize, to_idx: usize) -> Result<(), SystemError> {
        if from_idx >= self.cells.len() || to_idx >= self.cells.len() {
            return Err(SystemError::new(
                SystemErrorType::InvalidReference,
                "Cell index out of bounds".to_string(),
            ));
        }
        if let Some(cell) = self.cells.get_mut(from_idx) {
            cell.references.push(to_idx);
            cell.update_merkle_hash();
            Ok(())
        } else {
            Err(SystemError::new(
                SystemErrorType::InvalidReference,
                "Source cell not found".to_string(),
            ))
        }
    }

    pub fn add_root(&mut self, index: usize) {
        self.roots.push(index);
    }

    pub fn validate_bitcoin_state(&self) -> Result<bool, SystemError> {
        if self.cells.is_empty() {
            return Err(SystemError::new(
                SystemErrorType::InvalidState,
                "Empty BOC".to_string(),
            ));
        }
        if self.roots.len() != 2 {
            return Err(SystemError::new(
                SystemErrorType::InvalidState,
                "Invalid BOC structure".to_string(),
            ));
        }
        Ok(true)
    }

    pub fn get_merkle_proof(&self, cell_idx: usize) -> Result<Vec<u8>, SystemError> {
        if let Some(cell) = self.cells.get(cell_idx) {
            if let Some(proof) = &cell.proof {
                Ok(proof.clone())
            } else {
                Err(SystemError::new(
                    SystemErrorType::InvalidProof,
                    "Cell has no proof data".to_string(),
                ))
            }
        } else {
            Err(SystemError::new(
                SystemErrorType::InvalidReference,
                "Cell index not found".to_string(),
            ))
        }
    }

    pub fn extract_bitcoin_state(&self) -> Result<Vec<u8>, SystemError> {
        if let Some(root_cell) = self.get_root_cell() {
            if root_cell.references.len() != 2 {
                return Err(SystemError::new(
                    SystemErrorType::InvalidState,
                    "Invalid Bitcoin state structure".to_string(),
                ));
            }
            Ok(root_cell.data.clone())
        } else {
            Err(SystemError::new(
                SystemErrorType::InvalidState,
                "No roots found".to_string(),
            ))
        }
    }

    pub fn get_root_cell(&self) -> Option<&Cell> {
        self.roots.first().and_then(|&root_idx| self.cells.get(root_idx))
    }

    pub fn get_root_cell_mut(&mut self) -> Option<&mut Cell> {
        if let Some(&root_idx) = self.roots.first() {
            self.cells.get_mut(root_idx)
        } else {
            None
        }
    }

      
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.roots.clear();
    }

    pub fn set_data(&mut self, data: &[u8]) -> Result<(), SystemError> {
        if let Some(root_cell) = self.get_root_cell_mut() {
            root_cell.data = data.to_vec();
            root_cell.update_merkle_hash();
            Ok(())
        } else {
            Err(SystemError::new(
                SystemErrorType::NoRoots,
                "No roots found".to_string(),
            ))
        }
    }

    pub fn set_merkle_root(&mut self, merkle_root: [u8; 32]) -> Result<(), SystemError> {
        if let Some(root_cell) = self.get_root_cell_mut() {
            root_cell.merkle_hash = merkle_root;
            Ok(())
        } else {
            Err(SystemError::new(
                SystemErrorType::NoRoots,
                "No roots found".to_string(),
            ))
        }
    }

    pub fn set_proof(&mut self, proof: &[u8]) -> Result<(), SystemError> {
        if let Some(root_cell) = self.get_root_cell_mut() {
            root_cell.proof = Some(proof.to_vec());
            Ok(())
        } else {
            Err(SystemError::new(
                SystemErrorType::NoRoots,
                "No roots found".to_string(),
            ))
        }
    }


    pub fn root_hash(&self) -> Result<[u8; 32], SystemError> {
        if let Some(root_cell) = self.get_root_cell() {
            Ok(root_cell.merkle_hash)
        } else {
            Err(SystemError::new(
                SystemErrorType::NoRoots,
                "No roots found".to_string(),
            ))
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, BocError> {
        serde_json::to_vec(self).map_err(|e| BocError::SerializationError(e.to_string()))
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, BocError> {
        serde_json::from_slice(data).map_err(|e| BocError::DeserializationError(e.to_string()))
    }
}
    
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
        let idx1 = boc.add_cell(vec![1, 2, 3]).unwrap();
        boc.add_root(idx1);

        assert_eq!(boc.cell_count(), 1);
        assert_eq!(boc.root_count(), 1);

        let root_cell = boc.get_root_cell().unwrap();
        assert_eq!(root_cell.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_boc_clear() {
        let mut boc = BOC::new();
        let idx = boc.add_cell(vec![1, 2, 3]).unwrap();
        boc.add_root(idx);

        assert!(!boc.is_empty());
        boc.clear();
        assert!(boc.is_empty());
        assert_eq!(boc.root_count(), 0);
    }

    #[test]
    fn test_boc_set_data() {
        let mut boc = BOC::new();
        let idx = boc.add_cell(vec![1, 2, 3]).unwrap();
        boc.add_root(idx);

        let new_data = vec![7, 8, 9];
        boc.set_data(&new_data).unwrap();

        let root_cell = boc.get_root_cell().unwrap();
        assert_eq!(root_cell.data, new_data);
    }

    #[test]
    fn test_boc_set_merkle_root() {
        let mut boc = BOC::new();
        let idx = boc.add_cell(vec![1, 2, 3]).unwrap();
        boc.add_root(idx);

        let new_merkle_root = [42u8; 32];
        boc.set_merkle_root(new_merkle_root).unwrap();

        let root_cell = boc.get_root_cell().unwrap();
        assert_eq!(root_cell.merkle_hash, new_merkle_root);
    }

    #[test]
    fn test_boc_set_proof() {
        let mut boc = BOC::default();
        let idx = boc.add_cell(vec![1, 2, 3]).unwrap();
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
        let idx = boc.add_cell(vec![1, 2, 3]).unwrap();
        boc.add_root(idx);

        let root_hash = boc.root_hash().unwrap();
        assert_eq!(root_hash, boc.get_root_cell().unwrap().merkle_hash);
    }

    #[test]
    fn test_boc_serialize_deserialize() -> Result<(), SystemError> {
        let mut boc = BOC::new();
        let idx = boc.add_cell(vec![10, 20, 30]).unwrap();
        boc.add_root(idx);

        let serialized = boc.serialize().unwrap();
        let deserialized = BOC::deserialize(&serialized).unwrap();

        assert_eq!(boc.cells.len(), deserialized.cells.len());
        assert_eq!(boc.roots.len(), deserialized.roots.len());
        assert_eq!(boc.cells[0].data, deserialized.cells[0].data);
        assert_eq!(boc.cells[0].merkle_hash, deserialized.cells[0].merkle_hash);
        Ok(())
    }

    #[test]
    fn test_boc_no_root_cell_error() {
        let mut boc = BOC::new();
        let result = boc.set_data(&[1, 2, 3]);
        assert!(matches!(
            result,
            Err(SystemError {
                error_type: SystemErrorType::NoRoots,
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
