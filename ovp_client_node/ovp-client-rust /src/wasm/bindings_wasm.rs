use crate::common::types::state_boc::Cell;
use crate::common::types::state_boc::CellType;
use crate::wasm::types_wasm::{WasmCell, WasmCellType};
use wasm_bindgen::prelude::*;
use serde::Serialize;
use serde::ser::SerializeStruct;

#[wasm_bindgen]
pub fn cell_to_boc(cell: &WasmCell) -> Vec<u8> {
    let cell_type = convert_cell_type(cell.get_cell_type());
    let cell_core = Cell::new(
        cell.get_data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        [0; 32],
        None,
    );
    cell_core.serialize_to_vec().unwrap_or_default()
}

#[wasm_bindgen]
pub fn cell_to_json(cell: &WasmCell) -> String {
    let cell_type = convert_cell_type(cell.get_cell_type());
    let cell_core = Cell::new(
        cell.get_data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        [0; 32],
        None,
    );
    serde_json::to_string(&CellWrapper(cell_core)).unwrap_or_default()
}

#[wasm_bindgen]
pub fn cell_to_boc_with_hash(cell: &WasmCell) -> Result<Vec<u8>, JsValue> {
    let cell_type = convert_cell_type(cell.get_cell_type());
    let cell_core = Cell::new(
        cell.get_data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        [0; 32],
        None,
    );
    
    let data = cell_core.serialize_to_vec()
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))?;
        
    Ok(data)
}

#[wasm_bindgen]
pub fn cell_to_json_with_hash(cell: &WasmCell) -> Result<String, JsValue> {
    let cell_type = convert_cell_type(cell.get_cell_type());
    let cell_core = Cell::new(
        cell.get_data().clone(),
        Vec::new(),
        Vec::new(),
        cell_type,
        [0; 32],
        None,
    );
    
    serde_json::to_string(&CellWrapper(cell_core))
        .map_err(|e| JsValue::from_str(&format!("JSON serialization error: {}", e)))
}

fn convert_cell_type(wasm_type: WasmCellType) -> CellType {
    match wasm_type {
        WasmCellType::Ordinary |
        WasmCellType::PrunedBranch |
        WasmCellType::LibraryReference |
        WasmCellType::MerkleProof |
        WasmCellType::MerkleUpdate => CellType::Ordinary,
    }
}

#[derive(Serialize)]
struct CellWrapper(#[serde(serialize_with = "serialize_cell")] Cell);

fn serialize_cell<S>(cell: &Cell, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut state = serializer.serialize_struct("Cell", 1)?;
    state.serialize_field("data", &format!("{:?}", cell))?;
    state.end()
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

    // Mock WasmCell for testing
    struct MockWasmCell {
        cell_type: WasmCellType,
        data: Vec<u8>,
    }

    impl WasmCell for MockWasmCell {
        fn get_cell_type(&self) -> WasmCellType {
            self.cell_type.clone()
        }

        fn get_data(&self) -> &Vec<u8> {
            &self.data
        }
    }

    #[test]
    fn test_cell_to_boc() {
        let mock_cell = MockWasmCell {
            cell_type: WasmCellType::Ordinary,
            data: vec![1, 2, 3],
        };

        let result = cell_to_boc(&mock_cell);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_cell_to_json() {
        let mock_cell = MockWasmCell {
            cell_type: WasmCellType::Ordinary,
            data: vec![1, 2, 3],
        };

        let result = cell_to_json(&mock_cell);
        assert!(!result.is_empty());
        assert!(serde_json::from_str::<serde_json::Value>(&result).is_ok());
    }

    #[test]
    fn test_cell_to_boc_with_hash() {
        let mock_cell = MockWasmCell {
            cell_type: WasmCellType::Ordinary,
            data: vec![1, 2, 3],
        };

        let result = cell_to_boc_with_hash(&mock_cell);
        assert!(result.is_ok());
        assert!(!result.unwrap().is_empty());
    }

    #[test]
    fn test_cell_to_json_with_hash() {
        let mock_cell = MockWasmCell {
            cell_type: WasmCellType::Ordinary,
            data: vec![1, 2, 3],
        };

        let result = cell_to_json_with_hash(&mock_cell);
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(!json.is_empty());
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }
}