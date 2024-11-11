// src/core/state/BOC/builder.rs
use crate::core::error::errors::CellError;
use crate::core::state::boc::cell::Cell;
use crate::core::types::BOC;
use sha2::Digest;
use std::collections::HashMap;
/// A builder for constructing BOCs.
pub struct Builder {
    cells: Vec<Cell>,
    cell_indices: HashMap<[u8; 32], usize>,
    max_cell_count: usize,
    max_bits_size: usize,
    max_depth: usize,
}

impl Builder {
    /// Creates a new `Builder` with default limits.
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            cell_indices: HashMap::new(),
            max_cell_count: 1000,
            max_bits_size: 1_000_000,
            max_depth: 256,
        }
    }

    /// Creates a new `Builder` with custom limits.
    pub fn with_limits(max_cell_count: usize, max_bits_size: usize, max_depth: usize) -> Self {
        Self {
            cells: Vec::new(),
            cell_indices: HashMap::new(),
            max_cell_count,
            max_bits_size,
            max_depth,
        }
    }

    /// Adds a cell to the builder and returns its index.
    pub fn add_cell(&mut self, mut cell: Cell) -> Result<usize, CellError> {
        // Validate cell before adding
        if self.cells.len() >= self.max_cell_count {
            return Err(CellError::TooManyReferences);
        }

        // Calculate cell hash
        cell.calculate_merkle_hash()?;
        let hash = cell.calculate_hash();

        // Check if cell already exists
        if let Some(&idx) = self.cell_indices.get(&hash) {
            return Ok(idx);
        }

        // Add new cell
        let idx = self.cells.len();
        self.cells.push(cell);
        self.cell_indices.insert(hash, idx);
        Ok(idx)
    }

    /// Builds the BOC with the specified root indices.
    pub fn build(&self, roots: Vec<usize>) -> Result<BOC, CellError> {
        // Validate roots
        for &root in &roots {
            if root >= self.cells.len() {
                return Err(CellError::InvalidData);
            }
        }

        // Calculate total bits size
        let total_bits: usize = self.cells.iter().map(|cell| cell.data().len() * 8).sum();
        if total_bits > self.max_bits_size {
            return Err(CellError::DataTooLarge);
        }

        // Create BOC
        let mut boc = BOC {
            cells: self.cells.clone(),
            roots,
        };

        // Validate references
        for (i, cell) in boc.cells.iter().enumerate() {
            for reference in cell.references() {
                if reference.cell_index >= boc.cells.len() {
                    return Err(CellError::InvalidData);
                }
            }
        }

        Ok(boc)
    }

    /// Serializes the built BOC into bytes.
    pub fn serialize(&self, roots: Vec<usize>) -> Result<Vec<u8>, CellError> {
        let boc = self.build(roots)?;
        let mut buffer = Vec::new();

        // Write magic bytes and version
        buffer.extend_from_slice(&[0xB5, 0xEE, 0x9C, 0x72]); // Magic bytes
        buffer.push(0x01); // Version

        // Write counts
        buffer.extend_from_slice(&(boc.cells.len() as u32).to_le_bytes());
        buffer.extend_from_slice(&(boc.roots.len() as u32).to_le_bytes());

        // Write cells
        for cell in &boc.cells {
            cell.serialize(&mut buffer)
                .map_err(|_| CellError::InvalidData)?;
        }

        // Write roots
        for &root in &boc.roots {
            buffer.extend_from_slice(&(root as u32).to_le_bytes());
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::CellType;

    #[test]
    fn test_builder_basic() {
        let mut builder = Builder::new();
        let cell = Cell::new(vec![1, 2, 3], CellType::Ordinary).unwrap();
        let idx = builder.add_cell(cell.clone()).unwrap();
        assert_eq!(idx, 0);

        let boc = builder.build(vec![0]).unwrap();
        assert_eq!(boc.cells.len(), 1);
        assert_eq!(boc.roots, vec![0]);
    }

    #[test]
    fn test_builder_duplicate_cells() {
        let mut builder = Builder::new();
        let cell = Cell::new(vec![1, 2, 3], CellType::Ordinary).unwrap();

        let idx1 = builder.add_cell(cell.clone()).unwrap();
        let idx2 = builder.add_cell(cell.clone()).unwrap();

        assert_eq!(idx1, idx2);
        assert_eq!(builder.cells.len(), 1);
    }

    #[test]
    fn test_builder_serialization() {
        let mut builder = Builder::new();
        let cell = Cell::new(vec![1, 2, 3], CellType::Ordinary).unwrap();
        let idx = builder.add_cell(cell).unwrap();

        let serialized = builder.serialize(vec![idx]).unwrap();
        assert!(!serialized.is_empty());
    }

    #[test]
    fn test_builder_limits() {
        let mut builder = Builder::with_limits(1, 1000, 256);
        let cell1 = Cell::new(vec![1], CellType::Ordinary).unwrap();
        let cell2 = Cell::new(vec![2], CellType::Ordinary).unwrap();

        assert!(builder.add_cell(cell1).is_ok());
        assert!(builder.add_cell(cell2).is_err());
    }
}
