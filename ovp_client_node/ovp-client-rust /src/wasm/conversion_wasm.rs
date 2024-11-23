// src/wasm/conversion_wasm.rs
use crate::wasm::types_wasm::{WasmCell, WasmCellType};
use crate::wasm::bindings_wasm::cell_to_boc;
use crate::wasm::bindings_wasm::cell_to_json;
use crate::wasm::bindings_wasm::cell_to_boc_with_hash;
use crate::wasm::bindings_wasm::cell_to_json_with_hash;
/// Conversion functions between Rust and WASM types.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WasmRuntime {
    inner: Vec<i32>,
    memory: Vec<u8>,
}

#[wasm_bindgen]
impl WasmRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: vec![0; 4],
            memory: Vec::new(),
        }
    }

    pub fn get_cell(&self) -> WasmCell {
        let cell_type = match self.inner[0] {
            0 => WasmCellType::Ordinary,
            1 => WasmCellType::PrunedBranch,
            2 => WasmCellType::LibraryReference,
            3 => WasmCellType::MerkleProof,
            4 => WasmCellType::MerkleUpdate,
            _ => WasmCellType::Ordinary, // Default case
        };
        let data = self.memory[..self.inner[1] as usize].to_vec();
        let hash = self.memory[self.inner[1] as usize..(self.inner[1] + 32) as usize].try_into().unwrap();
        let depth = self.inner[2] as u8;
        let is_pruned = self.inner[3] != 0;
        WasmCell::new(cell_type, data, hash, depth, is_pruned)
    }

    pub fn set_cell(&mut self, cell: WasmCell) {
        let cell_type = match cell.get_cell_type() {
            WasmCellType::Ordinary => 0,
            WasmCellType::PrunedBranch => 1,
            WasmCellType::LibraryReference => 2,
            WasmCellType::MerkleProof => 3,
            WasmCellType::MerkleUpdate => 4,
        };
        self.inner[0] = cell_type;
        self.inner[1] = cell.get_data().len() as i32;
        self.inner[2] = cell.get_depth() as i32;
        self.inner[3] = cell.is_pruned() as i32;
        let start_index = self.inner[1] as usize;
        let end_index = start_index + cell.get_data().len();
        self.memory.resize(end_index + 32, 0);
        self.memory[start_index..end_index].copy_from_slice(cell.get_data());
        self.memory[end_index..end_index + 32].copy_from_slice(cell.get_hash());
    }

    pub fn get_cell_as_boc(&self) -> Vec<u8> {
        cell_to_boc(&self.get_cell())
    }

    pub fn get_cell_as_json(&self) -> String {
        cell_to_json(&self.get_cell())
    }

    pub fn get_cell_as_boc_with_hash(&self) -> Vec<u8> {
        cell_to_boc_with_hash(&self.get_cell())
    }

    pub fn get_cell_as_json_with_hash(&self) -> String {
        cell_to_json_with_hash(&self.get_cell())
    }
}