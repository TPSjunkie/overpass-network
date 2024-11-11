// ./src/types/boc/validation_boc.rs

use crate::core::types::ovp_types::*;

/// Enum representing the different types of cells within a BOC.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellType {
    Ordinary = 0,
    Pruned = 1,
    MerkleProof = 2,
    MerkleUpdate = 3,
}

/// Validates the integrity of a BOC (Bag of Cells) structure.
pub fn validate_boc(boc: &BOC) -> TvmResult<()> {
    if boc.op_code == 0 || boc.data.is_empty() || boc.merkle_root.is_empty() {
        return Err(OMError::InvalidBoc);
    }

    validate_cells(&boc.data)?;
    Ok(())
}

/// Validates each cell in the BOC data.
fn validate_cells(data: &[u8]) -> OMResult<()> {
    let mut slice_data = data;
    while !slice_data.is_empty() {
        let (cell_type, remaining_data) = deserialize_cell_type(slice_data)?;
        validate_cell_type(cell_type)?;
        slice_data = remaining_data;
    }
    Ok(())
}

/// Confirms if the cell type is valid within the BOC specification.
fn validate_cell_type(cell_type: CellType) -> TvmResult<()> {
    match cell_type {
        CellType::Ordinary | CellType::Pruned | CellType::MerkleProof | CellType::MerkleUpdate => {
            Ok(())
        }
        _ => Err(OMError::InvalidCellType),
    }
}

/// Deserializes a cell type from a slice of bytes.
fn deserialize_cell_type(data: &[u8]) -> TvmResult<(CellType, &[u8])> {
    let cell_type = CellType::from(data[0]);
    let remaining_data = data.get(1..).ok_or(OMError::InvalidBoc)?;
    Ok((cell_type, remaining_data))
}
