use crate::wasm::types_wasm::{WasmCell, WasmCellType, WasmCellTrait};
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

    pub fn get_cell(&self) -> Result<WasmCell, JsValue> {
        if self.inner[1] as usize + 32 > self.memory.len() {
            return Err(JsValue::from_str("Invalid memory layout"));
        }

        let cell_type = match self.inner[0] {
            0 => WasmCellType::Ordinary,
            1 => WasmCellType::PrunedBranch,
            2 => WasmCellType::LibraryReference,
            3 => WasmCellType::MerkleProof,
            4 => WasmCellType::MerkleUpdate,
            _ => return Err(JsValue::from_str("Invalid cell type")),
        };

        let data_len = self.inner[1] as usize;
        let data = self.memory[..data_len].to_vec();
        
        let mut hash = Vec::with_capacity(32);
        hash.extend_from_slice(&self.memory[data_len..data_len + 32]);

        let depth = self.inner[2] as u8;
        let is_pruned = self.inner[3] != 0;

        Ok(WasmCell::new(cell_type, data, hash, depth, is_pruned))
    }

    pub fn set_cell(&mut self, cell: &WasmCell) -> Result<(), JsValue> {
        // Set cell type
        self.inner[0] = match cell.cell_type() {
            WasmCellType::Ordinary => 0,
            WasmCellType::PrunedBranch => 1,
            WasmCellType::LibraryReference => 2,
            WasmCellType::MerkleProof => 3,
            WasmCellType::MerkleUpdate => 4,
        };

        // Set data length and depth
        let data = cell.data();
        self.inner[1] = data.len() as i32;
        self.inner[2] = cell.depth() as i32;
        self.inner[3] = cell.is_pruned() as i32;

        // Resize memory if needed
        let required_size = data.len() + 32;
        if self.memory.len() < required_size {
            self.memory.resize(required_size, 0);
        }

        // Copy data and hash
        let start_index = 0;
        let end_index = data.len();
        self.memory[start_index..end_index].copy_from_slice(&data);
        self.memory[end_index..end_index + 32].copy_from_slice(&cell.hash());

        Ok(())
    }

    pub fn get_cell_as_boc(&self) -> Result<Vec<u8>, JsValue> {
        let cell = self.get_cell()?;
        crate::wasm::bindings_wasm::cell_to_boc(&cell)
    }

    pub fn get_cell_as_json(&self) -> Result<String, JsValue> {
        let cell = self.get_cell()?;
        Ok(crate::wasm::bindings_wasm::cell_to_json(&cell))
    }

    pub fn get_cell_as_boc_with_hash(&self) -> Result<Vec<u8>, JsValue> {
        let cell = self.get_cell()?;
        crate::wasm::bindings_wasm::cell_to_boc_with_hash(&cell)
    }

    pub fn get_cell_as_json_with_hash(&self) -> Result<String, JsValue> {
        let cell = self.get_cell()?;
        crate::wasm::bindings_wasm::cell_to_json_with_hash(&cell)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_runtime() {
        let mut runtime = WasmRuntime::new();
        assert_eq!(runtime.inner.len(), 4);
        assert!(runtime.memory.is_empty());
    }

    #[test]
    fn test_set_get_cell() {
        let mut runtime = WasmRuntime::new();
        
        let cell = WasmCell::new(
            WasmCellType::Ordinary, 
            vec![1, 2, 3],
            vec![0; 32], // Changed from fixed array to Vec
            1,
            false
        );

        assert!(runtime.set_cell(&cell).is_ok());
        
        let retrieved = runtime.get_cell().unwrap();
        assert_eq!(retrieved.cell_type(), cell.cell_type());
        assert_eq!(retrieved.data(), cell.data());
        assert_eq!(retrieved.hash(), cell.hash());
        assert_eq!(retrieved.depth(), cell.depth());
        assert_eq!(retrieved.is_pruned(), cell.is_pruned());
    }

    #[test]
    fn test_cell_conversions() {
        let mut runtime = WasmRuntime::new();
        
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32], // Changed from fixed array to Vec
            1,
            false
        );

        runtime.set_cell(&cell).unwrap();

        assert!(runtime.get_cell_as_boc().is_ok());
        assert!(runtime.get_cell_as_json().is_ok());
        assert!(runtime.get_cell_as_boc_with_hash().is_ok());
        assert!(runtime.get_cell_as_json_with_hash().is_ok());
    }

    #[test]
    fn test_invalid_memory() {
        let mut runtime = WasmRuntime::new();
        runtime.inner[1] = 100; // Set invalid data length
        assert!(runtime.get_cell().is_err());
    }

    #[test]
    fn test_hash_validation() {
        let mut runtime = WasmRuntime::new();
        
        // Test with valid hash
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![1; 32],
            1,
            false
        );

        assert!(runtime.set_cell(&cell).is_ok());
        let retrieved = runtime.get_cell().unwrap();
        assert_eq!(retrieved.hash().len(), 32);
        assert_eq!(retrieved.hash(), cell.hash());
        
        // Test with invalid memory layout
        runtime.memory.truncate(16); // Make memory too small
        assert!(runtime.get_cell().is_err());
    }

    #[test]
    fn test_cell_type_conversion() {
        let mut runtime = WasmRuntime::new();
        
        for (i, cell_type) in [
            WasmCellType::Ordinary,
            WasmCellType::PrunedBranch,
            WasmCellType::LibraryReference,
            WasmCellType::MerkleProof,
            WasmCellType::MerkleUpdate,
        ].iter().enumerate() {
            let cell = WasmCell::new(
                *cell_type,
                vec![1, 2, 3],
                vec![0; 32],
                1,
                false
            );
            
            runtime.set_cell(&cell).unwrap();
            assert_eq!(runtime.inner[0], i as i32);
            
            let retrieved = runtime.get_cell().unwrap();
            assert_eq!(retrieved.cell_type(), *cell_type);
        }
    }
}