// client_proof_exporter.rs
// This module is responsible for exporting the wallet root and its associated proof in a BOC (Bag of Cells) format for submission to the intermediate layer.

use frame_support::{Deserialize, Serialize};
use crate::core::error::SystemError;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use sha2::{Digest, Sha256};
/// Enum representing different types of proofs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProofType {
    StateTransition = 0,
    BalanceTransfer = 1,
    MerkleInclusion = 2,
}
/// Data structure representing a wallet root and its associated proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletRootProof {
    pub wallet_root: [u8; 32],
    pub proof: ZkProof,
    pub metadata: ProofMetadata,
}
impl WalletRootProof {
    /// Exports the wallet root and its associated proof in a BOC (Bag of Cells) format for submission to the intermediate layer.
    pub fn export_proof_boc(&self) -> Result<BOC, SystemError> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.wallet_root);
        data.extend_from_slice(&self.proof.public_inputs.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>());        data.extend_from_slice(&self.proof.merkle_root);
        data.extend_from_slice(&self.proof.proof_data);
        data.extend_from_slice(&self.metadata.timestamp.to_le_bytes());
        data.extend_from_slice(&self.metadata.nonce.to_le_bytes());
        data.extend_from_slice(&self.metadata.wallet_id);
        data.push(self.metadata.proof_type.clone() as u8);

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let _hash = hasher.finalize();
        
        Ok(BOC::new())
    }
}
/// Metadata for tracking proof context.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub timestamp: u64,
    pub nonce: u64,
    pub wallet_id: [u8; 32],
    pub proof_type: ProofType,
}

impl WalletRootProof {
    /// Creates a new WalletRootProof with the given wallet root, proof, and metadata.
    pub fn new(wallet_root: [u8; 32], proof: ZkProof, metadata: ProofMetadata) -> Self {
        Self {
            wallet_root,
            proof,
            metadata,
        }
    }
}