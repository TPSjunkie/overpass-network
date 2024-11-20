// src/wasm/conversion_wasm.rs
use crate::wasm::cell_to_json_with_hash;
use crate::wasm::cell_to_boc_with_hash;
use crate::wasm::cell_to_json;
use crate::wasm::cell_to_boc;
use crate::wasm::WasmCell;
use crate::wasm::Runtime;
use wasm_bindgen_test::__rt::detect::Runtime as OtherRuntime;
/// Conversion functions between Rust and WASM types.
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Runtime {
    pub fn get_cell(&self) -> WasmCell {
        let cell_type = match self.get_register(0) {
            Some(0) => WasmCell::Ordinary,
            Some(1) => WasmCell::PrunedBranch,
            Some(2) => WasmCell::LibraryReference,
            Some(3) => WasmCell::MerkleProof,
            Some(4) => WasmCell::MerkleUpdate,
            _ => WasmCell::Ordinary, // Default case
        };
        let data = self.get_memory()[..self.get_register(1) as usize].to_vec();
        let hash = self.get_memory()[self.get_register(1) as usize..(self.get_register(1) + 32) as usize].try_into().unwrap();
        let depth = self.get_register(2);
        let is_pruned = self.get_register(3) != 0;
        WasmCell::new(cell_type, data, hash, depth, is_pruned)
    }

    pub fn set_cell(&mut self, cell: WasmCell) {
        let cell_type = match cell.get_cell_type() {
            WasmCell::Ordinary => 0,
            WasmCell::PrunedBranch => 1,
            WasmCell::LibraryReference => 2,
            WasmCell::MerkleProof => 3,
            WasmCell::MerkleUpdate => 4,
        };
        self.set_register(0, cell_type as i32);
        self.set_register(1, cell.get_data().len() as i32);
        self.set_register(2, cell.get_depth() as i32);
        self.set_register(3, cell.is_pruned() as i32);
        let start_index = self.get_register(1).unwrap_or(0) as usize;
        let end_index = start_index + cell.get_data().len();
        self.memory[start_index..end_index].copy_from_slice(cell.get_data());
        self.memory[start_index..start_index + 32].copy_from_slice(cell.get_hash());
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