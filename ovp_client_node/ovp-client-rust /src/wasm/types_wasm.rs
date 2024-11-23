use wasm_bindgen::prelude::wasm_bindgen;
use crate::common::types::state_boc::Cell;

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





impl WasmCell {
    pub fn new(cell_type: WasmCellType, data: Vec<u8>, hash: [u8; 32], depth: u8, is_pruned: bool) -> Self {
        Self {
            cell_type,
            data,
            hash,
            depth,
            is_pruned,
        }
    }

    pub fn get_cell_type(&self) -> WasmCellType {
        self.cell_type
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_hash(&self) -> &[u8; 32] {
        &self.hash
    }

    pub fn get_depth(&self) -> u8 {
        self.depth
    }

    pub fn is_pruned(&self) -> bool {
        self.is_pruned
    }
}


