// src/wasm/bindings_wasm.rs

use crate::common::types::state_boc::Cell;
use crate::common::types::state_boc::CellType;
use crate::wasm::types_wasm::{WasmCell, WasmCellType};
use wasm_bindgen::prelude::*;
use serde::Serialize;

#[wasm_bindgen]
pub fn cell_to_boc(cell: &WasmCell) -> Vec<u8> {
    let cell_type = match cell.get_cell_type() {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::Ordinary,
        WasmCellType::LibraryReference => CellType::Ordinary,
        WasmCellType::MerkleProof => CellType::Ordinary,
        WasmCellType::MerkleUpdate => CellType::Ordinary,
        _ => CellType::Ordinary,
    };
    let cell_core = Cell::new(
        cell.get_data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        [0; 32],
        None,
    );
    cell_core.serialize().unwrap_or_default()
}
#[wasm_bindgen]
pub fn cell_to_json(cell: &WasmCell) -> String {
    let cell_type = match cell.get_cell_type() {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::Ordinary,
        WasmCellType::LibraryReference => CellType::Ordinary,
        WasmCellType::MerkleProof => CellType::Ordinary,
        WasmCellType::MerkleUpdate => CellType::Ordinary,
        _ => CellType::Ordinary,
    };
    let cell_core = Cell::new(
        cell.get_data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        [0; 32],
        None,
    );
    serde_json::to_string(&CellWrapper(cell_core)).unwrap_or_default()
}

#[wasm_bindgen]
pub fn cell_to_boc_with_hash(cell: &WasmCell) -> Vec<u8> {
    let cell_type = match cell.get_cell_type() {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::Ordinary,
        WasmCellType::LibraryReference => CellType::Ordinary,
        WasmCellType::MerkleProof => CellType::Ordinary,
        WasmCellType::MerkleUpdate => CellType::Ordinary,
    };
    let cell_core = Cell::new(
        cell.get_data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        [0; 32],
        None,
    );
    cell_core.get_data().to_vec()
}#[wasm_bindgen]
pub fn cell_to_json_with_hash(cell: &WasmCell) -> String {
    let cell_type = match cell.get_cell_type() {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::Ordinary,
        WasmCellType::LibraryReference => CellType::Ordinary,
        WasmCellType::MerkleProof => CellType::Ordinary,
        WasmCellType::MerkleUpdate => CellType::Ordinary,
    };
    let cell_core = Cell::new(
        cell.get_data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        [0; 32],
        None,
    );
    serde_json::to_string(&CellWrapper(cell_core)).unwrap_or_default() // Simplified for example purposes
}

#[derive(Serialize)]
struct CellWrapper(#[serde(serialize_with = "serialize_cell")] Cell);

fn serialize_cell<S>(cell: &Cell, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Implement custom serialization for Cell
    // This is a placeholder implementation
    serializer.serialize_str(&format!("Cell: {:?}", cell))
}