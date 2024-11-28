// ./src/core/client/wallet_extension/zkp_handler.rs

use crate::core::client::wallet_extension::user::ChannelState;
use crate::bitcoin::{
    client::BitcoinClient,
    transactions::TransactionManager,
    wallet::WalletManager,
};
use crate::common::types::state_boc::STATEBOC;
use crate::core::zkps::{
    proof::ZkProof,
    cross_chain::{CrossChainSwap, StealthAddress, BridgeConfig},
};
use std::sync::{Arc, RwLock};
use bitcoin::Amount;
use wasm_bindgen::prelude::*;
use bitcoin::hashes::{sha256, Hash};

pub type Plonky2SystemHandle<F> = plonky2::iop::witness::PartialWitness<F>;

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ChannelConfig {
    pub timeout: u64,
    pub min_balance: u64,
    pub max_balance: u64,
    pub security_bits: usize,
    pub bridge_enabled: bool,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            timeout: 144, // 24 hours in blocks
            min_balance: 546, // Bitcoin dust limit
            max_balance: 21_000_000 * 100_000_000, // Max bitcoin supply
            security_bits: 128, // Minimum security parameter λ
            bridge_enabled: false,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct CrossChainConfig {
    pub min_confirmation_depth: u32,
    pub max_timelock_duration: u32,
    pub min_value_sat: u64,
    pub fee_rate: u64,
    pub security_bits: usize,
}

impl new for CrossChainConfig {
    fn new() -> Self {
        Self {
            min_confirmation_depth: 6,
            max_timelock_duration: 144, // 24 hours in blocks
            min_value_sat: 546,         // Bitcoin dust limit
            fee_rate: 1000,             // 1000 satoshis per byte
            security_bits: 128,         // Minimum security parameter λ
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ZkpHandler {
    client: Arc<RwLock<BitcoinClient>>,
    wallet_manager: Arc<RwLock<WalletManager>>,
    transaction_manager: Arc<RwLock<TransactionManager>>,
    bridge_config: CrossChainConfig,
}

#[wasm_bindgen]
impl ZkpHandler {
    #[wasm_bindgen(constructor)]
    pub fn new(config: BitcoinClientConfig) -> Result<ZkpHandler, JsValue> {
        let client = BitcoinClient::new(config)
            .map_err(|e| JsValue::from_str(&format!("Failed to create BitcoinClient: {}", e)))?;
        
        let plonky2_system = Arc::new(RwLock::new(Plonky2SystemHandle::new()));
        let state_boc = Arc::new(RwLock::new(STATEBOC::new()));
        let wallet_manager = Arc::new(RwLock::new(WalletManager::new()));
        let client_arc = Arc::new(RwLock::new(client));

        Ok(ZkpHandler {
            client: client_arc.clone(),
            wallet_manager: wallet_manager.clone(),
            transaction_manager: Arc::new(RwLock::new(TransactionManager::new(
                client_arc,
                wallet_manager,
                plonky2_system,
                state_boc,
            ))),
            bridge_config: CrossChainConfig::new(),
        })
    }
    // New cross-chain methods
    pub async fn create_cross_chain_swap(
        &self,
        sender: Vec<u8>,
        recipient: Vec<u8>,
        amount: u64,
        timelock: u32,
    ) -> Result<Vec<u8>, JsValue> {
        let sender_array: [u8; 32] = sender.try_into()
            .map_err(|_| JsValue::from_str("Invalid sender length"))?;
        let recipient_array: [u8; 32] = recipient.try_into()
            .map_err(|_| JsValue::from_str("Invalid recipient length"))?;

        // Generate stealth address
        let stealth_address = self.generate_stealth_address(&recipient_array)?;

        // Create HTLC parameters
        let mut preimage = [0u8; 32];
        getrandom::getrandom(&mut preimage)
            .map_err(|e| JsValue::from_str(&format!("Failed to generate random preimage: {}", e)))?;
        
        let hash_lock = sha256::Hash::hash(&preimage);

        let swap = CrossChainSwap {
            htlc_params: HtlcParams {
                amount: Amount::from_sat(amount),
                timelock,
                hash_lock: hash_lock.into_inner(),
                recipient_key: stealth_address.spend_pubkey,
                refund_key: self.wallet_manager.read().unwrap().get_refund_key()?,
                stealth_pubkey: Some(stealth_address.scan_pubkey),
                merkle_proof: None,
            },
            stealth_address,
            bridge_config: BridgeConfig {
                network: self.client.read().unwrap().get_network(),
                min_confirmation_depth: self.bridge_config.min_confirmation_depth,
                max_timelock_duration: self.bridge_config.max_timelock_duration,
                min_value_sat: self.bridge_config.min_value_sat,
                security_level: self.bridge_config.security_bits,
            },
            merkle_root: [0u8; 32],
        };

        // Create and sign cross-chain transaction
        let transaction_manager = self.transaction_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Transaction manager lock error: {}", e)))?;

        let swap_tx = transaction_manager
            .create_cross_chain_swap(
                &sender_array.into(),
                swap,
                Amount::from_sat(self.bridge_config.fee_rate),
            )
            .map_err(|e| JsValue::from_str(&format!("Failed to create cross-chain swap: {}", e)))?;

        Ok(swap_tx.txid().to_vec())
    }

    pub async fn claim_cross_chain_swap(
        &self,
        swap_id: Vec<u8>,
        preimage: Vec<u8>,
    ) -> Result<Vec<u8>, JsValue> {
        let swap_id_array: [u8; 32] = swap_id.try_into()
            .map_err(|_| JsValue::from_str("Invalid swap ID length"))?;
        let preimage_array: [u8; 32] = preimage.try_into()
            .map_err(|_| JsValue::from_str("Invalid preimage length"))?;

        let transaction_manager = self.transaction_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Transaction manager lock error: {}", e)))?;

        // Get swap details
        let swap = self.wallet_manager.read().unwrap()
            .get_swap(&swap_id_array)
            .map_err(|e| JsValue::from_str(&format!("Failed to get swap: {}", e)))?;

        // Claim funds using preimage
        let claim_tx = transaction_manager
            .claim_cross_chain_htlc(
                swap.outpoint,
                preimage_array,
                &self.wallet_manager.read().unwrap().get_stealth_key()?,
                Amount::from_sat(self.bridge_config.fee_rate),
            )
            .map_err(|e| JsValue::from_str(&format!("Failed to claim swap: {}", e)))?;

        Ok(claim_tx.txid().to_vec())
    }

    pub async fn verify_cross_chain_proof(
        &self,
        proof_data: &[u8],
        public_inputs: &[u64],
        merkle_root: &[u8],
    ) -> Result<bool, JsValue> {
        let merkle_root_array: [u8; 32] = merkle_root.try_into()
            .map_err(|_| JsValue::from_str("Invalid merkle root length"))?;

        let proof = ZkProof {
            proof_data: proof_data.to_vec(),
            public_inputs: public_inputs.to_vec(),
            merkle_root: merkle_root_array.to_vec(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            security_bits: self.bridge_config.security_bits,
        };

        let client = self.client.read()
            .map_err(|e| JsValue::from_str(&format!("Client lock error: {}", e)))?;

        let result = client
            .verify_cross_chain_proof(
                &proof,
                &self.bridge_config,
                &self.wallet_manager.read().unwrap(),
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to verify cross-chain proof: {}", e)))?;

        Ok(result)
    }

    // Helper methods
    fn generate_stealth_address(&self, recipient: &[u8; 32]) -> Result<StealthAddress, JsValue> {
        let secp = bitcoin::secp256k1::Secp256k1::new();
        
        StealthAddress::generate(&secp)
            .map_err(|e| JsValue::from_str(&format!("Failed to generate stealth address: {}", e)))
    }

    pub fn configure_bridge(&mut self, config: CrossChainConfig) {
        self.bridge_config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    async fn test_cross_chain_swap() {
        let config = BitcoinClientConfig::default();
        let handler = ZkpHandler::new(config).unwrap();

        let sender = vec![0u8; 32];
        let recipient = vec![1u8; 32];
        let amount = 100_000;
        let timelock = 144;

        let result = handler.create_cross_chain_swap(
            sender,
            recipient,
            amount,
            timelock,
        ).await;

        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_verify_cross_chain_proof() {
        let config = BitcoinClientConfig::default();
        let handler = ZkpHandler::new(config).unwrap();

        let proof_data = vec![0u8; 64];
        let public_inputs = vec![100_000, 50_000];
        let merkle_root = vec![0u8; 32];

        let result = handler.verify_cross_chain_proof(
            &proof_data,
            &public_inputs,
            &merkle_root,
        ).await;

        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_bridge_config() {
        let config = BitcoinClientConfig::default();
        let mut handler = ZkpHandler::new(config).unwrap();

        let bridge_config = CrossChainConfig {
            min_confirmation_depth: 12,
            max_timelock_duration: 288,
            min_value_sat: 1000,
            fee_rate: 2000,
            security_bits: 256,
        };

        handler.configure_bridge(bridge_config.clone());
        assert_eq!(handler.bridge_config.security_bits, 256);
    }
}