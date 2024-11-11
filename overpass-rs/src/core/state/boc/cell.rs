// src/core/state/boc/cell.rs
use crate::core::error::CellError;
use crate::core::error::Error;
use crate::core::types::CellReference;
use crate::core::types::CellType;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};

pub const MAX_REFERENCES: usize = 4;
pub(crate) const MAX_BYTES: usize = 128;

/// Represents a cell in the Bag of Cells (BOCContract) structure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Cell {
    data: Vec<u8>,
    references: Vec<CellReference>,
    cell_type: CellType,
    merkle_hash: [u8; 32],
}

impl Cell {
    /// Creates a new `Cell`.
    pub fn new(data: Vec<u8>, cell_type: CellType) -> Result<Self, Error> {
        if data.len() > MAX_BYTES {
            return Err(Error::DataTooLarge);
        }

        let cell = Cell {
            data,
            references: Vec::new(),
            cell_type,
            merkle_hash: [0u8; 32],
        };

        Ok(cell)
    }

    /// Adds a reference to another cell.
    pub fn add_reference(&mut self, reference: CellReference) -> Result<(), CellError> {
        if self.references.len() >= MAX_REFERENCES {
            return Err(CellError::TooManyReferences);
        }

        self.references.push(reference);
        Ok(())
    }

    /// Calculates the Merkle hash of the cell.
    pub fn calculate_merkle_hash(&mut self) -> Result<(), CellError> {
        let mut hasher = Sha256::new();
        hasher.update(&[self.cell_type.to_u8()]);
        hasher.update(&self.data);

        for reference in &self.references {
            hasher.update(&reference.merkle_hash);
        }

        self.merkle_hash = hasher.finalize().into();
        Ok(())
    }

    // Other methods like getters omitted for brevity

    pub fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        // Serialization logic here
        Ok(())
    }

    pub fn deserialize<R: Read>(reader: &mut R) -> io::Result<Self> {
        // Deserialization logic here
        Ok(Cell {
            data: vec![],
            references: vec![],
            cell_type: CellType::Ordinary,
            merkle_hash: [0u8; 32],
        })
    }

    /// Calculates the hash of the cell's data and references.
    pub fn calculate_hash(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&self.data);
        for reference in &self.references {
            hasher.update(&reference.merkle_hash);
        }
        hasher.finalize().into()
    }
}
