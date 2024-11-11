// src/validation/validation_node.rs

use wasm_bindgen_test::__rt::node::Node;

use crate::core::types::ovp_types::*;
pub trait NodeValidation {
    fn validate(&self) -> Result<(), OMError>;
}

impl NodeValidation for Node {
    /// Validates the node's key and value are non-empty.
    fn validate(&self) -> Result<(), OMError> {
        if self.key.is_empty() {
            Err(OMError::InvalidNodeData)
        } else if self.value.is_empty() {
            Err(OMError::InvalidNodeData)
        } else {
            Ok(())
        }
    }
}
