// ./src/core/zkps/zkp_interface.rs

use crate::core::zkps::plonky2::Plonky2SystemHandle;
use crate::core::zkps::proof::{ProofType, ZkProof};
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofMetadataJS {
    proof_type: i32,
    channel_id: Option<Vec<u8>>,
    created_at: u64,
    verified_at: Option<u64>,
}

#[wasm_bindgen]
impl ProofMetadataJS {
    #[wasm_bindgen(constructor)]
    pub fn new(proof_type: i32, created_at: u64) -> Self {
        Self {
            proof_type,
            channel_id: None,
            created_at,
            verified_at: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn proof_type(&self) -> i32 {
        self.proof_type
    }

    #[wasm_bindgen(getter)]
    pub fn created_at(&self) -> u64 {
        self.created_at
    }

    #[wasm_bindgen(getter)]
    pub fn verified_at(&self) -> Option<u64> {
        self.verified_at
    }

    #[wasm_bindgen(setter)]
    pub fn set_channel_id(&mut self, channel_id: Option<Box<[u8]>>) {
        self.channel_id = channel_id.map(|b| b.to_vec());
    }

    #[wasm_bindgen(setter)]
    pub fn set_verified_at(&mut self, timestamp: Option<u64>) {
        self.verified_at = timestamp;
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofWithMetadataJS {
    proof: ZkProof,
    metadata: ProofMetadataJS,
}

#[wasm_bindgen]
impl ProofWithMetadataJS {
    #[wasm_bindgen(constructor)]
    pub fn new(proof_js: JsValue, metadata_js: JsValue) -> Result<ProofWithMetadataJS, JsValue> {
        let proof: ZkProof = serde_wasm_bindgen::from_value(proof_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize proof: {}", e)))?;
        let metadata: ProofMetadataJS = serde_wasm_bindgen::from_value(metadata_js)
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize metadata: {}", e)))?;

        Ok(ProofWithMetadataJS { proof, metadata })
    }

    #[wasm_bindgen(getter)]
    pub fn proof(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.proof)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof: {}", e)))
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.metadata)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize metadata: {}", e)))
    }
}

#[wasm_bindgen]
pub fn generate_proof(
    old_balance: u64,
    old_nonce: u64,
    new_balance: u64,
    new_nonce: u64,
    transfer_amount: u64,
) -> Result<Uint8Array, JsValue> {
    let plonky2_system_handle = Plonky2SystemHandle::new()?;

    let proof_bytes = plonky2_system_handle.generate_proof_js(
        old_balance,
        old_nonce,
        new_balance,
        new_nonce,
        transfer_amount,
    )?;

    Ok(Uint8Array::from(&proof_bytes[..]))
}

#[wasm_bindgen]
pub fn verify_proof(proof_bytes: &Uint8Array) -> Result<bool, JsValue> {
    let plonky2_system_handle = Plonky2SystemHandle::new()?;
    let proof_vec = proof_bytes.to_vec();
    let result = plonky2_system_handle.verify_proof_js(&proof_vec)?;
    Ok(result)
}

#[wasm_bindgen]
pub fn create_proof_with_metadata(
    proof_bytes: &Uint8Array,
    merkle_root: &Uint8Array,
    public_inputs: &Uint8Array,
    timestamp: u64,
) -> Result<JsValue, JsValue> {
    let proof_vec = proof_bytes.to_vec();
    let merkle_root_array: [u8; 32] = merkle_root
        .to_vec()
        .try_into()
        .map_err(|_| JsValue::from_str("Invalid merkle root length"))?;
    let public_inputs_vec = public_inputs.to_vec();

    let merkle_root_vec = merkle_root_array.to_vec();

    let public_inputs_u64: Vec<u64> = public_inputs_vec
        .chunks_exact(8)
        .map(|chunk| {
            let array: [u8; 8] = chunk.try_into().unwrap();
            u64::from_le_bytes(array)
        })
        .collect();

    let zk_proof = ZkProof {
        proof_data: proof_vec,
        merkle_root: merkle_root_vec,
        public_inputs: public_inputs_u64,
        timestamp,
    };

    let metadata = ProofMetadataJS::new(ProofType::StateTransition as i32, timestamp);

    let proof_js = serde_wasm_bindgen::to_value(&zk_proof)?;
    let metadata_js = serde_wasm_bindgen::to_value(&metadata)?;

    let proof_with_metadata = ProofWithMetadataJS::new(proof_js, metadata_js)?;
    serde_wasm_bindgen::to_value(&proof_with_metadata)
        .map_err(|e| JsValue::from_str(&format!("Failed to serialize bundle: {}", e)))
}

fn to_js_error<E: std::fmt::Display>(error: E) -> JsValue {
    JsValue::from_str(&format!("Error: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_metadata() {
        let timestamp = 12345;
        let metadata = ProofMetadataJS::new(ProofType::StateTransition as i32, timestamp);

        assert_eq!(metadata.created_at(), timestamp);
        assert_eq!(metadata.proof_type(), ProofType::StateTransition as i32);
        assert!(metadata.verified_at().is_none());
    }

    #[test]
    fn test_proof_with_metadata() {
        let timestamp = 12345;
        let zk_proof = ZkProof {
            proof_data: vec![1, 2, 3],
            public_inputs: vec![100, 200],
            merkle_root: vec![0; 32],
            timestamp,
        };

        let metadata = ProofMetadataJS::new(ProofType::StateTransition as i32, timestamp);

        let proof_js = serde_wasm_bindgen::to_value(&zk_proof).unwrap();
        let metadata_js = serde_wasm_bindgen::to_value(&metadata).unwrap();

        let bundle = ProofWithMetadataJS::new(proof_js, metadata_js).unwrap();

        // Test getters
        let proof_result = bundle.proof().unwrap();
        let metadata_result = bundle.metadata().unwrap();

        assert!(proof_result.is_object());
        assert!(metadata_result.is_object());
    }
}
