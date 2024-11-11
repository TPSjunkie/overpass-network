// src/core/global/intermediate_verifier.rs

use crate::core::hierarchy::client::wallet_extension::balance::ZkProof;

#[derive(Debug)]
pub enum VerificationError {
    InvalidProof,
    InvalidMerkleRoot,
    BalanceOverflow,
    InconsistentGlobalState,
}

pub struct IntermediateVerifier {
    global_merkle_root: [u8; 32],
}

impl IntermediateVerifier {
    /// Verifies proofs submitted by the intermediate layers and updates global state
    pub fn verify_intermediate_proof(
        &mut self,
        intermediate_root: [u8; 32],
        proof: ZkProof,
        shard_balance: u64,
    ) -> Result<bool, VerificationError> {
        // Verify the Merkle proof for the intermediate contract state
        if !self.verify_merkle_proof(&intermediate_root, &proof) {
            return Err(VerificationError::InvalidProof);
        }

        // Check for balance overflow
        if shard_balance >= u64::MAX {
            return Err(VerificationError::BalanceOverflow);
        }

        // Verify the intermediate root hash
        let computed_hash = self.compute_poseidon_hash(&intermediate_root);
        if !self.verify_root_hash(computed_hash) {
            return Err(VerificationError::InvalidMerkleRoot);
        }

        // Update global Merkle root (O(log m) operation)
        self.update_global_root(intermediate_root)?;

        Ok(true)
    }

    fn verify_merkle_proof(&self, root: &[u8; 32], proof: &ZkProof) -> bool {
        let mut current_hash = proof.leaf_hash;

        for (sibling, is_left) in proof.siblings.iter().zip(proof.path.iter()) {
            let (left, right) = if *is_left {
                (sibling, &current_hash)
            } else {
                (&current_hash, sibling)
            };

            let mut combined = [0u8; 64];
            combined[..32].copy_from_slice(left);
            combined[32..].copy_from_slice(right);

            current_hash = self.compute_poseidon_hash(&combined[..32]);
        }

        &current_hash == root
    }

    fn compute_poseidon_hash(&self, data: &[u8; 32]) -> [u8; 32] {
        let mut state = [0u8; 32];
        let mut t = [0u8; 32];

        // Initialize state with input
        state.copy_from_slice(data);

        // Apply Poseidon round constants and S-boxes
        for r in 0..8 {
            // Add round constants
            for i in 0..32 {
                state[i] = state[i].wrapping_add(POSEIDON_ROUND_CONSTANTS[r][i]);
            }

            // Apply S-box (x^5 in GF(p))
            for i in 0..32 {
                let x = state[i] as u32;
                let x2 = (x * x) % 255;
                let x4 = (x2 * x2) % 255;
                state[i] = ((x4 * x) % 255) as u8;
            }

            // Mix layer using MDS matrix
            t.copy_from_slice(&state);
            for i in 0..32 {
                state[i] = 0;
                for j in 0..32 {
                    state[i] = state[i]
                        .wrapping_add((t[j] as u32 * POSEIDON_MDS_MATRIX[i][j] as u32 % 255) as u8);
                }
            }
        }

        state
    }

    fn verify_root_hash(&self, hash: [u8; 32]) -> bool {
        // Verify that the provided hash matches the stored global Merkle root
        // This ensures the integrity of the intermediate contract hierarchy

        // First check if hash is valid (non-zero)
        let is_valid = hash.iter().any(|&x| x != 0);
        if !is_valid {
            return false;
        }

        // Compare the provided hash with stored global root
        // Both must be exactly equal for verification to pass
        let mut result = true;
        for i in 0..32 {
            if hash[i] != self.global_merkle_root[i] {
                result = false;
                break;
            }
        }

        result
    }

    fn update_global_root(
        &mut self,
        new_intermediate_root: [u8; 32],
    ) -> Result<(), VerificationError> {
        // Validate the new root hash is non-zero
        let is_valid = new_intermediate_root.iter().any(|&x| x != 0);
        if !is_valid {
            return Err(VerificationError::InvalidHash);
        }

        // Hash the new root to ensure it follows proper format
        let hashed_root = self.hash_state(&new_intermediate_root);

        // Update the stored global Merkle root with proper synchronization
        self.global_merkle_root.copy_from_slice(&hashed_root);

        // Emit event or update metadata if needed
        self.last_update_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(())
    }
}
