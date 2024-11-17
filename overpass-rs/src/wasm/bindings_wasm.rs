// src/wasm/bindings_wasm.rs

use crate::core::types::boc::Cell;
use crate::core::types::boc::CellType;
use crate::wasm::types_wasm::{WasmCell, WasmCellType};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn cell_to_boc(cell: &WasmCell) -> Vec<u8> {
    let cell_type = match cell.get_cell_type() {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::PrunedBranch,
        WasmCellType::LibraryReference => CellType::LibraryReference,
        WasmCellType::MerkleProof => CellType::MerkleProof,
        WasmCellType::MerkleUpdate => CellType::MerkleUpdate,
        _ => CellType::Ordinary,
    };
    let cell_core = Cell::new(cell.get_data().clone(), Vec::new(), cell_type, [0; 32], None);
    cell_core.get_data().clone()
}

#[wasm_bindgen]
pub fn cell_to_json(cell: &WasmCell) -> String {
    let cell_type = match cell.get_cell_type() {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::PrunedBranch,
        WasmCellType::LibraryReference => CellType::LibraryReference,
        WasmCellType::MerkleProof => CellType::MerkleProof,
        WasmCellType::MerkleUpdate => CellType::MerkleUpdate,
        _ => CellType::Ordinary,
    };
    let cell_core = Cell::new(cell.get_data().clone(), Vec::new(), cell_type, [0; 32], None);
    serde_json::to_string(&cell_core).unwrap_or_default()
}

#[wasm_bindgen]
pub fn cell_to_boc_with_hash(cell: &WasmCell) -> Vec<u8> {
    let cell_type = match cell.get_cell_type() {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::PrunedBranch,
        WasmCellType::LibraryReference => CellType::LibraryReference,
        WasmCellType::MerkleProof => CellType::MerkleProof,
        WasmCellType::MerkleUpdate => CellType::MerkleUpdate,
    };
    let cell_core = Cell::new(cell.get_data().clone(), Vec::new(), cell_type, [0; 32], None);
    cell_core.get_data().clone() // Simplified for example purposes
}

#[wasm_bindgen]
pub fn cell_to_json_with_hash(cell: &WasmCell) -> String {
    let cell_type = match cell.get_cell_type() {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::PrunedBranch,
        WasmCellType::LibraryReference => CellType::LibraryReference,
        WasmCellType::MerkleProof => CellType::MerkleProof,
        WasmCellType::MerkleUpdate => CellType::MerkleUpdate,
    };
    let cell_core = Cell::new(cell.get_data().clone(), Vec::new(), cell_type, [0; 32], None);
    serde_json::to_string(&cell_core).unwrap_or_default() // Simplified for example purposes
}