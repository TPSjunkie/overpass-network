// src/wasm/types_wasm.rs
use wasm_bindgen::prelude::wasm_bindgen;
use crate::core::types::boc::Cell;

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

// Implement conversion from Rust types to Wasm types
impl From<WasmCell> for crate::core::types::boc::Cell {
    fn from(wasm_cell: WasmCell) -> Self {
        let cell_type = match wasm_cell.get_cell_type() {
            WasmCellType::Ordinary => crate::core::types::boc::CellType::Ordinary,
            WasmCellType::PrunedBranch => crate::core::types::boc::CellType::PrunedBranch,
            WasmCellType::LibraryReference => crate::core::types::boc::CellType::LibraryReference,
            WasmCellType::MerkleProof => crate::core::types::boc::CellType::MerkleProof,
            WasmCellType::MerkleUpdate => crate::core::types::boc::CellType::MerkleUpdate,
        };
        let cell_core = crate::core::types::boc::Cell::new(
            wasm_cell.get_data().clone(),
            Vec::new(),
            cell_type,
            wasm_cell.get_hash().clone(),
            None,
        );
        cell_core
    }
}   

impl From<crate::core::types::boc::Cell> for WasmCell {
    fn from(cell_core: crate::core::types::boc::Cell) -> Self {
        let cell_type = match cell_core.get_cell_type() {
            crate::core::types::boc::CellType::Ordinary => WasmCellType::Ordinary,
            crate::core::types::boc::CellType::PrunedBranch => WasmCellType::PrunedBranch,
            crate::core::types::boc::CellType::LibraryReference => WasmCellType::LibraryReference,
            crate::core::types::boc::CellType::MerkleProof => WasmCellType::MerkleProof,
            crate::core::types::boc::CellType::MerkleUpdate => WasmCellType::MerkleUpdate,
        };
        Self {
            cell_type,
            data: cell_core.get_data().clone(),
            hash: cell_core.get_hash().clone(),
            depth: cell_core.get_depth(),
            is_pruned: cell_core.is_pruned(),
        }
    }
}   






#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_to_wasm_cell() {
        let cell = crate::core::types::boc::Cell::new(
            vec![1, 2, 3],
            vec![4, 5, 6],
            crate::core::types::boc::CellType::Ordinary,
            [0; 32],
            None,
        );
        let wasm_cell = WasmCell::from(cell);
        assert_eq!(wasm_cell.get_data(), vec![1, 2, 3]);
        assert_eq!(wasm_cell.get_hash(), [0; 32]);
        assert_eq!(wasm_cell.get_depth(), 0);
        assert_eq!(wasm_cell.is_pruned(), false);
    }

    #[test]
    fn test_wasm_cell_to_cell() {
        let wasm_cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            [0; 32],
            0,
            false,
        );
        let cell = wasm_cell.into();
        assert_eq!(cell.get_data(), vec![1, 2, 3]);
        assert_eq!(cell.get_hash(), [0; 32]);
        assert_eq!(cell.get_depth(), 0);
        assert_eq!(cell.is_pruned(), false);
    }
}   

