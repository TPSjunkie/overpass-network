// src/validation/validation_common.rs
use crate::core::types::ovp_types::*;
impl CommonMsgInfo {
    /// Validates the `CommonMsgInfo` data.
    pub fn validate(&self) -> OMResult<()> {
        if self.data.data.is_empty() {
            Err(OMError::InvalidData)
        } else {
            Ok(())
        }
    }
}
