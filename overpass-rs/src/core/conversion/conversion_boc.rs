// ./src/types/boc/conversion_boc.rs

use crate::core::types::ovp_types::{Cell, CellData, SliceData};
use crate::core::types::CellType;
use serde_wasm_bindgen::Error as SerdeWasmBindgenError;
/// Deserializes a cell from BOC format to structured `CellData`.
pub fn deserialize_cell<T, E>(cell: &Cell) -> Result<T, E>
where
    T: Default + CellData,
    E: From<SerdeWasmBindgenError>,
{
    let mut cell_data = T::default();
    let mut slice_data = SliceData::from(cell.as_slice());

    while !slice_data.is_empty() {
        let (cell_type, remaining_data) = deserialize_cell_type(slice_data.as_slice())?;
        slice_data = remaining_data;
        cell_data.push(cell_type, slice_data.clone());
    }

    Ok(cell_data)
}
/// Helper function to extract cell type from raw data slice.
fn deserialize_cell_type(data: &[u8]) -> Result<(CellType, SliceData), SerdeWasmBindgenError> {
    let cell_type = CellType::from(data[0]);
    let slice_data = SliceData::from(&data[1..]);
    Ok((cell_type, slice_data))
}

/// Serializes `CellData` back into a BOC `Cell`.
pub fn serialize_cell<T: CellData>(cell_data: &T) -> Result<Cell, SerdeWasmBindgenError> {
    let mut cell = Cell::new();
    let mut slice_data = SliceData::new();

    for (cell_type, data) in cell_data.iter() {
        slice_data.append_raw(&[cell_type.to_byte()], data.as_slice())?;
    }

    cell.data = slice_data.into();
    Ok(cell)
}
