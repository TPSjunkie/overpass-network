// src/wasm/bindings_wasm.rs

use crate::core::types::ovp_types::*;
use crate::wasm::types_wasm::{WasmCell, WasmCellType};
use crate::SparseMerkleTree;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn cell_to_json(cell: &WasmCell) -> String {
    serde_json::to_string(&cell).unwrap_or_default()
}

#[wasm_bindgen]
pub fn cell_to_boc(cell: &WasmCell) -> Vec<u8> {
    let cell_type = match cell.cell_type {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::PrunedBranch,
        WasmCellType::LibraryReference => CellType::LibraryReference,
        WasmCellType::MerkleProof => CellType::MerkleProof,
        WasmCellType::MerkleUpdate => CellType::MerkleUpdate,
        WasmCellType::MerkleRoot => CellType::MerkleRoot,
        WasmCellType::MerklePath => CellType::MerklePath,
        WasmCellType::MerklePathBranch => CellType::MerklePathBranch,
        WasmCellType::MerklePathExtension => CellType::MerklePathExtension,
        WasmCellType::MerklePathLeaf => CellType::MerklePathLeaf,
        WasmCellType::MerklePathNode => CellType::MerklePathNode,
    };
    let cell_core = Cell::new(cell.data().clone(), cell_type, Vec::new(), 0);
    cell_core.data().clone() // Simplified for example purposes
}
#[wasm_bindgen]
pub fn smt_to_boc(smt: &SparseMerkleTree) -> Vec<u8> {
    smt.to_boc().unwrap_or_default()
}
