// ./src/core/zkps/proof.rs

use crate::core::error::errors::{SystemError, SystemErrorType};
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

// Proof metadata for tracking context
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub proof_type: ProofType,
    pub channel_id: Option<[u8; 32]>,
    pub created_at: u64,
    pub verified_at: Option<u64>,
    pub(crate) version: i32,
    pub(crate) height_bounds: (u64, u64),
}

// Bundle of proof with its metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofBundle {
    pub proof: ZkProof,
    pub metadata: ProofMetadata,
}

// Circuit verifier
pub struct ProofVerifier<F: RichField> {
    config: CircuitConfig,
    _marker: std::marker::PhantomData<F>,
}

impl<F: RichField> ProofVerifier<F> {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            config,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn verify(&self, proof: &ZkProof) -> Result<bool, SystemError> {
        // Basic proof validation
        proof.verify_internally()?;

        // Type-specific verification
        match ProofType::StateTransition {
            ProofType::StateTransition => self.verify_state_transition(proof),
            ProofType::BalanceTransfer => self.verify_balance_transfer(proof),
            ProofType::MerkleInclusion => self.verify_merkle_inclusion(proof),
        }
    }

    fn verify_state_transition(&self, proof: &ZkProof) -> Result<bool, SystemError> {
        // Validate state transition constraints
        if proof.public_inputs.len() < 3 {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "State transition proof requires at least 3 public inputs".to_string(),
            ));
        }

        // Extract values
        let old_balance = proof.public_inputs[0];
        let new_balance = proof.public_inputs[1];
        let amount = proof.public_inputs[2];

        // Verify basic constraints
        if new_balance > old_balance {
            return Err(SystemError::new(
                SystemErrorType::InvalidAmount,
                "New balance cannot exceed old balance".to_string(),
            ));
        }

        if old_balance - new_balance != amount {
            return Err(SystemError::new(
                SystemErrorType::InvalidAmount,
                "Balance difference must equal transfer amount".to_string(),
            ));
        }

        Ok(true)
    }

    fn verify_balance_transfer(&self, proof: &ZkProof) -> Result<bool, SystemError> {
        // Validate balance transfer constraints
        if proof.public_inputs.len() < 4 {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Balance transfer proof requires at least 4 public inputs".to_string(),
            ));
        }

        // Extract values
        let sender_balance = proof.public_inputs[0];
        let amount = proof.public_inputs[2];
        let fee = proof.public_inputs[3];

        // Verify transfer constraints
        if amount + fee > sender_balance {
            return Err(SystemError::new(
                SystemErrorType::InsufficientBalance,
                "Insufficient balance for transfer and fee".to_string(),
            ));
        }

        Ok(true)
    }

    fn verify_merkle_inclusion(&self, proof: &ZkProof) -> Result<bool, SystemError> {
        // Validate Merkle inclusion proof
        if proof.public_inputs.is_empty() || proof.merkle_root.len() != 32 {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Invalid Merkle inclusion proof structure".to_string(),
            ));
        }

        Ok(true)
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
        // Generate proof using Plonky2
        let proof_bytes = self.plonky2_system.generate_proof_js(
            old_balance,
            0, // nonce
            new_balance,
            1, // new nonce
            amount,
        )?;

        // Convert channel_id if provided
        let channel_id_array = channel_id.map(|bytes| {
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes[..32]);
            array
        });

        // Create proof bundle
        let bundle = ProofBundle {
            proof: ZkProof {
                proof_data: proof_bytes,
                public_inputs: vec![old_balance, new_balance, amount],
                merkle_root: vec![0; 32],
                timestamp: current_timestamp(),
            },
            metadata: ProofMetadata {
                proof_type: ProofType::StateTransition,
                channel_id: channel_id_array,
                created_at: current_timestamp(),
                verified_at: None,
                version: todo!(),
                height_bounds: todo!(),
            },
        };

        serde_wasm_bindgen::to_value(&bundle)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof bundle: {}", e)))
    }
    pub fn verify_state_transition(
        &self,
        bundle_js: &JsValue,
        old_balance: u64,
        new_balance: u64,
        amount: u64,
    ) -> Result<bool, JsValue> {
        // Deserialize proof bundle
        let bundle: ProofBundle =
            serde_wasm_bindgen::from_value(bundle_js.clone()).map_err(|e| {
                JsValue::from_str(&format!("Failed to deserialize proof bundle: {}", e))
            })?;

        // Verify proof type
        if !matches!(bundle.metadata.proof_type, ProofType::StateTransition) {
            return Ok(false);
        }

        // Verify the inputs match
        let claimed_inputs = vec![old_balance, new_balance, amount];
        if bundle.proof.public_inputs != claimed_inputs {
            return Ok(false);
        }

        // Verify the proof itself
        self.plonky2_system
            .verify_proof_js(&bundle.proof.proof_data)
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
        // Verify proof data is present
        if self.proof_data.is_empty() {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Empty proof data".to_string(),
            ));
        }

        // Verify public inputs
        if self.public_inputs.is_empty() {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Missing public inputs".to_string(),
            ));
        }

        // Verify merkle root
        if self.merkle_root.len() != 32 {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Invalid merkle root length".to_string(),
            ));
        }

        Ok(true)
    }
}

// Helper function for timestamp
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

        // Generate proof
        let bundle_js = generator
            .generate_state_transition_proof(old_balance, new_balance, amount, None)
            .unwrap();

        // Verify proof
        let is_valid = generator
            .verify_state_transition(&bundle_js, old_balance, new_balance, amount)
            .unwrap();

        assert!(is_valid);
    }

    #[test]
    fn test_proof_verification_constraints() {
        let generator = ProofGenerator::try_new().unwrap();

        // Test invalid balance transition
        let bundle_js = generator
            .generate_state_transition_proof(
                1000, // old_balance
                950,  // new_balance
                100,  // amount (doesn't match balance difference)
                None,
            )
            .unwrap();

        let is_valid = generator
            .verify_state_transition(&bundle_js, 1000, 950, 100)
            .unwrap();

        assert!(!is_valid);
    }
}
