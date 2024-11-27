// ./src/core/client/wallet_extension/bitcoin_b2o/bitcoin_state_converter.rs
use bitcoin::{
    hashes::Hash as BitcoinHash,
    secp256k1::{PublicKey, SecretKey},
    bip32::ExtendedPrivKey,
    bip32::{ChainCode, ChildNumber, DerivationPath, ExtendedPubKey},
};
use bitcoin_hashes::HashEngine;
use std::sync::{Arc, RwLock};
use bitcoin_hashes::{sha256, sha256d, hash160, Hash};
use serde::{Serialize, Deserialize};
use thiserror::Error;

use crate::{
    common::error::client_errors::{SystemError, SystemErrorType},
    common::types::state_boc::{Cell, CellType, STATEBOC},
    core::{
        state::sparse_merkle_tree_wasm::SparseMerkleTreeWasm,
        client::wallet_extension::channel_manager::Plonky2SystemHandle,
        zkps::proof::ZkProof,
    },
    bitcoin::bitcoin_types::{
        BitcoinLockState, HTLCParameters, StealthAddress,
        CrossChainState, BridgeParameters,
    },
};

const MIN_SECURITY_BITS: usize = 128; // Minimum security parameter λ

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
    
    #[error("Security parameter error: {0}")]
    SecurityError(String),
    
    #[error("Cross-chain error: {0}")]
    CrossChainError(String),
    
    #[error("HTLC error: {0}")]
    HTLCError(String),
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
    pub htlc_params: Option<HTLCParameters>,
    pub stealth_data: Option<StealthAddress>,
    pub security_bits: usize,
}

impl OverpassBitcoinState {
    /// Creates new state with required security parameter
    pub fn new(
        channel_id: [u8; 32],
        state_root: [u8; 32],
        current_balance: u64,
        pubkey_hash: [u8; 20],
        sequence: u32,
    ) -> Result<Self, StateConversionError> {
        Ok(Self {
            channel_id,
            state_root,
            current_balance,
            nonce: 0,
            sequence,
            pubkey_hash,
            merkle_proof: Vec::new(),
            htlc_params: None,
            stealth_data: None,
            security_bits: MIN_SECURITY_BITS,
        })
    }

    /// Verifies state transition with security guarantees
    pub fn verify_transition(&self, next_state: &Self) -> Result<bool, StateConversionError> {
        // Verify security parameter
        if self.security_bits < MIN_SECURITY_BITS {
            return Err(StateConversionError::SecurityError(
                format!("Security parameter λ must be at least {} bits", MIN_SECURITY_BITS)
            ));
        }

        // Verify balance transitions
        if next_state.current_balance > self.current_balance {
            return Err(StateConversionError::InvalidTransition(
                "Balance can only decrease".into()
            ));
        }

        // Verify nonce monotonicity
        if next_state.nonce != self.nonce + 1 {
            return Err(StateConversionError::InvalidTransition(
                "Invalid nonce increment".into()
            ));
        }

        // Verify pubkey hash
        if next_state.pubkey_hash != self.pubkey_hash {
            return Err(StateConversionError::InvalidTransition(
                "Pubkey hash cannot change".into()
            ));
        }

        // Verify HTLC parameters if present
        if let (Some(curr_htlc), Some(next_htlc)) = (&self.htlc_params, &next_state.htlc_params) {
            if !curr_htlc.verify_transition(next_htlc)? {
                return Err(StateConversionError::HTLCError(
                    "Invalid HTLC state transition".into()
                ));
            }
        }

        Ok(true)
    }
}

pub struct BitcoinStateConverter {
    proof_system: Arc<Plonky2SystemHandle>,
    state_tree: Arc<RwLock<SparseMerkleTreeWasm>>,
    security_bits: usize,
}

impl BitcoinStateConverter {
    pub fn new(
        proof_system: Arc<Plonky2SystemHandle>,
        state_tree: Arc<RwLock<SparseMerkleTreeWasm>>,
    ) -> Result<Self, StateConversionError> {
        // Validate security parameter
        let security_bits = MIN_SECURITY_BITS;
        if security_bits < MIN_SECURITY_BITS {
            return Err(StateConversionError::SecurityError(
                format!("Security parameter λ must be at least {} bits", MIN_SECURITY_BITS)
            ));
        }

        Ok(Self {
            proof_system,
            state_tree,
            security_bits,
        })
    }

    /// Converts Bitcoin lock state to Overpass state with cross-chain support
    pub fn convert_lock_to_state(
        &self,
        lock_state: &BitcoinLockState,
        bridge_params: Option<&BridgeParameters>,
    ) -> Result<(OverpassBitcoinState, ZkProof), SystemError> {
        // Generate channel ID with additional entropy
        let channel_id = self.generate_channel_id(lock_state)?;

        // Create state data with HTLC support
        let mut state_data = Vec::new();
        state_data.extend_from_slice(&lock_state.lock_amount.to_be_bytes());
        state_data.extend_from_slice(&lock_state.pubkey_hash);
        
        // Add HTLC data if present
        if let Some(htlc) = &lock_state.htlc_params {
            state_data.extend_from_slice(&htlc.serialize()?);
        }

        // Add bridge data if present
        if let Some(bridge) = bridge_params {
            state_data.extend_from_slice(&bridge.serialize()?);
        }

        // Update state tree with merkle proof
        let merkle_proof = {
            let mut tree = self.state_tree.write().map_err(|e| 
                SystemError::new(
                    SystemErrorType::LockAcquisitionError,
                    format!("Failed to acquire state tree write lock: {}", e)
                ))?;

            tree.insert(&channel_id, &state_data).map_err(|e|
                SystemError::new(
                    SystemErrorType::StateUpdateError,
                    format!("Failed to insert state: {}", e)
                ))?;

            tree.prove(&channel_id).map_err(|e|
                SystemError::new(
                    SystemErrorType::ProofGenerationError,
                    format!("Failed to generate merkle proof: {}", e)
                ))?
        };

        let state_root = self.state_tree.read().map_err(|e|
            SystemError::new(
                SystemErrorType::LockAcquisitionError,
                format!("Failed to acquire state tree read lock: {}", e)
            ))?.root();

        // Generate proof with cross-chain support
        let proof = self.generate_conversion_proof(lock_state, bridge_params)?;

        let overpass_state = OverpassBitcoinState {
            channel_id,
            state_root,
            current_balance: lock_state.lock_amount,
            nonce: 0,
            sequence: lock_state.sequence,
            pubkey_hash: lock_state.pubkey_hash,
            merkle_proof,
            htlc_params: lock_state.htlc_params.clone(),
            stealth_data: lock_state.stealth_address.clone(),
            security_bits: self.security_bits,
        };

        Ok((overpass_state, proof))
    }

    // ... rest of the implementation with similar security and cross-chain enhancements ...
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::{rand::thread_rng, SecretKey};

    #[test]
    fn test_cross_chain_state_conversion() {
        let converter = setup_test_converter();
        let lock_state = create_test_lock_state();
        
        // Create bridge parameters
        let bridge_params = BridgeParameters {
            min_confirmation_depth: 6,
            max_timelock_duration: 144,
            min_value_sat: 546,
            security_bits: MIN_SECURITY_BITS,
        };

        let result = converter.convert_lock_to_state(&lock_state, Some(&bridge_params));
        assert!(result.is_ok());

        let (state, proof) = result.unwrap();
        assert_eq!(state.current_balance, lock_state.lock_amount);
        assert_eq!(state.security_bits, MIN_SECURITY_BITS);
        assert!(!proof.proof_data.is_empty());
    }

    #[test]
    fn test_htlc_state_transition() {
        let converter = setup_test_converter();
        let mut lock_state = create_test_lock_state();

        // Add HTLC parameters
        let secret_key = SecretKey::new(&mut thread_rng());
        let htlc_params = HTLCParameters::new(
            1000000,
            &secret_key,
            144,
            None,
        ).unwrap();
        lock_state.htlc_params = Some(htlc_params);

        let (initial_state, _) = converter.convert_lock_to_state(&lock_state, None).unwrap();

        // Create next state with HTLC spend
        let next_state = OverpassBitcoinState {
            channel_id: initial_state.channel_id,
            state_root: [2u8; 32],
            current_balance: initial_state.current_balance - 1000000,
            nonce: initial_state.nonce + 1,
            sequence: initial_state.sequence,
            pubkey_hash: initial_state.pubkey_hash,
            merkle_proof: Vec::new(),
            htlc_params: None, // HTLC spent
            stealth_data: None,
            security_bits: MIN_SECURITY_BITS,
        };

        assert!(initial_state.verify_transition(&next_state).is_ok());
    }

    // ... additional cross-chain and security-focused tests ...
}
