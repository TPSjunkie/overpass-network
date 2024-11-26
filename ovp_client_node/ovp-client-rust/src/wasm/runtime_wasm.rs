use wasm_bindgen::prelude::*;
use crate::wasm::types_wasm::WasmCell;

#[wasm_bindgen]
pub struct WasmRuntime {
    memory: Vec<u8>,
    cells: Vec<WasmCell>,
    registers: Vec<i32>,
}

#[wasm_bindgen]
impl WasmRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            memory: vec![0; 65536],  // 64KB of memory
            cells: Vec::new(),
            registers: vec![0; 16],   // 16 general purpose registers
        }
    }

    pub fn add_cell(&mut self, cell: WasmCell) -> Result<usize, JsValue> {
        let index = self.cells.len();
        self.cells.push(cell);
        Ok(index)
    }

    pub fn get_cell(&self, index: usize) -> Result<WasmCell, JsValue> {
        self.cells.get(index)
            .cloned()
            .ok_or_else(|| JsValue::from_str("Cell index out of bounds"))
    }

    pub fn get_memory(&self) -> Vec<u8> {
        self.memory.clone()
    }

    pub fn get_register(&self, index: usize) -> Option<i32> {
        self.registers.get(index).copied()
    }

    pub fn set_register(&mut self, index: usize, value: i32) {
        if index < self.registers.len() {
            self.registers[index] = value;
        }
    }

    pub fn reset(&mut self) {
        self.memory.fill(0);
        self.cells.clear();
        self.registers.fill(0);
    }
}

#[cfg(test)]
mod tests {
    use crate::wasm::WasmCellType;

    use super::*;

    #[test]
    fn test_wasm_cell_creation() {
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32], // Changed from fixed array to Vec
            1,
            false
        );

        assert_eq!(cell.cell_type(), WasmCellType::Ordinary);
        assert_eq!(cell.data(), vec![1, 2, 3]);
        assert_eq!(cell.depth(), 1);
        assert!(!cell.is_pruned());
        assert_eq!(cell.hash().len(), 32);
    }

    #[test]
    fn test_wasm_runtime() {
        let mut runtime = WasmRuntime::new();
        
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32], // Changed from fixed array to Vec
            1,
            false
        );

        let index = runtime.add_cell(cell.clone()).unwrap();
        let retrieved = runtime.get_cell(index).unwrap();

        assert_eq!(retrieved.cell_type(), cell.cell_type());
        assert_eq!(retrieved.data(), cell.data());
        assert_eq!(retrieved.hash(), cell.hash());
        assert_eq!(retrieved.depth(), cell.depth());
        assert_eq!(retrieved.is_pruned(), cell.is_pruned());
    }

    #[test]
    fn test_runtime_registers() {
        let mut runtime = WasmRuntime::new();
        
        // Test register access
        runtime.set_register(0, 42);
        assert_eq!(runtime.get_register(0), Some(42));
        
        // Test out of bounds
        assert_eq!(runtime.get_register(99), None);
        
        // Test reset
        runtime.reset();
        assert_eq!(runtime.get_register(0), Some(0));
    }

    #[test]
    fn test_runtime_memory() {
        let runtime = WasmRuntime::new();
        assert_eq!(runtime.memory.len(), 65536);
        
        // Test memory access
        let memory = runtime.get_memory();
        assert_eq!(memory.len(), 65536);
        assert!(memory.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_cell_operations() {
        let mut runtime = WasmRuntime::new();
        
        // Add cells
        let cell1 = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32],
            1,
            false
        );
        
        let cell2 = WasmCell::new(
            WasmCellType::PrunedBranch,
            vec![4, 5, 6],
            vec![1; 32],
            2,
            true
        );

        let index1 = runtime.add_cell(cell1).unwrap();
        let index2 = runtime.add_cell(cell2).unwrap();

        // Verify cells
        let retrieved1 = runtime.get_cell(index1).unwrap();
        let retrieved2 = runtime.get_cell(index2).unwrap();

        assert_eq!(retrieved1.cell_type(), WasmCellType::Ordinary);
        assert_eq!(retrieved1.data(), vec![1, 2, 3]);
        assert_eq!(retrieved1.depth(), 1);
        assert!(!retrieved1.is_pruned());

        assert_eq!(retrieved2.cell_type(), WasmCellType::PrunedBranch);
        assert_eq!(retrieved2.data(), vec![4, 5, 6]);
        assert_eq!(retrieved2.depth(), 2);
        assert!(retrieved2.is_pruned());

        // Test invalid index
        assert!(runtime.get_cell(99).is_err());

        // Test reset
        runtime.reset();
        assert!(runtime.cells.is_empty());
    }
}