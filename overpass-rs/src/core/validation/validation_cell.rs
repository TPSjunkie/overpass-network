// src/validation/validation_cell.rs

use crate::core::types::ovp_types::*;

impl Cell {
    /// Validates the cell data.
    pub fn validate(&self) -> OMResult<()> {
        if self.get_data().is_empty() {
            Err(OMError::InvalidData)
        } else {
            Ok(())
        }
    }
}
