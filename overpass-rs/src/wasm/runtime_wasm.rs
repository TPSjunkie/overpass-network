// src/wasm/runtime_wasm.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn start_storage_node() -> Result<(), JsValue> {
    // Initialize the storage node  
    let storage_node = crate::core::storage_node::storage_node_contract::StorageNode::new(
        "storage_node_1".to_string(),
        "127.0.0.1:8080".to_string(),
        vec!["127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string()],
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .await
    .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Start the storage node
    storage_node
        .start()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Join the network
    storage_node
        .join_network()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Get the root hash
    let root_hash = storage_node.get_root_hash();

    // Get the root cell
    let root_cell = storage_node.get_root_cell();

    // Get the root cell as a BOC
    let root_cell_boc = storage_node.get_root_cell_boc();

    // Get the root cell as a JSON
    let root_cell_json = storage_node.get_root_cell_json();

    // Get the root cell as a BOC with hash
    let root_cell_boc_with_hash = storage_node.get_root_cell_boc_with_hash();

    // Get the root cell as a JSON with hash
    let root_cell_json_with_hash = storage_node.get_root_cell_json_with_hash();

    // Get the root cell as a WasmCell    
    let root_cell_wasm = storage_node.get_root_cell_wasm();

    Ok(())
}