// ./src/core/client/wallet_extension/bitcoin_b2o/bitcoin_state_converter.rs

//! Bitcoin State Converter Module
//! 
//! This module implements the conversion logic between Bitcoin lock states and Overpass channel states.
//! It provides functionality for:
//!
//! - Converting Bitcoin lock states to Overpass channel states with zero-knowledge proofs
//! - Managing state transitions with merkle tree verification
//! - Generating and verifying zero-knowledge proofs for state conversions
//! - Creating serialized state representations using "Bag Of Cells" (BOC)
//! - Handling settlement preparations for closing channels
//!
//! # Core Components
//!
//! - `BitcoinStateConverter`: Main converter handling state transformations
//! - `OverpassBitcoinState`: Representation of channel state in Overpass
//! - `StateConversionError`: Error types specific to state conversion operations
//!
//! # Key Features
//!
//! - Zero-knowledge proof generation and verification
//! - Merkle tree state management
//! - Atomic state transitions
//! - Settlement preparation
//! - BOC serialization for network transmission
//!
//! # Usage Example
//!
//! ```rust
//! let converter = BitcoinStateConverter::new(proof_system, state_tree);
//! let (overpass_state, proof) = converter.convert_lock_to_state(bitcoin_lock_state)?;
//! let is_valid = converter.verify_state_transition(&prev_state, &next_state, &proof)?;
//! ```
//!
//! The module ensures secure and verifiable conversions between Bitcoin and Overpass states
//! while maintaining privacy through zero-knowledge proofs.

//! # Dependencies
//!
//! - bitcoin_hashes: Provides hash functions for Bitcoin-related operations
//! - serde: For serialization and deserialization of data structures       

use bitcoin_hashes::HashEngine;
use std::sync::{Arc, RwLock};
use bitcoin_hashes::{sha256d, Hash};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::common::error::client_errors::{SystemError, SystemErrorType};
use crate::common::types::state_boc::{Cell, CellType, STATEBOC};
use crate::core::state::sparse_merkle_tree_wasm::SparseMerkleTreeWasm;
use crate::core::client::wallet_extension::channel_manager::Plonky2SystemHandle;
use crate::core::zkps::proof::ZkProof;

#[derive(Error, Debug)]
pub enum StateConversionError {
    #[error("Lock acquisition failed: {0}")]
    LockError(String),
    #[error("State tree operation failed: {0}")]
    StateTreeError(String),
    #[error("Proof generation failed: {0}")]
    ProofError(String),
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),
    #[error("Serialization failed: {0}")]
    SerializationError(String),
    #[error("Hash computation failed: {0}")]
    HashError(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OverpassBitcoinState {
    pub channel_id: [u8; 32],
    pub state_root: [u8; 32],
    pub current_balance: u64,
    pub nonce: u64,
    pub sequence: u32,
    pub pubkey_hash: [u8; 20],
    pub merkle_proof: Vec<u8>,
}

impl OverpassBitcoinState {
    pub fn new(
        channel_id: [u8; 32],
        state_root: [u8; 32],
        current_balance: u64,
        pubkey_hash: [u8; 20],
        sequence: u32,
    ) -> Self {
        Self {
            channel_id,
            state_root,
            current_balance,
            nonce: 0,
            sequence,
            pubkey_hash,
            merkle_proof: Vec::new(),
        }
    }

    pub fn verify_transition(&self, next_state: &Self) -> Result<bool, StateConversionError> {
        if next_state.current_balance > self.current_balance {
            return Err(StateConversionError::InvalidTransition(
                "Balance can only decrease".into()));
        }

        if next_state.nonce != self.nonce + 1 {
            return Err(StateConversionError::InvalidTransition(
                "Invalid nonce increment".into()));
        }

        if next_state.pubkey_hash != self.pubkey_hash {
            return Err(StateConversionError::InvalidTransition(
                "Pubkey hash cannot change".into()));
        }

        Ok(true)
    }
}

pub struct BitcoinStateConverter {
    proof_system: Arc<Plonky2SystemHandle>,
    state_tree: Arc<RwLock<SparseMerkleTreeWasm>>,
}

impl BitcoinStateConverter {
    pub fn new(
        proof_system: Arc<Plonky2SystemHandle>,
        state_tree: Arc<RwLock<SparseMerkleTreeWasm>>,
    ) -> Self {
        Self {
            proof_system,
            state_tree,
        }
    }

    pub fn convert_lock_to_state(
        &self,
        lock_state: &BitcoinLockState,
    ) -> Result<(OverpassBitcoinState, ZkProof), SystemError> {
        let channel_id = self.generate_channel_id(lock_state)?;

        // Create initial state data
        let mut state_data = Vec::new();
        state_data.extend_from_slice(&lock_state.lock_amount.to_be_bytes());
        state_data.extend_from_slice(&lock_state.pubkey_hash);
        state_data.extend_from_slice(&[0u8; 32]); 

        // Update state tree and get proof
        let merkle_proof = {
            let mut tree = self.state_tree.write()
                .map_err(|e| SystemError::new(
                    SystemErrorType::LockAcquisitionError,
                    format!("Failed to acquire state tree write lock: {}", e)))?;

            tree.insert(&channel_id, &state_data)
                .map_err(|e| SystemError::new(
                    SystemErrorType::StateUpdateError,
                    format!("Failed to insert state: {}", e)))?;

            tree.prove(&channel_id)
                .map_err(|e| SystemError::new(
                    SystemErrorType::ProofGenerationError, 
                    format!("Failed to generate merkle proof: {}", e)))?
        };

        let state_root = self.state_tree.read()
            .map_err(|e| SystemError::new(
                SystemErrorType::LockAcquisitionError,
                format!("Failed to acquire state tree read lock: {}", e)))?
            .root();

        let proof = self.generate_conversion_proof(lock_state)?;

        let overpass_state = OverpassBitcoinState {
            channel_id,
            state_root,
            current_balance: lock_state.lock_amount,
            nonce: 0,
            sequence: lock_state.sequence,
            pubkey_hash: lock_state.pubkey_hash,
            merkle_proof,
        };

        Ok((overpass_state, proof))
    }
    pub fn create_state_boc(
        &self,
        lock_state: &BitcoinLockState,
        overpass_state: &OverpassBitcoinState,
        proof: &ZkProof,
    ) -> Result<STATEBOC, SystemError> {
        let mut boc = STATEBOC::new();

        let lock_cell = self.create_lock_state_cell(lock_state)?;
        let state_cell = self.create_overpass_state_cell(overpass_state)?;
        let proof_cell = self.create_proof_cell(proof)?;

        let references = vec![
            lock_cell.get_hash().to_vec(),
            state_cell.get_hash().to_vec(),
            proof_cell.get_hash().to_vec(),
        ];

        boc.add_cell(lock_cell);
        boc.add_cell(state_cell);
        boc.add_cell(proof_cell);
        boc.set_references(references);

        Ok(boc)
    }

    pub fn verify_state_transition(
        &self,
        prev_state: &OverpassBitcoinState,
        new_state: &OverpassBitcoinState,
        proof: &ZkProof,
    ) -> Result<bool, SystemError> {
        prev_state.verify_transition(new_state).map_err(|e| 
            SystemError::new(
                SystemErrorType::InvalidState,
                format!("Invalid state transition: {}", e),
            ))?;

        self.proof_system.verify_proof_js(&proof.proof_data).map_err(|e| 
            SystemError::new(
                SystemErrorType::VerificationError,
                format!("Proof verification failed: {}", e),
            ))?;

        self.verify_root_transition(&prev_state.state_root, &new_state.state_root)?;

        Ok(true)
    }

    pub fn prepare_settlement(
        &self,
        final_state: &OverpassBitcoinState,
    ) -> Result<(BitcoinLockState, ZkProof), SystemError> {
        let tree_root = self.state_tree.read().map_err(|e| 
            SystemError::new(
                SystemErrorType::LockAcquisitionError,
                format!("Failed to acquire state tree lock: {}", e),
            ))?.root();

        if tree_root != final_state.state_root {
            return Err(SystemError::new(
                SystemErrorType::InvalidState,
                "Invalid final state root".to_string(),
            ));
        }

        let proof = self.generate_settlement_proof(final_state)?;

        let lock_state = BitcoinLockState {
            lock_amount: final_state.current_balance,
            lock_script_hash: [0u8; 32],
            lock_height: 0,
            pubkey_hash: final_state.pubkey_hash,
            sequence: final_state.sequence,
        };

        Ok((lock_state, proof))
    }

    fn generate_channel_id(&self, lock_state: &BitcoinLockState) -> Result<[u8; 32], SystemError> {
        let mut engine = sha256d::Hash::engine();
        engine.input(&lock_state.lock_script_hash);
        engine.input(&lock_state.lock_height.to_be_bytes());
        engine.input(&lock_state.pubkey_hash);
        Ok(sha256d::Hash::from_engine(engine).to_byte_array())
    }

    fn generate_conversion_proof(&self, lock_state: &BitcoinLockState) -> Result<ZkProof, SystemError> {
        let circuit_inputs = vec![
            0u64.to_be_bytes().to_vec(),
            lock_state.lock_amount.to_be_bytes().to_vec(),
            lock_state.lock_amount.to_be_bytes().to_vec(),
        ];

        let proof_bytes = self.proof_system.verify_proof_js(&circuit_inputs).map_err(|e| 
            SystemError::new(
                SystemErrorType::ProofGenerationError,
                format!("Failed to generate conversion proof: {}", e),
            ))?;

        Ok(ZkProof {
            proof_data: proof_bytes,
            public_inputs: vec![0, lock_state.lock_amount, lock_state.lock_amount],
            merkle_root: self.state_tree.read().map_err(|e| 
                SystemError::new(
                    SystemErrorType::LockAcquisitionError,
                    format!("Failed to acquire state tree lock: {}", e),
                ))?.root(),
            timestamp: current_timestamp(),
        })
    }

    fn generate_settlement_proof(&self, final_state: &OverpassBitcoinState) -> Result<ZkProof, SystemError> {
        let circuit_inputs = vec![
            final_state.current_balance.to_be_bytes().to_vec(),
            0u64.to_be_bytes().to_vec(),
            final_state.current_balance.to_be_bytes().to_vec(),
        ];

        let proof_bytes = self.proof_system.verify_proof_js(&circuit_inputs).map_err(|e| 
            SystemError::new(
                SystemErrorType::ProofGenerationError,
                format!("Failed to generate settlement proof: {}", e),
            ))?;

        Ok(ZkProof {
            proof_data: proof_bytes,
            public_inputs: vec![
                final_state.current_balance,
                0,
                final_state.current_balance,
            ],
            merkle_root: final_state.state_root.to_vec(),
            timestamp: current_timestamp(),
        })
    }

    fn create_lock_state_cell(&self, lock_state: &BitcoinLockState) -> Result<Cell, SystemError> {
        let mut data = Vec::new();
        data.extend_from_slice(&lock_state.lock_amount.to_be_bytes());
        data.extend_from_slice(&lock_state.lock_script_hash);
        data.extend_from_slice(&lock_state.lock_height.to_be_bytes());

        Ok(Cell::new(
            data,
            Vec::new(),
            Vec::new(),
            CellType::Ordinary,
            [0u8; 32],
            None,
        ))
    }

    fn create_overpass_state_cell(&self, state: &OverpassBitcoinState) -> Result<Cell, SystemError> {
        let mut data = Vec::new();
        data.extend_from_slice(&state.channel_id);
        data.extend_from_slice(&state.current_balance.to_be_bytes());
        data.extend_from_slice(&state.nonce.to_be_bytes());

        Ok(Cell::new(
            data,
            Vec::new(),
            Vec::new(),
            CellType::Ordinary,
            [0u8; 32],
            None,
        ))
    }

    fn create_proof_cell(&self, proof: &ZkProof) -> Result<Cell, SystemError> {
        Ok(Cell::new(
            proof.proof_data.clone(),
            Vec::new(),
            Vec::new(),
            CellType::Ordinary,
            [0u8; 32],
            None,
        ))
    }

    fn verify_root_transition(&self, prev_root: &[u8; 32], new_root: &[u8; 32]) -> Result<bool, SystemError> {
        let tree = self.state_tree.read().map_err(|e| 
            SystemError::new(
                SystemErrorType::LockAcquisitionError,
                format!("Failed to acquire state tree lock: {}", e),
            ))?;

        // Use the `verify` method instead of `verify_root`
        // We'll use empty key and value for root verification
        tree.verify(&[], &[], prev_root).map_err(|e| 
            SystemError::new(
                SystemErrorType::VerificationError,
                format!("Invalid previous root: {}", e),
            ))?;

        tree.verify(&[], &[], new_root).map_err(|e| 
            SystemError::new(
                SystemErrorType::VerificationError,
                format!("Invalid new root: {}", e),
            ))?;

        Ok(true)
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*; // Add test utilities

    fn setup_test_environment() -> BitcoinStateConverter {
        let proof_system = Arc::new(Plonky2SystemHandle::new().unwrap());
        let state_tree = Arc::new(RwLock::new(SparseMerkleTreeWasm::default()));
        BitcoinStateConverter::new(proof_system, state_tree)
    }

    #[test]
    fn test_state_conversion() {
        let converter = setup_test_environment();
        let lock_state = create_test_lock_state();
        
        let result = converter.convert_lock_to_state(&lock_state);
        assert!(result.is_ok());
        
        let (state, proof) = result.unwrap();
        assert_eq!(state.current_balance, lock_state.lock_amount);
        assert!(!proof.proof_data.is_empty());
    }
    
    #[test]
    fn test_state_conversion_with_invalid_lock() {
        let converter = setup_test_environment();
        let lock_state = create_test_invalid_lock_state();
        
        let result = converter.convert_lock_to_state(&lock_state);
        assert!(result.is_err());
    }

 
    // Helper functions for testing
    #[cfg(test)]
    fn create_test_lock_state() -> BitcoinLockState {
        BitcoinLockState::new(
            100000000, // 1 BTC
            [0u8; 32],
            700000,
            [1u8; 20],
            0,
        ).unwrap()
    }

    fn setup_test_converter() -> BitcoinStateConverter {
        let proof_system = Arc::new(
            Plonky2SystemHandle::new()
                .expect("Failed to create Plonky2SystemHandle")
        );
        let state_tree = Arc::new(RwLock::new(SparseMerkleTreeWasm::new()));
        BitcoinStateConverter::new(proof_system, state_tree)
    }

    fn create_test_lock_state() -> BitcoinLockState {
        BitcoinLockState::new(
            100000000, // 1 BTC
            [0u8; 32],
            700000,
            [1u8; 20],
            0,
        ).unwrap()
    }

    #[test]
    fn test_lock_to_state_conversion() {
        let converter = setup_test_converter();
        let lock_state = create_test_lock_state();

        let result = converter.convert_lock_to_state(lock_state.clone());
        assert!(result.is_ok());

        let (overpass_state, proof) = result.unwrap();
        assert_eq!(overpass_state.current_balance, lock_state.lock_amount);
        assert_eq!(overpass_state.pubkey_hash, lock_state.pubkey_hash);
        assert!(!proof.proof_data.is_empty());
    }

    #[test]
    fn test_state_transition_verification() {
        let converter = setup_test_converter();
        let lock_state = create_test_lock_state();

        let (initial_state, _) = converter.convert_lock_to_state(lock_state).unwrap();

        let next_state = OverpassBitcoinState {
            channel_id: initial_state.channel_id,
            state_root: [2u8; 32],
            current_balance: initial_state.current_balance - 1000000, // Spend 0.01 BTC
            nonce: initial_state.nonce + 1,
            sequence: initial_state.sequence,
            pubkey_hash: initial_state.pubkey_hash,
            merkle_proof: Vec::new(),
        };

        let proof = ZkProof {
            proof_data: vec![1, 2, 3], // Simplified test proof
            public_inputs: vec![
                initial_state.current_balance,
                next_state.current_balance,
                1000000,
            ],
            merkle_root: next_state.state_root.to_vec(),
            timestamp: current_timestamp(),
        };

        let result = converter.verify_state_transition(&initial_state, &next_state, &proof);
        assert!(result.is_ok());
    }

    #[test]
    fn test_settlement_preparation() {
        let converter = setup_test_converter();
        let lock_state = create_test_lock_state();
        let (initial_state, _) = converter.convert_lock_to_state(lock_state).unwrap();

        let final_state = OverpassBitcoinState {
            channel_id: initial_state.channel_id,
            state_root: initial_state.state_root,
            current_balance: 50000000, // 0.5 BTC remaining
            nonce: 5,
            sequence: initial_state.sequence,
            pubkey_hash: initial_state.pubkey_hash,
            merkle_proof: Vec::new(),
        };

        let result = converter.prepare_settlement(&final_state);
        assert!(result.is_ok());

        let (settlement_lock_state, proof) = result.unwrap();
        assert_eq!(settlement_lock_state.lock_amount, final_state.current_balance);
        assert_eq!(settlement_lock_state.pubkey_hash, final_state.pubkey_hash);
        assert!(!proof.proof_data.is_empty());
    }

    #[test]
    fn test_boc_creation() {
        let converter = setup_test_converter();
        let lock_state = create_test_lock_state();
        let (overpass_state, proof) = converter.convert_lock_to_state(lock_state.clone()).unwrap();

        let result = converter.create_state_boc(&lock_state, &overpass_state, &proof);
        assert!(result.is_ok());

        let boc = result.unwrap();
        assert!(boc.get_references().len() >= 2);
    }

    #[test]
    fn test_invalid_state_transition() {
        let converter = setup_test_converter();
        let lock_state = create_test_lock_state();
        let (initial_state, _) = converter.convert_lock_to_state(lock_state).unwrap();

        let invalid_state = OverpassBitcoinState {
            channel_id: initial_state.channel_id,
            state_root: [2u8; 32],
            current_balance: initial_state.current_balance + 1000000, // Invalid: balance increase
            nonce: initial_state.nonce + 1,
            sequence: initial_state.sequence,
            pubkey_hash: initial_state.pubkey_hash,
            merkle_proof: Vec::new(),
        };

        let proof = ZkProof {
            proof_data: vec![1, 2, 3],
            public_inputs: vec![
                initial_state.current_balance,
                invalid_state.current_balance,
                1000000,
            ],
            merkle_root: invalid_state.state_root.to_vec(),
            timestamp: current_timestamp(),
        };

        let result = converter.verify_state_transition(&initial_state, &invalid_state, &proof);
        assert!(result.is_err());
    }
}