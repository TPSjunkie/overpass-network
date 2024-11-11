// ./src/core/types/error/conversion_error.rs

use crate::core::types::ovp_types::*;

impl OMError {
    pub fn to_bytes(&self) -> OMResult<Vec<u8>> {
        let mut bytes = Vec::new();
        bytes.extend(self.to_u8().to_le_bytes());
        Ok(bytes)
    }
}

impl OMError {
    pub fn from_bytes(bytes: &[u8]) -> OMResult<Vec<u8>> {
        if bytes.is_empty() {
            return Err(OMError::InvalidErrorCode);
        }
        let error_code = bytes[0];
        match error_code {
            0 => Ok(bytes[1..].to_vec()),
            _ => Err(OMError::InvalidErrorCode),
        }
    }
}
