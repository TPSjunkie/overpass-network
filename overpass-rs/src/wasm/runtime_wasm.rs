// src/wasm/runtime_wasm.rs

use wasm_bindgen::prelude::wasm_bindgen;

/// Represents a cell in the WASM environment.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WasmCell {
    cell_type: WasmCellType,
    data: Vec<u8>,
    hash: [u8; 32],
    depth: u8,
    is_pruned: bool,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Copy)]
pub enum WasmCellType {
    Ordinary = 0,
    PrunedBranch = 1,
    LibraryReference = 2,
    MerkleProof = 3,
    MerkleUpdate = 4,
}

impl WasmCellType {
    pub fn from(byte: u8) -> Self {
        match byte {
            0 => WasmCellType::Ordinary,
            1 => WasmCellType::PrunedBranch,
            2 => WasmCellType::LibraryReference,
            3 => WasmCellType::MerkleProof,
            4 => WasmCellType::MerkleUpdate,
            _ => WasmCellType::Ordinary, // Default case
        }
    }
}