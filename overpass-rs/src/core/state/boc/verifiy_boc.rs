// src/core/state/boc/validation.rs

use super::cell::*;
use crate::core::state::boc::cell::MAX_REFERENCES;
use crate::core::types::ovp_types::BOC;
use crate::core::types::CellReference;
use crate::core::types::CellType;
use sha2::Digest;
use std::collections::HashSet;

pub struct VerifyBOC {
    max_cell_count: usize,
    max_bits_size: usize,
    max_depth: usize,
}

impl Default for VerifyBOC {
    fn default() -> Self {
        Self {
            max_cell_count: 1000,
            max_bits_size: 1000,
            max_depth: 100,
        }
    }
}

impl VerifyBOC {
    pub fn new(max_cell_count: usize, max_bits_size: usize, max_depth: usize) -> Self {
        Self {
            max_cell_count,
            max_bits_size,
            max_depth,
        }
    }

    pub fn verify_boc(&self, boc: &BOC) -> Result<(), BocError> {
        // verify basic BOC structure
        self.verify_structure(boc)?;

        // verify all cells
        for (i, cell) in boc.cells.iter().enumerate() {
            self.verify_cell(cell, i, boc)?;
        }

        // verify root references
        self.verify_roots(boc)?;

        // verify the overall DAG structure
        self.verify_dag(boc)?;

        Ok(())
    }

    fn verify_structure(&self, boc: &BOC) -> Result<(), BocError> {
        // Check cell count
        if boc.cells.len() > self.max_cell_count {
            return Err(BocError::TooManyCells);
        }

        // Check that we have at least one root
        if boc.roots.is_empty() {
            return Err(BocError::NoRoots);
        }

        // Check total bits size
        let total_bits: usize = boc.cells.iter().map(|cell| cell.bit_length()).sum();
        if total_bits > self.max_bits_size {
            return Err(BocError::TotalSizeTooLarge);
        }

        Ok(())
    }

    fn verify_cell(&self, cell: &Cell, index: usize, boc: &BOC) -> Result<(), BocError> {
        // verify data size
        if cell.data().len() > MAX_BYTES {
            return Err(BocError::CellDataTooLarge);
        }

        // verify reference count
        if cell.references().len() > MAX_REFERENCES {
            return Err(BocError::TooManyReferences);
        }

        // verify all references point to valid cells
        for reference in cell.references() {
            if reference.cell_index >= boc.cells.len() {
                return Err(BocError::InvalidReference {
                    from: index,
                    to: reference.cell_index,
                });
            }
        }

        // verify cell type specific rules
        match cell.cell_type() {
            CellType::MerkleProof => {
                // Merkle proof cells must have valid merkle hash
                if !cell.verify_merkle_hash()? {
                    return Err(BocError::InvalidMerkleProof);
                }
            }
            CellType::PrunedBranch => {
                // Pruned branches cannot have references
                if !cell.references().is_empty() {
                    return Err(BocError::InvalidPrunedBranch);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn verify_roots(&self, boc: &BOC) -> Result<(), BocError> {
        for &root_idx in &boc.roots {
            if root_idx >= boc.cells.len() {
                return Err(BocError::InvalidRoot(root_idx));
            }
        }
        Ok(())
    }

    fn verify_dag(&self, boc: &BOC) -> Result<(), BocError> {
        // Detect cycles
        let mut visited = HashSet::new();
        let mut path = Vec::new();

        for &root in &boc.roots {
            self.check_cycle(root, &mut visited, &mut path, boc, 0)?;
        }

        Ok(())
    }

    fn check_cycle(
        &self,
        cell_idx: usize,
        visited: &mut HashSet<usize>,
        path: &mut Vec<usize>,
        boc: &BOC,
        depth: usize,
    ) -> Result<(), BocError> {
        // Check max depth
        if depth > self.max_depth {
            return Err(BocError::MaxDepthExceeded);
        }

        // Check for cycles
        if path.contains(&cell_idx) {
            return Err(BocError::CycleDetected);
        }

        // Skip if already fully verifyd
        if visited.contains(&cell_idx) {
            return Ok(());
        }

        path.push(cell_idx);

        let cell = &boc.cells[cell_idx];
        for reference in cell.references() {
            self.check_cycle(reference.cell_index, visited, path, boc, depth + 1)?;
        }

        path.pop();
        visited.insert(cell_idx);

        Ok(())
    }
}

#[derive(Debug)]
pub enum BocError {
    TooManyCells,
    NoRoots,
    TotalSizeTooLarge,
    CellDataTooLarge,
    TooManyReferences,
    InvalidReference { from: usize, to: usize },
    InvalidRoot(usize),
    InvalidMerkleProof,
    InvalidPrunedBranch,
    CycleDetected,
    MaxDepthExceeded,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_verifier() -> VerifyBOC {
        VerifyBOC::new(1000, 1_000_000, 256)
    }

    #[test]
    fn test_valid_boc() {
        let cell = Cell::new(vec![1, 2, 3], CellType::Ordinary).unwrap();
        let boc = BOC {
            cells: vec![cell],
            roots: vec![0],
        };

        let verifier = create_test_verifier();
        assert!(verifier.verify_boc(&boc).is_ok());
    }

    #[test]
    fn test_no_roots() {
        let cell = Cell::new(vec![1, 2, 3], CellType::Ordinary).unwrap();
        let boc = BOC {
            cells: vec![cell],
            roots: vec![],
        };

        let verifier = create_test_verifier();
        assert!(matches!(verifier.verify_boc(&boc), Err(BocError::NoRoots)));
    }

    #[test]
    fn test_invalid_root_reference() {
        let cell = Cell::new(vec![1, 2, 3], CellType::Ordinary).unwrap();
        let boc = BOC {
            cells: vec![cell],
            roots: vec![1], // Points to non-existent cell
        };

        let verifier = create_test_verifier();
        assert!(matches!(
            verifier.verify_boc(&boc),
            Err(BocError::InvalidRoot(_))
        ));
    }

    #[test]
    fn test_cycle_detection() {
        // Create cells that reference each other creating a cycle
        let mut cell1 = Cell::new(vec![1], CellType::Ordinary).unwrap();
        let mut cell2 = Cell::new(vec![2], CellType::Ordinary).unwrap();

        cell1
            .add_reference(CellReference {
                cell_index: 1,
                merkle_hash: [0u8; 32],
            })
            .unwrap();

        cell2
            .add_reference(CellReference {
                cell_index: 0,
                merkle_hash: [0u8; 32],
            })
            .unwrap();

        let boc = BOC {
            cells: vec![cell1, cell2],
            roots: vec![0],
        };

        let verifier = create_test_verifier();
        assert!(matches!(
            verifier.verify_boc(&boc),
            Err(BocError::CycleDetected)
        ));
    }

    #[test]
    fn test_max_depth() {
        // Create a chain of cells exceeding max depth
        let mut cells = Vec::new();
        let max_depth = 3;

        for i in 0..max_depth + 2 {
            let mut cell = Cell::new(vec![i as u8], CellType::Ordinary).unwrap();
            if i < max_depth + 1 {
                cell.add_reference(CellReference {
                    cell_index: i + 1,
                    merkle_hash: [0u8; 32],
                })
                .unwrap();
            }
            cells.push(cell);
        }

        let boc = BOC {
            cells,
            roots: vec![0],
        };

        let verifier = VerifyBOC::new(1000, 1_000_000, max_depth);
        assert!(matches!(
            verifier.verify_boc(&boc),
            Err(BocError::MaxDepthExceeded)
        ));
    }

    #[test]
    fn test_invalid_merkle_proof() {
        let mut cell = Cell::new(vec![1, 2, 3], CellType::MerkleProof).unwrap();
        // Corrupt merkle hash
        cell.set_merkle_hash([0u8; 32]);

        let boc = BOC {
            cells: vec![cell],
            roots: vec![0],
        };

        let verifier = create_test_verifier();
        assert!(matches!(
            verifier.verify_boc(&boc),
            Err(BocError::InvalidMerkleProof)
        ));
    }
    #[test]
    fn test_invalid_pruned_branch() {
        let mut cell = Cell::new(vec![1], CellType::PrunedBranch).unwrap();
        // Add reference which is not allowed for pruned branches
        cell.add_reference(CellReference {
            cell_index: 0,
            merkle_hash: [0u8; 32],
        })
        .unwrap();

        let boc = BOC {
            cells: vec![cell],
            roots: vec![0],
        };

        let verifier = create_test_verifier();
        assert!(matches!(
            verifier.verify_boc(&boc),
            Err(BocError::InvalidPrunedBranch)
        ));
    }
}
