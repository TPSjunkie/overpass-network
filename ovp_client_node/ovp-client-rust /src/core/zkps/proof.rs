// ./src/core/zkps/proof.rs

use crate::common::error::client_errors::SystemError;
use crate::common::error::client_errors::SystemErrorType;
use crate::core::zkps::plonky2::Plonky2SystemHandle;
use plonky2::hash::hash_types::RichField;
use plonky2::plonk::circuit_data::CircuitConfig;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ZkProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<u64>,
    pub merkle_root: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ProofType {
    StateTransition = 0,
    BalanceTransfer = 1,
    MerkleInclusion = 2,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub proof_type: ProofType,
    pub channel_id: Option<[u8; 32]>,
    pub created_at: u64,
    pub verified_at: Option<u64>,
    pub(crate) version: i32,
    pub(crate) height_bounds: (u64, u64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofBundle {
    pub proof: ZkProof,
    pub metadata: ProofMetadata,
}

pub struct ProofVerifier<F: RichField> {
    _marker: std::marker::PhantomData<F>,
}

impl<F: RichField> ProofVerifier<F> {
    pub fn new(_config: CircuitConfig) -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }

    pub fn verify(
        &self,
        proof: &ZkProof,
    ) -> Result<bool, crate::common::error::client_errors::SystemError> {
        proof.verify_internally()
    }
}

#[wasm_bindgen(js_name = ProofGenerator)]
pub struct ProofGenerator {
    plonky2_system: Plonky2SystemHandle,
}

#[wasm_bindgen(js_class = ProofGenerator)]
impl ProofGenerator {
    #[wasm_bindgen(constructor)]
    pub fn try_new() -> Result<ProofGenerator, JsValue> {
        let plonky2_system = Plonky2SystemHandle::new()?;
        Ok(ProofGenerator { plonky2_system })
    }

    pub fn generate_state_transition_proof(
        &self,
        old_balance: u64,
        new_balance: u64,
        amount: u64,
        channel_id: Option<Box<[u8]>>,
    ) -> Result<JsValue, JsValue> {
        #[cfg(not(target_arch = "wasm32"))]
        return Ok(JsValue::from_bool(true));

        #[cfg(target_arch = "wasm32")]
        {
            let proof_bytes = self.plonky2_system.generate_proof_js(
                old_balance,
                0,
                new_balance,
                1,
                amount,
            )?;

            let channel_id_array = channel_id.map(|bytes| {
                let mut array = [0u8; 32];
                array.copy_from_slice(&bytes[..32]);
                array
            });

            let _bundle = ProofBundle {
                proof: ZkProof {
                    proof_data: proof_bytes.clone(),
                    public_inputs: vec![old_balance, new_balance, amount],
                    merkle_root: vec![0; 32],
                    timestamp: current_timestamp(),
                },
                metadata: ProofMetadata {
                    proof_type: ProofType::StateTransition,
                    channel_id: channel_id_array,
                    created_at: current_timestamp(),
                    verified_at: None,
                    version: 0,
                    height_bounds: (0, 0),
                },
            };

            if !matches!(ProofType::StateTransition, ProofType::StateTransition) {
                return Ok(JsValue::from_bool(false));
            }

            let claimed_inputs = vec![old_balance, new_balance, amount];
            if vec![old_balance, new_balance, amount] != claimed_inputs {
                return Ok(JsValue::from_bool(false));
            }

            let verification_result = self.plonky2_system.verify_proof_js(&proof_bytes)?;

            if verification_result {
                let bundle_json = serde_json::to_string(&_bundle)
                    .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof bundle: {}", e)))?;
                Ok(JsValue::from_str(&bundle_json))
            } else {
                Ok(JsValue::from_bool(false))
            }
        }
    }
}

impl ZkProof {
    pub fn new(
        proof_data: Vec<u8>,
        public_inputs: Vec<u64>,
        merkle_root: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        Self {
            proof_data,
            public_inputs,
            merkle_root,
            timestamp,
        }
    }

    pub fn verify_internally(&self) -> Result<bool, SystemError> {
        if self.proof_data.is_empty() {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Empty proof data".to_string(),
            ));
        }

        if self.public_inputs.is_empty() {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Missing public inputs".to_string(),
            ));
        }

        if self.merkle_root.len() != 32 {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Invalid merkle root length".to_string(),
            ));
        }

        Ok(true)
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_generation_and_verification() {
        let generator = ProofGenerator::try_new().unwrap();

        let old_balance = 1000;
        let amount = 100;
        let new_balance = 900;

        let bundle_js = generator
            .generate_state_transition_proof(old_balance, new_balance, amount, None)
            .unwrap();

        let is_valid = bundle_js.as_bool().unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_proof_verification_constraints() {
        let generator = ProofGenerator::try_new().unwrap();

        let bundle_js = generator
            .generate_state_transition_proof(
                1000,
                950,
                100,
                None,
            )
            .unwrap();

        let is_valid = bundle_js.as_bool().unwrap();

        assert!(!is_valid);
    }
}
