use crate::core::error::errors::{SystemError, SystemErrorType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BOC {
    pub cells: Vec<Vec<u8>>,
    pub references: Vec<Vec<u8>>,
    pub roots: Vec<Vec<u8>>,
    pub hash: Option<[u8; 32]>,
}

impl BOC {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            references: Vec::new(),
            roots: Vec::new(),
            hash: None,
        }
    }

    pub fn with_hash(mut self, hash: [u8; 32]) -> Self {
        self.hash = Some(hash);
        self
    }

    pub fn with_cells(mut self, cells: Vec<Vec<u8>>) -> Self {
        self.cells = cells;
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

    pub fn cells(&self) -> &Vec<Vec<u8>> {
        &self.cells
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

    pub fn set_cells(&mut self, cells: Vec<Vec<u8>>) {
        self.cells = cells;
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
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();

        // Hash cells
        for cell in &self.cells {
            hasher.update(cell);
        }

        // Hash references
        for reference in &self.references {
            hasher.update(reference);
        }

        // Hash roots
        for root in &self.roots {
            hasher.update(root);
        }

        hasher.finalize().into()
    }
}

impl Default for BOC {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boc_new() {
        let boc = BOC::new();
        assert!(boc.cells.is_empty());
        assert!(boc.references.is_empty());
        assert!(boc.roots.is_empty());
        assert!(boc.hash.is_none());
    }

    #[test]
    fn test_boc_builder() {
        let cells = vec![vec![1, 2, 3]];
        let references = vec![vec![4, 5, 6]];
        let roots = vec![vec![7, 8, 9]];
        let hash = [0u8; 32];

        let boc = BOC::new()
            .with_cells(cells.clone())
            .with_references(references.clone())
            .with_roots(roots.clone())
            .with_hash(hash);

        assert_eq!(boc.cells(), &cells);
        assert_eq!(boc.references(), &references);
        assert_eq!(boc.roots(), &roots);
        assert_eq!(boc.hash(), hash);
    }

    #[test]
    fn test_boc_setter() {
        let mut boc = BOC::new();
        let cells = vec![vec![1, 2, 3]];
        let references = vec![vec![4, 5, 6]];
        let roots = vec![vec![7, 8, 9]];
        let hash = [0u8; 32];

        boc.set_cells(cells.clone());
        boc.set_references(references.clone());
        boc.set_roots(roots.clone());
        boc.set_hash(hash);

        assert_eq!(boc.cells(), &cells);
        assert_eq!(boc.references(), &references);
        assert_eq!(boc.roots(), &roots);
        assert_eq!(boc.hash(), hash);
    }

    #[test]
    fn test_boc_serialization() {
        let boc = BOC::new()
            .with_cells(vec![vec![1, 2, 3]])
            .with_references(vec![vec![4, 5, 6]])
            .with_roots(vec![vec![7, 8, 9]]);

        let serialized = boc.serialize().unwrap();
        let deserialized = BOC::deserialize(&serialized).unwrap();

        assert_eq!(boc, deserialized);
    }

    #[test]
    fn test_compute_hash() {
        let boc = BOC::new()
            .with_cells(vec![vec![1, 2, 3]])
            .with_references(vec![vec![4, 5, 6]])
            .with_roots(vec![vec![7, 8, 9]]);

        let hash = boc.compute_hash();
        assert_ne!(hash, [0u8; 32]);
    }
}
