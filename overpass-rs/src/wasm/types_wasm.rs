// src/wasm/types_wasm.rs

use wasm_bindgen::prelude::*;

/// Represents a cell in the WASM environment.
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WasmCell {
    pub cell_type: WasmCellType,
    pub data: Vec<u8>,
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
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
