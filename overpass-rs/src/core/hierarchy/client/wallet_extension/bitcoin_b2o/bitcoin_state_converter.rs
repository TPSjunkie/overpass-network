// src/core/hierarchy/client/converters/bitcoin_state_converter.rs

use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::secp256k1::Secp256k1;

use crate::core::error::errors::SystemError;
use crate::core::hierarchy::client::wallet_extension::sparse_merkle_tree_wasm::SparseMerkleTreeWasm;
use crate::core::types::boc::BOC;
use crate::core::types::ovp_ops::OpCode;
use crate::core::zkps::plonky2::Plonky2SystemHandle;
use crate::core::zkps::proof::ZkProof;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinLockState {
    pub lock_amount: u64,
    pub lock_script_hash: [u8; 32],
    pub lock_height: u32,
    pub pubkey_hash: [u8; 20],
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverpassBitcoinState {
    pub channel_id: [u8; 32],
    pub state_root: [u8; 32],
    pub current_balance: u64,
    pub nonce: u64,
    pub sequence: u64,
    pub pubkey_hash: [u8; 20],
    pub merkle_proof: Vec<u8>,
}
#[wasm_bindgen]
pub struct BitcoinStateConverter {
    proof_system: Arc<Plonky2SystemHandle>,
    state_tree: Arc<RwLock<SparseMerkleTreeWasm>>,
    secp: bitcoin::secp256k1::Secp256k1<bitcoin::secp256k1::All>,
}
impl BitcoinStateConverter {
    pub fn new(
        proof_system: Arc<Plonky2SystemHandle>,
        state_tree: Arc<RwLock<SparseMerkleTreeWasm>>,
    ) -> Self {
        Self {
            proof_system,
            state_tree,
            secp: Secp256k1::new(),
        }
    }

    /// Converts initial Bitcoin lock state to Overpass state
    pub fn convert_lock_to_state(
        &self,
        lock_state: BitcoinLockState,
    ) -> Result<(OverpassBitcoinState, ZkProof), SystemError> {
        // Create initial state data
        let mut state_data = Vec::new();
        state_data.extend_from_slice(&lock_state.lock_amount.to_le_bytes());
        state_data.extend_from_slice(&lock_state.pubkey_hash);
        state_data.extend_from_slice(&[0u8; 32]); // Initial state root

        // Generate channel ID from lock script
        let channel_id = self.generate_channel_id(&lock_state)?;

        // Update state tree with initial state
        let merkle_proof = {
            let mut tree = self.state_tree.write().map_err(|_| {
                SystemError::new(SystemErrorType::LockAcquisitionError, "Failed to acquire state tree lock".to_string())
            })?;
        
            tree.update(&channel_id, &state_data).map_err(|_| {
                SystemError::new(SystemErrorType::StateUpdateError, "Failed to update state tree".to_string())
            })?;

            tree.get_proof(&channel_id).map_err(|_| {
                SystemError::new(SystemErrorType::ProofGenerationError, "Failed to generate merkle proof".to_string())
            })?
        };

        // Generate proof of state conversion
        let proof_data = self.generate_conversion_proof(&lock_state, &state_data)?;

        // Create Overpass state
        let overpass_state = OverpassBitcoinState {
            channel_id,
            state_root: self.state_tree.read().map_err(|_| {
                SystemError::new(SystemErrorType::LockAcquisitionError, "Failed to acquire state tree lock".to_string())
            })?.root().try_into().map_err(|_| {
                SystemError::new(SystemErrorType::DataConversionError, "Invalid root length".to_string())
            })?,
            current_balance: lock_state.lock_amount,
            nonce: 0,
            sequence: lock_state.sequence,
            pubkey_hash: lock_state.pubkey_hash,
            merkle_proof,
        };

        Ok((overpass_state, proof_data))
    }

    /// Creates BOC for state conversion
    pub fn create_conversion_boc(
        &self,
        lock_state: &BitcoinLockState,
        overpass_state: &OverpassBitcoinState,
        proof: &ZkProof,
    ) -> Result<BOC, SystemError> {
        let mut boc = BOC::new();

        // Create cells for lock state
        let mut lock_data = Vec::new();
        lock_data.extend_from_slice(&lock_state.lock_amount.to_le_bytes());
        lock_data.extend_from_slice(&lock_state.lock_script_hash);
        lock_data.extend_from_slice(&lock_state.lock_height.to_le_bytes());
        
        // Create cells for Overpass state
        let mut state_data = Vec::new();
        state_data.extend_from_slice(&overpass_state.channel_id);
        state_data.extend_from_slice(&overpass_state.current_balance.to_le_bytes());
        state_data.extend_from_slice(&overpass_state.nonce.to_le_bytes());
        
        // Add cells to BOC
        let lock_cell = boc.add_cell(lock_data).map_err(|e| SystemError::new(SystemErrorType::BOCError, e.to_string()))?;
        let state_cell = boc.add_cell(state_data).map_err(|e| SystemError::new(SystemErrorType::BOCError, e.to_string()))?;
        let proof_cell = boc.add_cell(proof.to_vec()).map_err(|e| SystemError::new(SystemErrorType::BOCError, e.to_string()))?;

        // Add references
        boc.add_reference(lock_cell, state_cell).map_err(|e| SystemError::new(SystemErrorType::BOCError, e.to_string()))?;
        boc.add_reference(state_cell, proof_cell).map_err(|e| SystemError::new(SystemErrorType::BOCError, e.to_string()))?;

        Ok(boc)
    }

    /// Verifies state transition within Overpass
    pub fn verify_state_transition(
        &self,
        prev_state: &OverpassBitcoinState,
        new_state: &OverpassBitcoinState,
        proof: &ZkProof,
    ) -> Result<bool, SystemError> {
        // Verify basic state properties
        if !self.verify_state_constraints(prev_state, new_state)? {
            return Ok(false);
        }

        // Verify state root transition
        if !self.verify_root_transition(&prev_state.state_root, &new_state.state_root)? {
            return Ok(false);
        }

        // Verify proof
        let mut verification_data = Vec::new();
        verification_data.extend_from_slice(&prev_state.state_root);
        verification_data.extend_from_slice(&new_state.state_root);
        verification_data.extend_from_slice(&new_state.current_balance.to_le_bytes());

        self.proof_system.verify(
            &proof.data(),
            &verification_data,
            &new_state.state_root,
        )
    }

    /// Prepares settlement state for Bitcoin withdrawal
    pub fn prepare_settlement(
        &self,
        final_state: &OverpassBitcoinState,
    ) -> Result<(BitcoinLockState, ZkProof), SystemError> {
        // Verify final state validity
        let tree_root = self.state_tree.read().map_err(|_| {
            SystemError::new(SystemErrorType::LockAcquisitionError, "Failed to acquire state tree lock".to_string())
        })?.root();

        if tree_root != final_state.state_root {
            return Err(SystemError::new(SystemErrorType::InvalidStateError, "Invalid final state root".to_string()));
        }

        // Generate proof of final state
        let proof = self.generate_settlement_proof(final_state)?;

        // Create Bitcoin lock state for settlement
        let lock_state = BitcoinLockState {
            lock_amount: final_state.current_balance,
            lock_script_hash: [0u8; 32], // Will be filled by settlement handler
            lock_height: 0, // Will be filled by settlement handler
            pubkey_hash: final_state.pubkey_hash,
            sequence: final_state.sequence,
        };

        Ok((lock_state, proof))
    }
    // Helper functions
    fn generate_channel_id(&self, lock_state: &BitcoinLockState) -> Result<[u8; 32], SystemError> {
        let mut hasher = sha256d::Hash::engine();
        hasher.input(&lock_state.lock_script_hash);
        hasher.input(&lock_state.lock_height.to_le_bytes());
        hasher.input(&lock_state.pubkey_hash);
        
        let hash = sha256d::Hash::from_engine(hasher);
        let mut channel_id = [0u8; 32];
        channel_id.copy_from_slice(&hash[..]);
        
        Ok(channel_id)
    }

    fn generate_conversion_proof(
        &self,
        lock_state: &BitcoinLockState,
        state_data: &[u8],
    ) -> Result<ZkProof, SystemError> {
        let mut proof_inputs = Vec::new();
        proof_inputs.extend_from_slice(&lock_state.lock_amount.to_le_bytes());
        proof_inputs.extend_from_slice(&lock_state.pubkey_hash);
        proof_inputs.extend_from_slice(state_data);

        self.proof_system.generate_proof(&proof_inputs)
    }

    fn verify_state_constraints(
        &self,
        prev_state: &OverpassBitcoinState,
        new_state: &OverpassBitcoinState,
    ) -> Result<bool, SystemError> {
        // Channel ID must remain constant
        if prev_state.channel_id != new_state.channel_id {
            return Ok(false);
        }

        // Sequence must increment
        if new_state.sequence != prev_state.sequence + 1 {
            return Ok(false);
        }

        // Pubkey hash must remain constant
        if prev_state.pubkey_hash != new_state.pubkey_hash {
            return Ok(false);
        }

        Ok(true)
    }

    fn verify_root_transition(
        &self,
        prev_root: &[u8; 32],
        new_root: &[u8; 32],
    ) -> Result<bool, SystemError> {
        let tree = self.state_tree.read().map_err(|_| {
            SystemError::new_string("Failed to acquire state tree lock")
        })?;

        // Verify root transition in SMT
        tree.verify_root_transition(prev_root, new_root)
            .map_err(|_| SystemError::new_string("Failed to verify root transition"))
    }

    fn generate_settlement_proof(
        &self,
        final_state: &OverpassBitcoinState,
    ) -> Result<ZkProof, SystemError> {
        let mut proof_inputs = Vec::new();
        proof_inputs.extend_from_slice(&final_state.state_root);
        proof_inputs.extend_from_slice(&final_state.current_balance.to_le_bytes());
        proof_inputs.extend_from_slice(&final_state.pubkey_hash);

        self.proof_system.generate_proof(&proof_inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::hex::FromHex;

    fn setup_test_converter() -> BitcoinStateConverter {
        let proof_system = Arc::new(Plonky2SystemHandle::new());
        let state_tree = Arc::new(RwLock::new(SparseMerkleTreeWasm::new()));
        BitcoinStateConverter::new(proof_system, state_tree)
    }

    #[test]
    fn test_lock_to_state_conversion() {
        let converter = setup_test_converter();
        
        let lock_state = BitcoinLockState {
            lock_amount: 100000000, // 1 BTC
            lock_script_hash: [0u8; 32],
            lock_height: 700000,
            pubkey_hash: [1u8; 20],
            sequence: 0,
        };

        let result = converter.convert_lock_to_state(lock_state);
        assert!(result.is_ok());

        let (overpass_state, proof) = result.unwrap();
        assert_eq!(overpass_state.current_balance, 100000000);
        assert!(proof.verify().unwrap());
    }

    #[test]
    fn test_state_transition_verification() {
        let converter = setup_test_converter();
        
        // Create test states
        let prev_state = OverpassBitcoinState {
            channel_id: [0u8; 32],
            state_root: [1u8; 32],
            current_balance: 100000000,
            nonce: 0,
            sequence: 0,
            pubkey_hash: [1u8; 20],
            merkle_proof: vec![],
        };

        let new_state = OverpassBitcoinState {
            channel_id: [0u8; 32],
            state_root: [2u8; 32],
            current_balance: 90000000,
            nonce: 1,
            sequence: 1,
            pubkey_hash: [1u8; 20],
            merkle_proof: vec![],
        };

        let proof = ZkProof::new(vec![], vec![], vec![], 0);

        let result = converter.verify_state_transition(&prev_state, &new_state, &proof);
        assert!(result.is_ok());
    }

    #[test]
    fn test_settlement_preparation() {
        let converter = setup_test_converter();
        
        let final_state = OverpassBitcoinState {
            channel_id: [0u8; 32],
            state_root: [1u8; 32],
            current_balance: 100000000,
            nonce: 10,
            sequence: 10,
            pubkey_hash: [1u8; 20],
            merkle_proof: vec![],
        };

        let result = converter.prepare_settlement(&final_state);
        assert!(result.is_ok());

        let (lock_state, proof) = result.unwrap();
        assert_eq!(lock_state.lock_amount, final_state.current_balance);
        assert_eq!(lock_state.pubkey_hash, final_state.pubkey_hash);
    }
}