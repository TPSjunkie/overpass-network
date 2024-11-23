// src/core/hierarchy/client/converters/bitcoin_state_converter.rs

use crate::common::error::client_errors::SystemError;
use crate::common::error::client_errors::SystemErrorType;
use crate::common::types::state_boc;
use crate::common::types::state_boc::CellType;
use crate::common::types::state_boc::STATEBOC;
use crate::core::client::wallet_extension::wallet_extension_types::WalletExtension;
use crate::core::state::sparse_merkle_tree_wasm::SparseMerkleTreeWasm;
use crate::core::zkps::plonky2::Plonky2SystemHandle;
use crate::core::zkps::proof::ZkProof;
use bitcoin::hashes::HashEngine;
use bitcoin::hashes::{sha256d, Hash};
use bitcoin::secp256k1::Secp256k1;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::vec::Vec;
use wasm_bindgen::prelude::*;

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
                SystemError::new(
                    SystemErrorType::LockAcquisitionError,
                    "Failed to acquire state tree lock".to_string(),
                )
            })?;

            tree.update(&channel_id, &state_data).map_err(|_| {
                SystemError::new(
                    SystemErrorType::StateUpdateError,
                    "Failed to update state tree".to_string(),
                )
            })?;

            tree.get_proof(&channel_id).map_err(|_| {
                SystemError::new(
                    SystemErrorType::ProofGenerationError,
                    "Failed to generate merkle proof".to_string(),
                )
            })?
        };

        // Generate proof of state conversion
        let proof_data = self
            .proof_system
            .generate_proof_js(
                lock_state.lock_amount,
                0, // Initial nonce
                lock_state.lock_amount,
                0, // Initial nonce
                0, // No transfer amount for initial conversion
            )
            .map_err(|e| {
                SystemError::new(
                    SystemErrorType::ProofGenerationError,
                    format!("Failed to generate conversion proof: {:?}", e),
                )
            })?;

        // Create Overpass state
        let overpass_state = OverpassBitcoinState {
            channel_id,
            state_root: self
                .state_tree
                .read()
                .map_err(|_| {
                    SystemError::new(
                        SystemErrorType::LockAcquisitionError,
                        "Failed to acquire state tree lock".to_string(),
                    )
                })?
                .root()
                .try_into()
                .map_err(|_| {
                    SystemError::new(
                        SystemErrorType::DataConversionError,
                        "Invalid root length".to_string(),
                    )
                })?,
            current_balance: lock_state.lock_amount,
            nonce: 0,
            sequence: lock_state.sequence,
            pubkey_hash: lock_state.pubkey_hash,
            merkle_proof,
        };

        let zk_proof = ZkProof::new(proof_data, Vec::new(), Vec::new(), 0);

        Ok((overpass_state, zk_proof))
    }

    pub fn create_conversion_boc(
        &self,
        lock_state: &BitcoinLockState,
        overpass_state: &OverpassBitcoinState,
        proof: &ZkProof,
    ) -> Result<STATEBOC, SystemError> {
        let mut boc = STATEBOC::new();

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
        boc.add_cell(state_boc::Cell::new(
            lock_data,
            Vec::new(),
            Vec::new(),
            CellType::Ordinary,
            [0u8; 32],
            None,
        ));
        boc.add_cell(state_boc::Cell::new(
            state_data,
            Vec::new(),
            Vec::new(),
            CellType::Ordinary,
            [0u8; 32],
            None,
        ));
        boc.add_cell(state_boc::Cell::new(
            proof.proof_data.clone(),
            Vec::new(),
            Vec::new(),
            CellType::Ordinary,
            [0u8; 32],
            None,
        ));

        // Set references
        let references = vec![
            vec![0, 1], // lock_cell to state_cell
            vec![1, 2], // state_cell to proof_cell
        ];
        boc.set_references(references);

        Ok(boc)
    }    /// Verifies state transition within Overpass
    pub fn verify_state_transition(
        &self,
        prev_state: &OverpassBitcoinState,
        new_state: &OverpassBitcoinState,
        proof: &ZkProof,
    ) -> Result<bool, SystemError> {
        // Verify state root transition
        let mut state_root = prev_state.state_root.to_vec();
        state_root.extend_from_slice(&new_state.state_root);
        if state_root != proof.merkle_root {
            return Ok(false);
        }
        // Verify proof
        let mut verification_data = Vec::new();
        verification_data.extend_from_slice(&prev_state.state_root);
        verification_data.extend_from_slice(&new_state.state_root);
        verification_data.extend_from_slice(&new_state.current_balance.to_le_bytes());

        self.proof_system
            .verify_proof_js(&verification_data)
            .map_err(|e| {
                SystemError::new(
                    SystemErrorType::VerificationError,
                    e.as_string().unwrap_or_else(|| "Unknown error".to_string()),
                )
            })?;

        // Verify state root transition
        if !self.verify_root_transition(&prev_state.state_root, &new_state.state_root)? {
            return Ok(false);
        }

        Ok(true)
    }

    /// Prepares settlement state for Bitcoin withdrawal
    pub fn prepare_settlement(
        &self,
        final_state: &OverpassBitcoinState,
    ) -> Result<(BitcoinLockState, ZkProof), SystemError> {
        // Verify final state validity
        let tree_root = self
            .state_tree
            .read()
            .map_err(|_| {
                SystemError::new(
                    SystemErrorType::LockAcquisitionError,
                    "Failed to acquire state tree lock".to_string(),
                )
            })?
            .root();

        if tree_root != final_state.state_root {
            return Err(SystemError::new(
                SystemErrorType::InvalidState,
                "Invalid final state root".to_string(),
            ));
        }

        // Generate proof of final state
        let proof = self.generate_settlement_proof(final_state)?;

        // Create Bitcoin lock state for settlement
        let lock_state = BitcoinLockState {
            lock_amount: final_state.current_balance,
            lock_script_hash: [0u8; 32], // Will be filled by settlement handler
            lock_height: 0,              // Will be filled by settlement handler
            pubkey_hash: final_state.pubkey_hash,
            sequence: final_state.sequence,
        };

        Ok((lock_state, proof))
    }

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
        let old_balance = 0; // Initial balance before conversion
        let old_nonce = 0; // Initial nonce
        let new_balance = lock_state.lock_amount; // Amount being converted
        let new_nonce = lock_state.sequence; // Use sequence as nonce
        let transfer_amount = lock_state.lock_amount;

        let proof_bytes = self
            .proof_system
            .generate_proof_js(
                old_balance,
                old_nonce,
                new_balance,
                new_nonce,
                transfer_amount,
            )
            .map_err(|e| {
                SystemError::new(
                    SystemErrorType::ProofGenerationError,
                    format!("Failed to generate proof: {:?}", e),
                )
            })?;

        // Create ZkProof with converted types
        let proof = ZkProof::new(
            proof_bytes.to_vec(),
            lock_state
                .pubkey_hash
                .to_vec()
                .into_iter()
                .map(|b| b as u64)
                .collect(),
            state_data.to_vec(),
            lock_state.lock_amount,
        );

        Ok(proof)
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
            SystemError::new(
                SystemErrorType::LockAcquisitionError,
                "Failed to acquire state tree lock".to_string(),
            )
        })?;

        // Verify the previous root exists in the tree
        if !tree.verify(prev_root, &[], &[]).map_err(|e| {
            SystemError::new(
                SystemErrorType::VerificationError,
                format!("Failed to verify previous root: {:?}", e),
            )
        })? {
            return Ok(false);
        }

        // Verify the new root is valid according to tree rules
        if !tree.verify(new_root, &[], &[]).map_err(|e| {
            SystemError::new(
                SystemErrorType::VerificationError,
                format!("Failed to verify new root: {:?}", e),
            )
        })? {
            return Ok(false);
        }

        // For now, if both roots verify individually, consider the transition valid
        // A more sophisticated transition verification would need to be implemented
        // based on the specific rules of state transitions
        Ok(true)
    }

    fn generate_settlement_proof(
        &self,
        final_state: &OverpassBitcoinState,
    ) -> Result<ZkProof, SystemError> {
        // Get the current state from the tree
        let tree = self.state_tree.read().map_err(|_| {
            SystemError::new(
                SystemErrorType::LockAcquisitionError,
                "Failed to acquire state tree lock".to_string(),
            )
        })?;

        // Verify the final state exists in the tree
        // Use the `verify` method instead of `verify_root`
        if !tree
            .verify(&final_state.state_root, &[], &[])
            .map_err(|e| {
                SystemError::new(
                    SystemErrorType::VerificationError,
                    format!("Failed to verify state root: {:?}", e),
                )
            })?
        {
            return Err(SystemError::new(
                SystemErrorType::InvalidState,
                "Final state root not found in state tree".to_string(),
            ));
        }

        // Generate proof for the full balance withdrawal
        let proof_bytes = self
            .proof_system
            .generate_proof_js(
                final_state.current_balance, // Current balance as old balance
                final_state.nonce,           // Current nonce
                0,                           // New balance will be 0 after settlement
                final_state.nonce + 1,       // Increment nonce
                final_state.current_balance, // Transfer the full balance
            )
            .map_err(|e| {
                SystemError::new(
                    SystemErrorType::ProofGenerationError,
                    format!("Failed to generate settlement proof: {:?}", e),
                )
            })?;

        // Create settlement proof with final state data
        let mut state_data = Vec::new();
        state_data.extend_from_slice(&final_state.state_root);
        state_data.extend_from_slice(&final_state.channel_id);

        Ok(ZkProof::new(
            proof_bytes.to_vec(),
            final_state.pubkey_hash.iter().map(|&b| b as u64).collect(),
            state_data,
            final_state.current_balance,
        ))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::hashes::hex::FromHex;

    fn setup_test_converter() -> BitcoinStateConverter {
        let proof_system =
            Arc::new(Plonky2SystemHandle::new().expect("Failed to create Plonky2SystemHandle"));
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
        assert!(!proof.public_inputs.is_empty() && !proof.proof_data.is_empty());
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
