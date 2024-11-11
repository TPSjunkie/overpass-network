// ./src/types/intermediate/conversion_inter.rs

use crate::core::types::ovp_types::{IntermediateOp, OMError, OMResult};

use super::IntermediateContractRoot;

/// Extension for `IntermediateOp` to convert its fields, including the IC root, to bytes.
impl IntermediateOp {
    /// Serializes the `IntermediateOp` into a byte vector, including IC's root.
    pub fn to_bytes(&self, ic_root: &IntermediateContractRoot) -> OMResult<Vec<u8>> {
        let mut bytes = Vec::new();

        // Serialize basic fields
        bytes.extend(self.intermediate_op_id.to_le_bytes()); // ID
        bytes.extend(self.intermediate_op_type.to_le_bytes()); // Operation type
        bytes.extend(self.intermediate_op_data.to_bytes()?); // Operation data

        // Append IC root hash for hierarchical alignment with the root contract
        bytes.extend(ic_root.hash);

        Ok(bytes)
    }
}

/// Extension for `IntermediateOpData` to convert the operation data to bytes.
impl IntermediateOp {
    /// Serializes `IntermediateOpData` into a byte vector.
    pub fn to_bytes(&self) -> OMResult<Vec<u8>> {
        if self.intermediate_op_data.is_empty() {
            return Err(OMError::EmptyIntermediateOpData);
        }
        Ok(self.intermediate_op_data.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::ovp_types::*;

    #[test]
    fn test_to_bytes() {
        let op_data = IntermediateOp::new(vec![1, 2, 3, 4]);
        let intermediate_op = IntermediateOp {
            intermediate_op_id: 1,
            intermediate_op_type: 2,
            intermediate_op_data: op_data.clone(),
        };
        let ic_root = IntermediateContract {
            hash: [0xAA; 32],
            pending_channels: todo!(),
            closing_channels: todo!(),
            rebalance_queue: todo!(),
            wallet_states: todo!(),
            pending_updates: todo!(),
            storage_nodes: todo!(),
            last_root_submission: todo!(),
        }; // Sample IC root hash

        let bytes = intermediate_op
            .to_bytes(&ic_root)
            .expect("Serialization failed");

        // Verify basic fields
        assert_eq!(&bytes[0..8], &1u64.to_le_bytes()); // ID
        assert_eq!(&bytes[8..16], &2u64.to_le_bytes()); // Operation type
        assert_eq!(&bytes[16..20], &op_data.to_bytes().unwrap()[..]); // Operation data
        assert_eq!(&bytes[20..52], &ic_root.hash); // IC root hash
    }

    #[test]
    fn test_empty_op_data() {
        let empty_data = IntermediateOp::new(vec![]);
        let result = empty_data.to_bytes();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), OMError::EmptyIntermediateOpData);
    }
}

fn main() {
    let intermediate_op = IntermediateOp {
        intermediate_op_id: 1,
        intermediate_op_type: 2,
        intermediate_op_data: IntermediateOp::new(vec![1, 2, 3]),
    };
    let ic_root = IntermediateContractRoot { hash: [0xAB; 32] }; // Sample IC root hash

    let bytes = intermediate_op.to_bytes(&ic_root).unwrap();
    println!("Bytes: {:?}", bytes);
}
