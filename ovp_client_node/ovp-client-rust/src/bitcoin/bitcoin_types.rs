use std::fmt;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use bitcoin_hashes::{sha256d, Hash, HashEngine};

#[derive(Error, Debug)]
pub enum BitcoinStateError {
    #[error("Invalid lock amount: {0}")]
    InvalidLockAmount(String),
    #[error("Invalid script hash: {0}")]
    InvalidScriptHash(String),
    #[error("Invalid lock height: {0}")]
    InvalidLockHeight(String),
    #[error("Invalid pubkey hash length: expected 20 bytes, got {0}")]
    InvalidPubkeyHashLength(usize),
    #[error("Invalid sequence number: {0}")]
    InvalidSequence(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Hash computation error: {0}")]
    HashError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BitcoinLockState {
    pub lock_amount: u64,
    pub lock_script_hash: [u8; 32],
    pub lock_height: u64,
    pub pubkey_hash: [u8; 20],
    pub sequence: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LockParameters {
    pub min_lock_time: u32,
    pub max_lock_time: u32,
    pub min_amount: u64,
    pub max_amount: u64,
    pub allowed_sequences: Vec<u32>,
}

impl Default for LockParameters {
    fn default() -> Self {
        Self {
            min_lock_time: 0,
            max_lock_time: u32::MAX,
            min_amount: 546, // Bitcoin dust limit
            max_amount: 21_000_000 * 100_000_000, // Max bitcoin supply in satoshis
            allowed_sequences: vec![0xFFFFFFFF, 0], // Standard and immediate timelock
        }
    }
}

impl BitcoinLockState {
    pub fn new(
        lock_amount: u64,
        lock_script_hash: [u8; 32],
        lock_height: u64,
        pubkey_hash: [u8; 20],
        sequence: u32,
    ) -> Result<Self, BitcoinStateError> {
        let params = LockParameters::default();
        
        // Validate lock amount
        if lock_amount < params.min_amount || lock_amount > params.max_amount {
            return Err(BitcoinStateError::InvalidLockAmount(format!(
                "Lock amount must be between {} and {} satoshis",
                params.min_amount, params.max_amount
            )));
        }

        // Validate lock height
        if lock_height as u32 > params.max_lock_time {
            return Err(BitcoinStateError::InvalidLockHeight(format!(
                "Lock height cannot exceed {}",
                params.max_lock_time
            )));
        }

        // Validate sequence
        if !params.allowed_sequences.contains(&sequence) {
            return Err(BitcoinStateError::InvalidSequence(format!(
                "Invalid sequence number: {}",
                sequence
            )));
        }

        Ok(Self {
            lock_amount,
            lock_script_hash,
            lock_height,
            pubkey_hash,
            sequence,
        })
    }

    pub fn compute_state_hash(&self) -> Result<[u8; 32], BitcoinStateError> {
        let mut engine = sha256d::Hash::engine();
        
        // Serialize all fields in a deterministic order
        engine.input(&self.lock_amount.to_le_bytes());
        engine.input(&self.lock_script_hash);
        engine.input(&self.lock_height.to_le_bytes());
        engine.input(&self.pubkey_hash);
        engine.input(&self.sequence.to_le_bytes());

        Ok(sha256d::Hash::from_engine(engine).to_byte_array())
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, BitcoinStateError> {
        bincode::serialize(self).map_err(|e| BitcoinStateError::SerializationError(e.to_string()))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, BitcoinStateError> {
        bincode::deserialize(bytes).map_err(|e| BitcoinStateError::DeserializationError(e.to_string()))
    }

    pub fn verify_lock_constraints(&self, params: &LockParameters) -> Result<(), BitcoinStateError> {
        // Check amount constraints
        if self.lock_amount < params.min_amount || self.lock_amount > params.max_amount {
            return Err(BitcoinStateError::InvalidLockAmount(format!(
                "Lock amount {} is outside allowed range [{}, {}]",
                self.lock_amount, params.min_amount, params.max_amount
            )));
        }

        // Check lock time constraints
        if (self.lock_height as u32) < params.min_lock_time || (self.lock_height as u32) > params.max_lock_time {
            return Err(BitcoinStateError::InvalidLockHeight(format!(
                "Lock height {} is outside allowed range [{}, {}]",
                self.lock_height, params.min_lock_time, params.max_lock_time
            )));
        }

        // Check sequence constraints
        if !params.allowed_sequences.contains(&self.sequence) {
            return Err(BitcoinStateError::InvalidSequence(format!(
                "Sequence {} is not in allowed set {:?}",
                self.sequence, params.allowed_sequences
            )));
        }

        Ok(())
    }
}

impl fmt::Display for BitcoinLockState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use bitcoin::consensus::encode::serialize_hex;
        write!(
            f,
            "BitcoinLockState {{ lock_amount: {}, lock_script_hash: {}, lock_height: {}, pubkey_hash: {}, sequence: {} }}",
            self.lock_amount,
            serialize_hex(&self.lock_script_hash),
            self.lock_height,
            hex::encode(&self.pubkey_hash),
            self.sequence
        )
    }}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_lock_state() -> BitcoinLockState {
        BitcoinLockState::new(
            1_000_000, // 0.01 BTC
            [0u8; 32],
            750_000,
            [1u8; 20],
            0xFFFFFFFF,
        )
        .unwrap()
    }

    #[test]
    fn test_valid_lock_state_creation() {
        let lock_state = create_test_lock_state();
        assert_eq!(lock_state.lock_amount, 1_000_000);
        assert_eq!(lock_state.lock_height, 750_000);
        assert_eq!(lock_state.sequence, 0xFFFFFFFF);
    }

    #[test]
    fn test_invalid_lock_amount() {
        let result = BitcoinLockState::new(
            0, // Invalid amount
            [0u8; 32],
            750_000,
            [1u8; 20],
            0xFFFFFFFF,
        );
        assert!(matches!(result, Err(BitcoinStateError::InvalidLockAmount(_))));
    }

    #[test]
    fn test_hash_computation() {
        let lock_state = create_test_lock_state();
        let hash = lock_state.compute_state_hash().unwrap();
        assert_eq!(hash.len(), 32);
        
        // Ensure hash is deterministic
        let hash2 = lock_state.compute_state_hash().unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_serialization() {
        let lock_state = create_test_lock_state();
        let bytes = lock_state.to_bytes().unwrap();
        let decoded = BitcoinLockState::from_bytes(&bytes).unwrap();
        assert_eq!(lock_state, decoded);
    }

    #[test]
    fn test_lock_constraints() {
        let lock_state = create_test_lock_state();
        let params = LockParameters::default();
        assert!(lock_state.verify_lock_constraints(&params).is_ok(), "Lock constraints failed");

        // Test with custom constraints
        let strict_params = LockParameters {
            min_lock_time: 800_000,
            max_lock_time: 900_000,
            min_amount: 2_000_000,
            max_amount: 5_000_000,
            allowed_sequences: vec![0],
        };
        assert!(lock_state.verify_lock_constraints(&strict_params).is_err());
    }

    #[test]
    fn test_display_formatting() {
        let lock_state = create_test_lock_state();
        let display_string = format!("{}", lock_state);
        assert!(display_string.contains(&lock_state.lock_amount.to_string()));
        assert!(display_string.contains(&lock_state.lock_height.to_string()));
    }
}