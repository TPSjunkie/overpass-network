// ./src/types/intermediate/validation_inter.rs

use crate::core::types::ovp_types::*;
/// Validation for the `IntermediateOp` structure, ensuring that all fields are initialized
/// correctly and are suitable for IC processing within the hierarchical system.
impl IntermediateOp {
    /// Validates the `IntermediateOp` structure to ensure it is properly initialized.
    /// This includes checks for non-zero IDs, valid types, and non-empty operation data.
    pub fn validate(&self) -> OMResult<()> {
        if self.intermediate_op_id == 0 {
            return Err(OMError::InvalidIntermediateOp(
                "Operation ID cannot be zero".into(),
            ));
        }

        if self.intermediate_op_type == 0 {
            return Err(OMError::InvalidIntermediateOp(
                "Operation type cannot be zero".into(),
            ));
        }

        if self.intermediate_op_data.intermediate_op_data.is_empty() {
            return Err(OMError::InvalidIntermediateOp(
                "Operation data is empty".into(),
            ));
        }

        // Delegate validation to IntermediateOpData for data integrity checks
        self.intermediate_op_data.validate()?;

        Ok(())
    }
}

/// Validation for the `IntermediateOpData` structure, verifying that operation data is present
/// and meets required standards for IC processing.
impl IntermediateOp {
    /// Validates the `IntermediateOpData` to ensure it is not empty, avoiding placeholder data.
    pub fn validate(&self) -> OMResult<()> {
        if self.intermediate_op_data.is_empty() {
            return Err(OMError::InvalidIntermediateOpData(
                "Intermediate operation data is empty".into(),
            ));
        }
        if self.intermediate_op_data.len() > 3 {
            return Err(OMError::InvalidIntermediateOpData(
                "Intermediate operation data is invalid".into(),
            ));
        }
        Ok(())
    }
}

#[test]
fn test_validate_intermediate_op_success() {
    // Valid operation with non-zero ID and type, and non-empty data
    let op_data = IntermediateOp::new(vec![1, 2, 3]);
    let op = IntermediateOp {
        intermediate_op_id: 10,
        intermediate_op_type: 20,
        intermediate_op_data: op_data,
    };

    assert!(op.validate().is_ok());
}

#[test]
fn test_validate_intermediate_op_invalid_id() {
    // Operation with zero ID, which should trigger validation error
    let op_data = vec![1, 2, 3];
    let op = IntermediateOp {
        intermediate_op_id: 0,
        intermediate_op_type: 20,
        intermediate_op_data: op_data,
    };

    let result = op.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "InvalidIntermediateOp: Operation ID cannot be zero"
    );
}
#[test]
fn test_validate_intermediate_op_invalid_type() {
    // Operation with zero type, which should trigger validation error
    let op_data = IntermediateOp::new(vec![1, 2, 3]);
    let op = IntermediateOp {
        intermediate_op_id: 10,
        intermediate_op_type: 0,
        intermediate_op_data: op_data,
    };

    let result = op.validate();
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "InvalidIntermediateOp: Operation type cannot be zero"
    );
}

#[test]
fn test_validate_empty_intermediate_op_data() {
    // IntermediateOpData with empty data should fail validation
    let empty_op_data = IntermediateOp::new(vec![]);
    let result = empty_op_data.validate();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "InvalidIntermediateOpData: Intermediate operation data is empty"
    );
}
#[test]
fn test_validate_intermediate_op_data_success() {
    // IntermediateOpData with non-empty data should pass validation
    let op_data = IntermediateOp::new(vec![1, 2, 3]);
    let result = op_data.validate();

    assert!(result.is_ok());
}
#[test]
fn test_validate_intermediate_op_data_invalid_data() {
    // IntermediateOpData with invalid data should fail validation
    let op_data = IntermediateOp::new(vec![1, 2, 3, 4]);
    let result = op_data.validate();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "InvalidIntermediateOpData: Intermediate operation data is invalid"
    );
}
