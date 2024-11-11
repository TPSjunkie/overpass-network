// src/core/workflow/state_sync.rs

use crate::core::types::ovp_types::BOC;
use crate::network::SyncState;
use crate::SparseMerkleTree;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct StateSync {
    // Caches to prevent redundant syncs
    client_cache: Arc<Mutex<HashMap<[u8; 32], BOC>>>,
    intermediate_cache: Arc<Mutex<HashMap<[u8; 32], SyncState>>>,
    merkle_tree: Arc<Mutex<SparseMerkleTree>>,
}

impl StateSync {
    /// Initializes the `StateSync` with empty caches and an initialized Sparse Merkle Tree.
    pub fn new() -> Self {
        StateSync {
            client_cache: Arc::new(Mutex::new(HashMap::new())),
            intermediate_cache: Arc::new(Mutex::new(HashMap::new())),
            merkle_tree: Arc::new(Mutex::new(SparseMerkleTree::new())),
        }
    }

    /// Synchronizes state across the client, intermediate, and global layers in a lazy fashion.
    pub fn sync_state(&self) -> Result<SyncResult, StateSyncError> {
        let client_root = self.sync_client_to_intermediate()?;
        let intermediate_root = self.sync_intermediate_to_global(client_root)?;
        Ok(SyncResult {
            client_root,
            intermediate_root,
            global_root: intermediate_root,
        })
    }

    /// Syncs client wallets to the intermediate layer, caching finalized wallet roots to avoid duplicate submissions.
    fn sync_client_to_intermediate(&self) -> Result<[u8; 32], StateSyncError> {
        // Collect all finalized proofs from client wallets
        let client_proofs = ProofExporter::collect_finalized_proofs()?;
        let mut intermediate_root = [0u8; 32];

        // Obtain lock on merkle_tree for incremental updates
        let mut merkle_tree = self.merkle_tree.lock().unwrap();

        for (wallet_root, proof) in client_proofs {
            // Check if proof is already in cache to avoid re-verification
            if self.client_cache.lock().unwrap().contains_key(&wallet_root) {
                continue;
            }

            // Verify wallet proof for validity before adding to the intermediate layer
            WalletVerifier::verify_wallet_proof(&wallet_root, &proof)?;

            // Update the intermediate tree with verified wallet root
            intermediate_root = merkle_tree.update(&wallet_root, &proof)?;

            // Cache updated root for future client syncs
            self.client_cache.lock().unwrap().insert(wallet_root, proof);
        }

        Ok(intermediate_root)
    }

    /// Syncs intermediate layer to the global layer, aggregating roots with the Merkle tree for efficiency.
    fn sync_intermediate_to_global(
        &self,
        client_root: [u8; 32],
    ) -> Result<[u8; 32], StateSyncError> {
        let intermediate_proofs = IntermediateProofExporter::collect_finalized_proofs()?;
        let mut global_root = [0u8; 32];

        let mut merkle_tree = self.merkle_tree.lock().unwrap();

        for (intermediate_root, proof) in intermediate_proofs {
            // Verify intermediate proof before adding to global layer
            IntermediateVerifier::verify_intermediate_proof(&intermediate_root, &proof)?;

            // Update the global tree with verified intermediate root
            global_root = merkle_tree.update(&intermediate_root, &proof)?;

            // Cache intermediate state
            self.intermediate_cache
                .lock()
                .unwrap()
                .insert(intermediate_root, SyncState::Verified(client_root));
        }

        Ok(global_root)
    }
}

struct SyncResult {
    client_root: [u8; 32],
    intermediate_root: [u8; 32],
    global_root: [u8; 32],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_state() {
        let sync = StateSync::new();
        let result = sync.sync_state();
        assert!(result.is_ok());

        let sync_result = result.unwrap();
        assert_ne!(sync_result.client_root, [0u8; 32]);
        assert_ne!(sync_result.intermediate_root, [0u8; 32]);
        assert_ne!(sync_result.global_root, [0u8; 32]);
    }
}
