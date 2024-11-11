// src/wasm/conversion_wasm.rs

use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::wasm::types_wasm::{WasmCell, WasmCellType};


#[wasm_bindgen]
pub struct WasmEnv {
    wasm_cells: Arc<Mutex<HashMap<u32, WasmCell>>>,
}

#[wasm_bindgen]
impl WasmEnv {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmEnv, JsValue> {
        Ok(WasmEnv {
            wasm_cells: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    #[wasm_bindgen]
    pub fn get_cell(&self, cell_id: u32) -> Result<JsValue, JsValue> {
        let wasm_cells = self.wasm_cells.lock().unwrap();
        if let Some(cell) = wasm_cells.get(&cell_id) {
            Ok(JsValue::from_serde(cell).unwrap())
        } else {
            Err(JsValue::from_str(&format!("Cell with id {} not found", cell_id)))
        }
    }

    #[wasm_bindgen]
    pub fn add_cell(&self, cell_id: u32, cell_type: u8, data: Vec<u8>) -> Result<(), JsValue> {
        let mut wasm_cells = self.wasm_cells.lock().unwrap();
        let cell = WasmCell {
            cell_type: WasmCellType::from(cell_type),
            data,
        };
        wasm_cells.insert(cell_id, cell);
        Ok(())
    }
}
