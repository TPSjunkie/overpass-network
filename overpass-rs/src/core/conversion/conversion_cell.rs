// ./src/core/types/cell/conversion_cell.rs

use crate::core::types::ovp_types::OMResult;
use crate::core::types::ovp_types::{Cell, CellData, SliceData};
use crate::core::types::CellType;

/// Deserializes a cell from BOC format to structured `CellData`.
pub fn deserialize_cell(cell: &Cell) -> OMResult<dyn CellData> {
    let mut cell_data = CellData::new();
    let mut slice_data = SliceData::from(cell.as_slice());

    while !slice_data.is_empty() {
        let (cell_type, remaining_data) = deserialize_cell_type(slice_data.as_slice())?;
        slice_data = remaining_data.into();
        cell_data.push(cell_type, slice_data.clone());
    }

    Ok(cell_data)
}
/// Helper function to extract cell type from raw data slice.
fn deserialize_cell_type(data: &[u8]) -> OMResult<(CellType, SliceData)> {
    let cell_type = CellType::from(data[0]);
    let slice_data = SliceData::from(&data[1..]);
    Ok((cell_type, slice_data))
}

/// Serializes `CellData` back into a BOC `Cell`.
pub fn serialize_cell(cell_data: &dyn CellData) -> OMResult<Cell> {
    let mut slice_data = SliceData::new();

    for (cell_type, data) in cell_data.iter() {
        slice_data.append_raw(&[cell_type.to_byte()], data.as_slice())?;
    }

    Cell::with_data(slice_data.into())
}
