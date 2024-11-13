// ./src/core/zkps/bitcoin_proof.rs
use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::zkps::plonky2::Plonky2SystemHandle;
use plonky2::hash::hash_types::RichField;
use plonky2::plonk::circuit_data::CircuitConfig;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinZkProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<u64>,
    pub merkle_root: Vec<u8>,
    pub timestamp: u64,
    pub version: u8,
    pub proof_type: BitcoinProofType,
    pub height_bounds: HeightBounds,
    pub channel_id: String,
    pub created_at: u64,
    pub verified_at: Option<u64>,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct BitcoinProofSlice {
    pub boc_hash: [u8; 32],
    pub metadata: BitcoinProofMetadata,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct BitcoinProofMetadata {
    pub version: u8,
    pub proof_type: BitcoinProofType,
    pub height_bounds: HeightBounds,
}

#[derive(Clone, Copy, Debug, Encode, Decode, TypeInfo, Serialize, Deserialize, PartialEq)]
pub enum BitcoinProofType {
    Deposit = 1,
    Withdrawal = 2,
    Transfer = 3,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo, Serialize, Deserialize)]
pub struct HeightBounds {
    pub min: u64,
    pub max: u64,
    pub start: i32,
    pub end: i32,
}

#[derive(Clone, Debug, Encode, Decode, TypeInfo)]
pub struct BitcoinProofBoc {
    pub proof_data: Vec<u8>,
    pub vk_hash: [u8; 32],
    pub public_inputs: Vec<u8>,
    pub auxiliary_data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinProofBundle {
    pub proof: BitcoinZkProof,
    pub metadata: BitcoinProofMetadata,
}

/// Bitcoin proof verifier
pub struct BitcoinProofVerifier<F: RichField> {
    config: CircuitConfig,
    _marker: std::marker::PhantomData<F>,
}

impl<F: RichField> BitcoinProofVerifier<F> {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            config,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn verify(&self, proof: &BitcoinZkProof) -> Result<bool, SystemError> {
        // Basic proof validation
        proof.verify_internally()?;

        // Type-specific verification
        match proof.proof_type {
            BitcoinProofType::Deposit => self.verify_deposit(proof),
            BitcoinProofType::Withdrawal => self.verify_withdrawal(proof),
            BitcoinProofType::Transfer => self.verify_transfer(proof),
        }
    }

    fn verify_deposit(&self, proof: &BitcoinZkProof) -> Result<bool, SystemError> {
        if proof.public_inputs.len() < 3 {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Deposit proof requires at least 3 public inputs".to_string(),
            ));
        }

        let old_balance = proof.public_inputs[0];
        let new_balance = proof.public_inputs[1];
        let amount = proof.public_inputs[2];

        if new_balance != old_balance + amount {
            return Err(SystemError::new(
                SystemErrorType::InvalidAmount,
                "Invalid deposit amount".to_string(),
            ));
        }

        Ok(true)
    }

    fn verify_withdrawal(&self, proof: &BitcoinZkProof) -> Result<bool, SystemError> {
        if proof.public_inputs.len() < 4 {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Withdrawal proof requires at least 4 public inputs".to_string(),
            ));
        }

        let old_balance = proof.public_inputs[0];
        let new_balance = proof.public_inputs[1];
        let amount = proof.public_inputs[2];
        let fee = proof.public_inputs[3];

        if amount + fee > old_balance {
            return Err(SystemError::new(
                SystemErrorType::InsufficientBalance,
                "Insufficient balance for withdrawal".to_string(),
            ));
        }

        Ok(true)
    }

    fn verify_transfer(&self, proof: &BitcoinZkProof) -> Result<bool, SystemError> {
        if proof.public_inputs.len() < 3 {
            return Err(SystemError::new(
                SystemErrorType::InvalidTransaction,
                "Transfer proof requires at least 3 public inputs".to_string(),
            ));
        }

        let sender_balance = proof.public_inputs[0];
        let amount = proof.public_inputs[1];
        let recipient_balance = proof.public_inputs[2];

        if amount > sender_balance {
            return Err(SystemError::new(
                SystemErrorType::InsufficientBalance,
                "Insufficient balance for transfer".to_string(),
            ));
        }

        Ok(true)
    }
}

#[wasm_bindgen(js_name = BitcoinProofGenerator)]
pub struct BitcoinProofGenerator {
    plonky2_system: Plonky2SystemHandle,
}

#[wasm_bindgen(js_class = BitcoinProofGenerator)]
impl BitcoinProofGenerator {
    #[wasm_bindgen(constructor)]
    pub fn try_new() -> Result<BitcoinProofGenerator, JsValue> {
        let plonky2_system = Plonky2SystemHandle::new()?;
        Ok(BitcoinProofGenerator { plonky2_system })
    }

    pub fn generate_proof(
        &self,
        old_balance: u64,
        new_balance: u64,
        amount: u64,
        proof_type: BitcoinProofType,
        channel_id: Option<Box<[u8]>>,
    ) -> Result<JsValue, JsValue> {
        let proof_bytes = self.plonky2_system.generate_proof_js(
            old_balance,
            0, // nonce
            new_balance,
            1, // new nonce
            amount,
        )?;

        let channel_id_array = channel_id.map(|bytes| {
            let mut array = [0u8; 32];
            array.copy_from_slice(&bytes[..32]);
            array
        });

        let channel_id_str = channel_id_array
            .as_ref()
            .map(hex::encode)
            .unwrap_or_default();

        let current_time = current_timestamp();

        let bundle = BitcoinProofBundle {
            proof: BitcoinZkProof {
                proof_data: proof_bytes,
                public_inputs: vec![old_balance, new_balance, amount],
                merkle_root: Vec::new(),
                timestamp: current_time,
                version: 1,
                proof_type,
                channel_id: channel_id_str,
                created_at: current_time,
                verified_at: None,
                height_bounds: HeightBounds {
                    min: 0,
                    max: 0,
                    start: 0,
                    end: 0,
                },
            },
            metadata: BitcoinProofMetadata {
                version: 1,
                proof_type,
                height_bounds: HeightBounds {
                    min: 0,
                    max: 0,
                    start: 0,
                    end: 0,
                },
            },
        };

        serde_wasm_bindgen::to_value(&bundle)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize proof bundle: {}", e)))
    }
}

impl BitcoinZkProof {
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
    fn test_bitcoin_proof_generation() {
        let generator = BitcoinProofGenerator::try_new().unwrap();
        let result = generator.generate_proof(
            1000,
            900,
            100,
            BitcoinProofType::Transfer,
            None,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_bitcoin_proof_verification() {
        let config = CircuitConfig::standard_recursion_config();
        let verifier = BitcoinProofVerifier::<F>::new(config);

        let proof = BitcoinZkProof {
            proof_data: vec![1, 2, 3],
            public_inputs: vec![1000, 900, 100],
            merkle_root: vec![0; 32],
            timestamp: current_timestamp(),
            version: 1,
            proof_type: BitcoinProofType::Transfer,
            channel_id: String::new(),
            created_at: current_timestamp(),
            verified_at: None,
            height_bounds: HeightBounds {
                min: 0,
                max: 0,
                start: 0,
                end: 0,
            },
        };

        let result = verifier.verify(&proof);
        assert!(result.is_ok());
    }
}