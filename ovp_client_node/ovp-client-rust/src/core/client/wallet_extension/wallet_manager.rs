// ./src/core/hierarchy/client/wallet_extension/wallet_manager.rs
use crate::core::hierarchy::client::wallet_extension::wallet_extension_types::*;
use crate::core::types::ovp_ops::*;
use crate::core::zkps::plonky2::Plonky2System;
use crate::core::zkps::proof::{ProofMetadataJS, ProofType, ZkProof};
use crate::core::zkps::zkp_interface::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct WalletManager<StateManager> {
    wallets: HashMap<[u8; 32], Arc<RwLock<WalletExtension>>>,
    state_manager: Arc<RwLock<StateManager>>,
    proof_system: Arc<Plonky2System>,
    wallet_id: [u8; 32],
    state_history: Vec<WalletExtension>,
    spending_limit: u64,
}
impl<StateManager> WalletManager<StateManager> {
    pub fn new(state_manager: Arc<RwLock<StateManager>>, proof_system: Arc<Plonky2System>) -> Self {
        Self {
            wallet: HashMap::new(),
            state_manager,
            proof_system,
            wallet_id: [0; 32],
            state_history: Vec::new(),
            spending_limit: 0,
        }
    }

    pub async fn dispatch(&self, op_code: WalletOpCode, params: Vec<u8>) -> Result<Vec<u8>> {
        match op_code {
            WalletOpCode::CreateChannel => {
                // Decode parameters as necessary (e.g., channel_id)
                let channel_id = decode_channel_id(&params)?;
                self.create_channel(channel_id)?;
                Ok(vec![])
            }
            WalletOpCode::CloseChannel => {
                // Decode parameters as necessary (e.g., channel_id)
                let channel_id = decode_channel_id(&params)?;
                self.close_channel(channel_id)?;
                Ok(vec![])
            }
            WalletOpCode::GetChannel => {
                // Decode parameters as necessary (e.g., channel_id)
                let channel_id = decode_channel_id(&params)?;
                self.get_channel(channel_id)?;
                Ok(vec![])
            }
            WalletOpCode::UpdateWalletState => {
                // Decode parameters as necessary (e.g., channel_id)
                let new_state = decode_wallet_state(&params)?;
                self.update_wallet_state(new_state)?;
                Ok(vec![])
            }
            WalletOpCode::ValidateWalletState => {
                // Decode parameters as necessary (e.g., channel_id)
                let proof = decode_wallet_state_proof(&params)?;
                self.validate_wallet_state(proof)?;
                Ok(vec![])
            }
            WalletOpCode::CreateTransaction => {
                // Decode parameters as necessary (e.g., channel_id)
                let transaction = decode_transaction(&params)?;
                self.create_transaction(transaction)?;
                Ok(vec![])
            }
            WalletOpCode::GenerateWalletProof => {
                // Decode parameters as necessary (e.g., channel_id)
                let request = decode_proof_request(&params)?;
                self.generate_wallet_proof(request)?;
                Ok(vec![])
            }
            WalletOpCode::VerifyWalletProof => {
                // Decode parameters as necessary (e.g., channel_id)
                let request = decode_proof_request(&params)?;
                self.verify_wallet_proof(request)?;
                Ok(vec![])
            }
        }
    }

    fn create_channel(&self, channel_id: [u8; 32]) -> Result<(), SystemError> {
        let channel = ChannelContract::new(channel_id);
        self.channels
            .insert(channel_id, Arc::new(RwLock::new(channel)));
        self.total_locked_balance += channel.config.min_balance;
        self.balance_tracker
            .wallet_balances
            .insert(channel_id, channel.config.min_balance);
        self.balance_tracker
            .state_transitions
            .insert(channel_id, vec![]);
        Ok(())
    }
}
