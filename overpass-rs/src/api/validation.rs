// src/api/validation.rs

use crate::core::types::ovp_types::*;

/// Validates request parameters.
pub fn validate_epoch_id(epoch_id: u64) -> OMResult<()> {
    if epoch_id == 0 {
        Err(OMError::InvalidErrorCode)
    } else {
        Ok(())
    }
}

// Add more validation functions as needed
