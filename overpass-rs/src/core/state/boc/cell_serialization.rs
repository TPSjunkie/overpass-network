// ./src_new/core/state/BOC/serialization.rs

use crate::core::error::CellError;
use crate::core::state::boc::cell::Cell;
use crate::core::types::CellReference;
use serde::Serialize;
use sha2::Digest;
use std::io::{self, Read, Write};

/// Serialization and deserialization for `Cell` and `CellReference`
pub struct BOC {
    cells: Vec<Cell>,
    roots: Vec<usize>,
}

impl BOC {
    pub fn new(cells: Vec<Cell>, roots: Vec<usize>) -> Self {
        Self { cells, roots }
    }
    pub fn cells(&self) -> &Vec<Cell> {
        &self.cells
    }
    pub fn roots(&self) -> &Vec<usize> {
        &self.roots
    }
}
pub trait Serializable {
    fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()>;
}

impl Serializable for CellReference {
    fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&(self.cell_index as u32).to_le_bytes())?;
        writer.write_all(&self.merkle_hash)?;
        Ok(())
    }
}

impl Deserializable for CellReference {
    fn deserialize<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut index_buf = [0; 4];
        reader.read_exact(&mut index_buf)?;
        let cell_index = u32::from_le_bytes(index_buf) as usize;

        let mut merkle_hash = [0; 32];
        reader.read_exact(&mut merkle_hash)?;

        Ok(CellReference {
            cell_index,
            merkle_hash,
        })
    }
}

pub trait Deserializable: Sized {
    fn deserialize<R: Read>(reader: &mut R) -> io::Result<Self>;
}
impl Serializable for Cell {
    fn serialize<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&(self.get_data().len() as u32).to_le_bytes())?;
        writer.write_all(self.get_data())?;

        writer.write_all(&(self.get_references().len() as u32).to_le_bytes())?;
        for reference in self.get_references() {
            reference.serialize(writer)?;
        }

        writer.write_all(&[self.get_cell_type().to_u8()])?;
        writer.write_all(self.get_merkle_hash())?;

        Ok(())
    }
}

impl Deserializable for Cell {
    fn deserialize<R: Read>(reader: &mut R) -> io::Result<Self> {
        let mut len_buf = [0; 4];
        reader.read_exact(&mut len_buf)?;
        let data_len = u32::from_le_bytes(len_buf) as usize;

        let mut data = vec![0; data_len];
        reader.read_exact(&mut data)?;

        reader.read_exact(&mut len_buf)?;
        let references_len = u32::from_le_bytes(len_buf) as usize;
        let mut references = Vec::with_capacity(references_len);
        for _ in 0..references_len {
            let mut ref_hash = [0; 32];
            reader.read_exact(&mut ref_hash)?;
            references.push(CellReference::deserialize(reader)?);
        }

        let mut cell_type_buf = [0; 1];
        reader.read_exact(&mut cell_type_buf)?;
        let cell_type = CellType::from_u8(cell_type_buf[0])
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid cell type"))?;

        let mut merkle_hash = [0; 32];
        reader.read_exact(&mut merkle_hash)?;

        Cell::new(data, cell_type)
            .and_then(|mut cell| {
                for reference in references {
                    cell.add_reference(reference)?;
                }
                cell.set_merkle_hash(merkle_hash);
                Ok(cell)
            })
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }
}

pub struct CellType {
    value: u8,
}
impl CellType {
    pub fn from_u8(value: u8) -> Result<Self, CellError> {
        match value {
            0 => Ok(CellType::Ordinary),
            1 => Ok(CellType::PrunedBranch),
            2 => Ok(CellType::LibraryReference),
            3 => Ok(CellType::MerkleProof),
            4 => Ok(CellType::MerkleUpdate),
            _ => Err(CellError::InvalidData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::CellType;
    use std::io::Cursor;

    #[test]
    fn test_serialization_deserialization() {
        let data = vec![1, 2, 3, 4];
        let reference = CellReference {
            cell_index: 0,
            merkle_hash: [0u8; 32],
        };
        let mut cell = Cell::new(data.clone(), CellType::MerkleProof).unwrap();
        cell.add_reference(reference).unwrap();

        let mut serialized_data = Vec::new();
        cell.serialize(&mut serialized_data).unwrap();

        let mut cursor = Cursor::new(&serialized_data);
        let deserialized_cell = Cell::deserialize(&mut cursor).unwrap();

        assert_eq!(cell.data(), deserialized_cell.data());
        assert_eq!(
            cell.references().len(),
            deserialized_cell.references().len()
        );
        assert_eq!(cell.cell_type(), deserialized_cell.cell_type());
        assert_eq!(cell.merkle_hash(), deserialized_cell.merkle_hash());
    }
}
impl BOC {
    /// Adds a cell to the BOC.
    pub fn add_cell(&mut self, cell: Cell) -> [u8; 32] {
        let hash = cell.calculate_hash();
        self.cells.insert(hash, cell);
        hash
    }

    /// Marks a cell as a root, using its hash.
    pub fn add_root(&mut self, hash: [u8; 32]) {
        if self.cells.contains_key(&hash) {
            self.roots.push(hash);
        }
    }

    /// Retrieves a cell by its hash.
    pub fn get_cell(&self, hash: &[u8; 32]) -> Option<&Cell> {
        self.cells.get(hash)
    }
}

pub trait BOCSerializer {
    fn serialize_boc(boc: &BOC) -> io::Result<Vec<u8>>;
    fn serialize_cell(cell: &Cell) -> io::Result<Vec<u8>>;
    fn deserialize_cell<R: Read>(reader: &mut R) -> io::Result<Cell>;
    fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8>;
    fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32>;
}

impl BOCSerializer for BOC {
    fn serialize_boc(boc: &BOC) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Encode root count and cell count
        buffer.write_all(&(boc.roots.len() as u32).to_le_bytes())?;
        buffer.write_all(&(boc.cells.len() as u32).to_le_bytes())?;

        // Serialize each cell
        for hash in &boc.roots {
            if let Some(cell) = boc.get_cell(hash) {
                buffer.extend(Self::serialize_cell(cell)?);
            }
        }

        Ok(buffer)
    }

    fn serialize_cell(cell: &Cell) -> io::Result<Vec<u8>> {
        let mut cell_buffer = Vec::new();

        // Write level, cell type, and data size
        cell_buffer.push(cell.level());
        cell_buffer.push(cell.cell_type() as u8);
        cell_buffer.push(cell.data().len() as u8);

        // Write data
        cell_buffer.extend(cell.data());

        // Write references count and hashes
        cell_buffer.push(cell.references().len() as u8);
        for reference in cell.references() {
            cell_buffer.extend(reference);
        }

        Ok(cell_buffer)
    }

    fn deserialize_cell<R: Read>(reader: &mut R) -> io::Result<Cell> {
        let level = Self::read_u8(reader)?;
        let cell_type = Self::read_u8(reader)?;
        let data_size = Self::read_u8(reader)?;

        let mut data = vec![0; data_size as usize];
        reader.read_exact(&mut data)?;

        let refs_count = Self::read_u8(reader)?;
        let mut refs = Vec::new();
        for _ in 0..refs_count {
            let mut ref_hash = [0; 32];
            reader.read_exact(&mut ref_hash)?;
            refs.push(ref_hash);
        }

        Cell::new(
            data,
            match cell_type {
                0 => CellType::Ordinary,
                1 => CellType::LibraryReference,
                2 => CellType::Pruned,
                3 => CellType::Root,
                _ => CellType::Code,
            },
        )
        .map(|mut cell| {
            cell.set_level(level);
            for reference in refs {
                cell.add_reference(reference).unwrap();
            }
            cell
        })
    }

    fn read_u8<R: Read>(reader: &mut R) -> io::Result<u8> {
        let mut buf = [0];
        reader.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_u32<R: Read>(reader: &mut R) -> io::Result<u32> {
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}
