// ./src/core/hierarchy/client/wallet_extension/token_oc_data.rs
use serde::{Deserialize, Serialize};
use crate::core::hierarchy::client::wallet_extension::client_proof_exporter::ProofMetadata;
use crate::core::hierarchy::client::wallet_extension::user::User;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use sha2::{Digest, Sha256};
use crate::core::hierarchy::client::wallet_extension::client_proof_exporter::*;

pub enum TokenOC {
    TokenOCData(TokenOCData),
    TokenOCBoc(BOC),
    TokenOCProof(WalletRootProof),
}

#[derive(Clone, Debug, Serialize)]
#[serde(bound = "User: Clone")]
pub struct TokenOCData {
    pub wallet_root: [u8; 32],
    pub proof: ZkProof,
    pub metadata: ProofMetadata,
    #[serde(skip)]
    pub user: User,
}

impl<'de> Deserialize<'de> for TokenOCData {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        todo!()
    }
}

impl Default for TokenOCData {
    fn default() -> Self {
        Self {
            wallet_root: [0u8; 32],
            proof: ZkProof::default(),
            metadata: ProofMetadata {
                timestamp: 0,
                nonce: 0,
                wallet_id: [0u8; 32],
                proof_type: ProofType::WalletRoot,
                channel_id: Some([0u8; 32]),
                state_root: Some([0u8; 32]),
                state_proof: None,
            },
            user: User::new(String::new()),
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