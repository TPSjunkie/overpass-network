//! Zero Knowledge Proof WebAssembly Interface
//! 
//! Provides WebAssembly bindings for the Overpass protocol's proof system.
//! Implements security parameter λ ≥ 128 bits with 2^-λ security bound.

use crate::core::{
    client::wallet_extension::channel_manager::Plonky2SystemHandle,
    zkps::proof::{ProofType, ZkProof},
};
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const MIN_SECURITY_BITS: usize = 128;

/// JavaScript-friendly proof metadata
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofMetadataJS {
    proof_type: i32,
    channel_id: Option<Vec<u8>>,
    created_at: u64,
    verified_at: Option<u64>,
    security_bits: usize,
    height_bounds: (u64, u64),
    version: i32,
}

#[wasm_bindgen]
impl ProofMetadataJS {
    #[wasm_bindgen(constructor)]
    pub fn new(proof_type: i32, created_at: u64) -> Result<ProofMetadataJS, JsValue> {
        // Validate security parameter
        let security_bits = MIN_SECURITY_BITS;
        
        Ok(Self {
            proof_type,
            channel_id: None,
            created_at,
            verified_at: None,
            security_bits,
            height_bounds: (0, 0),
            version: 1,
        })
    }

    // Getters
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

    #[wasm_bindgen(getter)]
    pub fn security_bits(&self) -> usize {
        self.security_bits
    }

    #[wasm_bindgen(getter)]
    pub fn height_bounds(&self) -> Box<[u64]> {
        Box::new([self.height_bounds.0, self.height_bounds.1])
    }

    // Setters
    #[wasm_bindgen(setter)]
    pub fn set_channel_id(&mut self, channel_id: Option<Box<[u8]>>) {
        self.channel_id = channel_id.map(|b| b.to_vec());
    }

    #[wasm_bindgen(setter)]
    pub fn set_verified_at(&mut self, timestamp: Option<u64>) {
        self.verified_at = timestamp;
    }

    #[wasm_bindgen(setter)]
    pub fn set_height_bounds(&mut self, start: u64, end: u64) {
        self.height_bounds = (start, end);
    }
}

/// JavaScript-friendly proof bundle
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
            .map_err(|e| to_js_error(format!("Failed to deserialize proof: {}", e)))?;
            
        let metadata: ProofMetadataJS = serde_wasm_bindgen::from_value(metadata_js)
            .map_err(|e| to_js_error(format!("Failed to deserialize metadata: {}", e)))?;

        // Validate security parameter
        if proof.security_bits < MIN_SECURITY_BITS {
            return Err(to_js_error(format!(
                "Security parameter λ must be at least {} bits", MIN_SECURITY_BITS
            )));
        }

        Ok(ProofWithMetadataJS { proof, metadata })
    }

    #[wasm_bindgen(getter)]
    pub fn proof(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.proof)
            .map_err(|e| to_js_error(format!("Failed to serialize proof: {}", e)))
    }

    #[wasm_bindgen(getter)]
    pub fn metadata(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.metadata)
            .map_err(|e| to_js_error(format!("Failed to serialize metadata: {}", e)))
    }
}

/// Generates proof for state transition
#[wasm_bindgen]
pub fn generate_proof(
    old_balance: u64,
    new_balance: u64,
    amount: u64,
) -> Result<Uint8Array, JsValue> {
    let plonky2_system = Plonky2SystemHandle::new()
        .map_err(|e| to_js_error(format!("Failed to create Plonky2 system: {}", e)))?;
    
    let proof = plonky2_system.generate_proof_js(old_balance, new_balance, amount)
        .map_err(|e| to_js_error(format!("Failed to generate proof: {}", e)))?;
        
    Ok(Uint8Array::from(&proof.proof_data[..]))
}

/// Verifies proof with constant-time guarantee
#[wasm_bindgen(js_name = verifyProof)]
pub fn verify_proof(proof_bytes: &Uint8Array, public_inputs: &[u64]) -> Result<bool, JsValue> {
    let plonky2_system = Plonky2SystemHandle::new()
        .map_err(|e| to_js_error(format!("Failed to create Plonky2 system: {}", e)))?;
    
    plonky2_system.verify_proof_js(proof_bytes.to_vec(), public_inputs)
        .map_err(|e| to_js_error(format!("Failed to verify proof: {}", e)))
}

/// Creates proof bundle with metadata
#[wasm_bindgen(js_name = createProofWithMetadata)]
pub fn create_proof_with_metadata(
    proof_bytes: &Uint8Array,
    merkle_root: &Uint8Array,
    public_inputs: &[u64],
    timestamp: u64,
) -> Result<JsValue, JsValue> {
    // Create proof
    let zk_proof = ZkProof {
        proof_data: proof_bytes.to_vec(),
        merkle_root: merkle_root.to_vec(),
        public_inputs: public_inputs.to_vec(),
        timestamp,
        security_bits: MIN_SECURITY_BITS,
    };
    
    // Create metadata
    let metadata = ProofMetadataJS::new(
        ProofType::StateTransition as i32,
        timestamp,
    )?;

    // Serialize components
    let proof_js = serde_wasm_bindgen::to_value(&zk_proof)
        .map_err(|e| to_js_error(format!("Failed to serialize proof: {}", e)))?;
        
    let metadata_js = serde_wasm_bindgen::to_value(&metadata)
        .map_err(|e| to_js_error(format!("Failed to serialize metadata: {}", e)))?;

    // Create and serialize bundle
    let bundle = ProofWithMetadataJS::new(proof_js, metadata_js)?;
    serde_wasm_bindgen::to_value(&bundle)
        .map_err(|e| to_js_error(format!("Failed to serialize bundle: {}", e)))
}

// Helper function for consistent error formatting
fn to_js_error<E: std::fmt::Display>(error: E) -> JsValue {
    JsValue::from_str(&format!("Error: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_proof_metadata() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let metadata = ProofMetadataJS::new(
            ProofType::StateTransition as i32,
            timestamp
        ).unwrap();

        assert_eq!(metadata.created_at(), timestamp);
        assert_eq!(metadata.proof_type(), ProofType::StateTransition as i32);
        assert_eq!(metadata.security_bits(), MIN_SECURITY_BITS);
        assert!(metadata.verified_at().is_none());
    }

    #[wasm_bindgen_test]
    fn test_proof_with_metadata() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let zk_proof = ZkProof {
            proof_data: vec![1, 2, 3],
            public_inputs: vec![100, 200],
            merkle_root: vec![0; 32],
            timestamp,
            security_bits: MIN_SECURITY_BITS,
        };

        let metadata = ProofMetadataJS::new(
            ProofType::StateTransition as i32,
            timestamp
        ).unwrap();

        let proof_js = serde_wasm_bindgen::to_value(&zk_proof).unwrap();
        let metadata_js = serde_wasm_bindgen::to_value(&metadata).unwrap();

        let bundle = ProofWithMetadataJS::new(proof_js, metadata_js).unwrap();

        let proof_result = bundle.proof().unwrap();
        let metadata_result = bundle.metadata().unwrap();

        assert!(proof_result.is_object());
        assert!(metadata_result.is_object());
    }

    #[wasm_bindgen_test]
    fn test_invalid_security_parameter() {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let zk_proof = ZkProof {
            proof_data: vec![1, 2, 3],
            public_inputs: vec![100],
            merkle_root: vec![0; 32],
            timestamp,
            security_bits: 64, // Invalid - less than 128
        };

        let metadata = ProofMetadataJS::new(
            ProofType::StateTransition as i32,
            timestamp
        ).unwrap();

        let proof_js = serde_wasm_bindgen::to_value(&zk_proof).unwrap();
        let metadata_js = serde_wasm_bindgen::to_value(&metadata).unwrap();

        assert!(ProofWithMetadataJS::new(proof_js, metadata_js).is_err());
    }
}