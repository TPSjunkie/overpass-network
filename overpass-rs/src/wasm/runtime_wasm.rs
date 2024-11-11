// src/wasm/runtime_wasm.rs

use crate::core::{storage_node::storage_node::StorageNode, types::ovp_types::*};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn start_storage_node() -> Result<(), JsValue> {
    let storage_node = StorageNode::new(
        "storage_node_1".to_string(),
        "127.0.0.1:8080".to_string(),
        vec!["127.0.0.1:8081".to_string(), "127.0.0.1:8082".to_string()],
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))?;

    storage_node
        .start()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    storage_node
        .join_network()
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(())
}
