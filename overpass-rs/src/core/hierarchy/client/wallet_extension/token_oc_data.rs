// ./src/core/hierarchy/client/wallet_extension/token_oc_data.rs
use serde::{Deserialize, Serialize};
use crate::core::hierarchy::client::wallet_extension::client_proof_exporter::ProofMetadata;
use crate::core::hierarchy::client::wallet_extension::user::User;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "User: Clone")]
pub struct TokenOCData {
    pub wallet_root: [u8; 32],
    pub proof: ZkProof,
    pub metadata: ProofMetadata,
    #[serde(skip)]
    pub user: User,
}

impl Default for TokenOCData {
    fn default() -> Self {
        Self {
            wallet_root: [0u8; 32],
            proof: ZkProof::default(),
            metadata: ProofMetadata::default(),
            // TODO: This should be a real user
            // For now, we'll just use a dummy user
            // Later, we'll need to fetch the user from the blockchain or database
            // and populate the user field with the actual user data
            // This will require some changes to the intermediate layer
            // to handle the user data  
            //   - The intermediate layer will need to store the user data
            user: User::new(String::new(), std::collections::HashSet::new()),
        }
    }
}

impl TokenOCData {
    pub fn new(wallet_root: [u8; 32], proof: ZkProof, metadata: ProofMetadata, user: User) -> Self {
        Self {
            wallet_root,
            proof,
            metadata,
            user,
        }
    }
    // This function exports the wallet root and its associated proof in a BOC (Bag of Cells) format for submission to the intermediate layer.
    pub fn export_proof_boc(&self) -> Result<BOC, String> {
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