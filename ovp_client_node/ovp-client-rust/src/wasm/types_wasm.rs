use wasm_bindgen::prelude::*;


#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct WasmCell {
    cell_type: WasmCellType,
    data: Vec<u8>,
    hash: Vec<u8>,  // Changed from [u8; 32] to Vec<u8> for WASM compatibility
    depth: u8,
    is_pruned: bool,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum WasmCellType {
    Ordinary = 0,
    PrunedBranch = 1,
    LibraryReference = 2,
    MerkleProof = 3,
    MerkleUpdate = 4,
}

#[wasm_bindgen]
impl WasmCell {
    #[wasm_bindgen(constructor)]
    pub fn new(cell_type: WasmCellType, data: Vec<u8>, hash: Vec<u8>, depth: u8, is_pruned: bool) -> Self {
        let hash = if hash.len() == 32 {
            hash
        } else {
            vec![0; 32]
        };
        
        Self {
            cell_type,
            data,
            hash,
            depth,
            is_pruned,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn cell_type(&self) -> WasmCellType {
        self.cell_type
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn hash(&self) -> Vec<u8> {
        self.hash.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn depth(&self) -> u8 {
        self.depth
    }

    #[wasm_bindgen(getter)]
    pub fn is_pruned(&self) -> bool {
        self.is_pruned
    }
}

pub trait WasmCellTrait {
    fn cell_type(&self) -> WasmCellType;
    fn data(&self) -> &Vec<u8>;
    fn hash(&self) -> &Vec<u8>;
    fn depth(&self) -> u8;
    fn is_pruned(&self) -> bool;
}

impl WasmCellTrait for WasmCell {
    fn cell_type(&self) -> WasmCellType {
        self.cell_type
    }

    fn data(&self) -> &Vec<u8> {
        &self.data
    }

    fn hash(&self) -> &Vec<u8> {
        &self.hash
    }

    fn depth(&self) -> u8 {
        self.depth
    }

    fn is_pruned(&self) -> bool {
        self.is_pruned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_cell_creation() {
        let cell = WasmCell::new(
            WasmCellType::Ordinary, 
            vec![1, 2, 3],
            vec![0; 32],
            1,
            false
        );

        assert_eq!(cell.cell_type(), WasmCellType::Ordinary);
        assert_eq!(cell.data(), vec![1, 2, 3]);
        assert_eq!(cell.depth(), 1);
        assert!(!cell.is_pruned());
    }

    #[test]
    fn test_wasm_cell_trait() {
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32],
            1,
            false
        );

        assert_eq!(WasmCellTrait::cell_type(&cell), WasmCellType::Ordinary);
        assert_eq!(WasmCellTrait::data(&cell), &vec![1, 2, 3]);
        assert_eq!(WasmCellTrait::hash(&cell), &vec![0; 32]);
        assert_eq!(WasmCellTrait::depth(&cell), 1);
        assert!(!WasmCellTrait::is_pruned(&cell));
    }

    #[test]
    fn test_hash_validation() {
        // Test with correct hash length
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![1; 32],
            1,
            false
        );
        assert_eq!(cell.hash().len(), 32);

        // Test with incorrect hash length
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![1; 16], // Too short
            1,
            false
        );
        assert_eq!(cell.hash().len(), 32); // Should be normalized to 32 bytes
        assert_eq!(cell.hash(), vec![0; 32]); // Should be zeroed
    }
}