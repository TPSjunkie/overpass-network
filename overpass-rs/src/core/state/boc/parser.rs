// src/core/state/boc/parser.rs

use crate::core::types::ovp_types::*;

use std::io::{Cursor, Read};

use crate::core::error::CellError;

/// Parses a BOC (Bag of Cells) from raw bytes.
pub struct BOCParser;

impl BOCParser {
    /// Parses a BOC from the given bytes.
    pub fn parse(bytes: &[u8]) -> Result<BOC, CellError> {
        let mut reader = Cursor::new(bytes);
        let mut cells = Vec::new();
        // Read cell count
        let cell_count = BOCParser::read_u32(&mut reader)?;
        // Parse each cell
        for _ in 0..cell_count {
            let cell = Cell::deserialize(&mut reader)?;
            cells.push(cell);
        }
        // Read root indices
        let root_count = BOCParser::read_u32(&mut reader)?;
        let mut roots = Vec::new();
        for _ in 0..root_count {
            let root_idx = BOCParser::read_u32(&mut reader)? as usize;
            roots.push(root_idx);
        }
        Ok(BOC { cells, roots })
    }

    fn read_u32<R: Read>(reader: &mut R) -> Result<u32, CellError> {
        let mut buf = [0u8; 4];
        reader.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::state::boc::cell::Cell;

    #[test]
    fn test_parse_boc() {
        let cell = Cell::new(vec![1, 2, 3], Vec::new()).unwrap();
        let boc = BOC {
            cells: vec![cell],
            roots: vec![0],
        };

        let mut bytes = Vec::new();
        // Serialize cell count
        bytes.extend_from_slice(&(1u32).to_le_bytes());
        // Serialize cell
        boc.cells[0].serialize(&mut bytes).unwrap();
        // Serialize root count
        bytes.extend_from_slice(&(1u32).to_le_bytes());
        // Serialize root index
        bytes.extend_from_slice(&(0u32).to_le_bytes());

        let parsed_boc = BOCParser::parse(&bytes).unwrap();
        assert_eq!(parsed_boc.cells.len(), 1);
        assert_eq!(parsed_boc.roots.len(), 1);
    }
}
