use crate::common::types::state_boc::Cell;
use crate::common::types::state_boc::CellType;
use crate::wasm::types_wasm::{WasmCell, WasmCellType};
use wasm_bindgen::prelude::*;
use serde::Serialize;
use serde::ser::SerializeStruct;

#[wasm_bindgen]
pub fn cell_to_boc(cell: &WasmCell) -> Result<Vec<u8>, JsValue> {
    let cell_type = convert_cell_type(cell.cell_type());
    let cell_core = Cell::new(
        cell.data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        cell.hash().try_into().map_err(|_| JsValue::from_str("Invalid hash length"))?,
        None,
    );
    
    bincode::serialize(&CellWrapper(&cell_core))
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn cell_to_json(cell: &WasmCell) -> String {
    let cell_type = convert_cell_type(cell.cell_type());
    let cell_core = Cell::new(
        cell.data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        cell.hash().try_into().unwrap_or([0; 32]),
        None,
    );
    serde_json::to_string(&CellWrapper(&cell_core)).unwrap_or_default()
}

#[wasm_bindgen]
pub fn cell_to_boc_with_hash(cell: &WasmCell) -> Result<Vec<u8>, JsValue> {
    let cell_type = convert_cell_type(cell.cell_type());
    let cell_core = Cell::new(
        cell.data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        cell.hash().try_into().map_err(|_| JsValue::from_str("Invalid hash length"))?,
        None,
    );
    
    bincode::serialize(&CellWrapper(&cell_core))
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn cell_to_json_with_hash(cell: &WasmCell) -> Result<String, JsValue> {
    let cell_type = convert_cell_type(cell.cell_type());
    let cell_core = Cell::new(
        cell.data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        cell.hash().try_into().map_err(|_| JsValue::from_str("Invalid hash length"))?,
        None,
    );
    
    serde_json::to_string(&CellWrapper(&cell_core))
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

fn convert_cell_type(wasm_type: WasmCellType) -> CellType {
    match wasm_type {
        WasmCellType::Ordinary => CellType::Ordinary,
        WasmCellType::PrunedBranch => CellType::Ordinary,
        WasmCellType::LibraryReference => CellType::Ordinary,
        WasmCellType::MerkleProof => CellType::Ordinary,
        WasmCellType::MerkleUpdate => CellType::Ordinary,
    }
}

#[derive(Serialize)]
struct CellWrapper<'a>(&'a Cell);

impl Serialize for Cell {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Cell", 1)?;
        state.serialize_field("data", &format!("{:?}", self))?;
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_cell_type() {
        assert!(matches!(
            convert_cell_type(WasmCellType::Ordinary),
            CellType::Ordinary
        ));
        assert!(matches!(
            convert_cell_type(WasmCellType::PrunedBranch),
            CellType::Ordinary
        ));
        assert!(matches!(
            convert_cell_type(WasmCellType::LibraryReference),
            CellType::Ordinary
        ));
        assert!(matches!(
            convert_cell_type(WasmCellType::MerkleProof),
            CellType::Ordinary
        ));
        assert!(matches!(
            convert_cell_type(WasmCellType::MerkleUpdate),
            CellType::Ordinary
        ));
    }

    #[test]
    fn test_cell_to_boc() {
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32],
            1,
            false
        );

        let result = cell_to_boc(&cell);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_cell_to_json() {
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32],
            1,
            false
        );

        let result = cell_to_json(&cell);
        assert!(!result.is_empty());
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }

    #[test]
    fn test_cell_to_boc_with_hash() {
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32],
            1,
            false
        );

        let result = cell_to_boc_with_hash(&cell);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_cell_to_json_with_hash() {
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 32],
            1,
            false
        );

        let result = cell_to_json_with_hash(&cell);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(!json.is_empty());
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }

    #[test]
    fn test_serialization_errors() {
        // Create an invalid cell to test error handling
        let cell = WasmCell::new(
            WasmCellType::Ordinary,
            vec![1, 2, 3],
            vec![0; 16], // Invalid hash length
            1,
            false
        );

        assert!(cell_to_boc(&cell).is_err());
        assert_eq!(cell_to_json(&cell), "{}"); // Default on error
        assert!(cell_to_boc_with_hash(&cell).is_err());
        assert!(cell_to_json_with_hash(&cell).is_err());
    }

    #[test]
    fn test_hash_handling() {
        // Test with different hash lengths
        let test_cases = vec![
            (vec![0; 31], false), // Too short
            (vec![0; 32], true),  // Correct length
            (vec![0; 33], false), // Too long
        ];

        for (hash, should_succeed) in test_cases {
            let cell = WasmCell::new(
                WasmCellType::Ordinary,
                vec![1, 2, 3],
                hash,
                1,
                false
            );

            let boc_result = cell_to_boc(&cell);
            assert_eq!(boc_result.is_ok(), should_succeed);
        }
    }
}