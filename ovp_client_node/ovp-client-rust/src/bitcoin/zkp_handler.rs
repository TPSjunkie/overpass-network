use crate::core::client::wallet_extension::user::ChannelState;
use crate::bitcoin::client::BitcoinClient;
use crate::bitcoin::transactions::TransactionManager;
use crate::bitcoin::wallet::WalletManager;
use crate::common::types::state_boc::STATEBOC;
use crate::core::zkps::proof::ZkProof;
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;

pub type Plonky2SystemHandle<F> = plonky2::iop::witness::PartialWitness<F>;

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ChannelConfig {
    pub timeout: u64,
    pub min_balance: u64,
    pub max_balance: u64,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            timeout: 144, // 24 hours in blocks
            min_balance: 546, // Bitcoin dust limit
            max_balance: 21_000_000 * 100_000_000, // Max bitcoin supply
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct ZkpHandler {
    client: Arc<RwLock<BitcoinClient>>,
    wallet_manager: Arc<RwLock<WalletManager>>,
    transaction_manager: Arc<RwLock<TransactionManager>>,
}

#[wasm_bindgen]
impl ZkpHandler {
    #[wasm_bindgen(constructor)]
    pub fn new(config: BitcoinClientConfig) -> Result<ZkpHandler, JsValue> {
        let client = BitcoinClient::new(config)
            .map_err(|e| JsValue::from_str(&format!("Failed to create BitcoinClient: {}", e)))?;
        
        Ok(ZkpHandler {
            client: Arc::new(RwLock::new(client)),
            wallet_manager: Arc::new(RwLock::new(WalletManager::new())),
            transaction_manager: Arc::new(RwLock::new(TransactionManager::new(
                Arc::new(RwLock::new(Plonky2SystemHandle::new())),
                Arc::new(RwLock::new(STATEBOC::new())),
            ))),
        })
    }

    pub async fn create_channel(
        &self,
        sender: Vec<u8>,
        recipient: Vec<u8>,
        deposit: u64,
        config: ChannelConfig,
    ) -> Result<Vec<u8>, JsValue> {
        let sender_array: [u8; 32] = sender.try_into()
            .map_err(|_| JsValue::from_str("Invalid sender length"))?;
        let recipient_array: [u8; 32] = recipient.try_into()
            .map_err(|_| JsValue::from_str("Invalid recipient length"))?;

        let client = self.client.read()
            .map_err(|e| JsValue::from_str(&format!("Client lock error: {}", e)))?;
        let wallet_manager = self.wallet_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Wallet manager lock error: {}", e)))?;
        let transaction_manager = self.transaction_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Transaction manager lock error: {}", e)))?;

        let channel_id = client
            .create_and_sign_transaction(
                &sender_array,
                recipient_array,
                deposit,
                &config,
                &wallet_manager,
                &transaction_manager,
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to create channel: {}", e)))?;

        Ok(channel_id.to_vec())
    }

    pub async fn update_channel_state(
        &self,
        channel_id: Vec<u8>,
        new_state: ChannelState,
    ) -> Result<Vec<u8>, JsValue> {
        let channel_id_array: [u8; 32] = channel_id.try_into()
            .map_err(|_| JsValue::from_str("Invalid channel_id length"))?;

        let client = self.client.read()
            .map_err(|e| JsValue::from_str(&format!("Client lock error: {}", e)))?;
        let wallet_manager = self.wallet_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Wallet manager lock error: {}", e)))?;
        let transaction_manager = self.transaction_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Transaction manager lock error: {}", e)))?;

        // Create and sign the state update transaction
        let proof = client
            .create_and_sign_transaction(
                &wallet_manager.get_wallet_address(),
                channel_id_array,
                new_state.balance,
                &ChannelConfig::default(),
                &wallet_manager,
                &transaction_manager,
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to update channel state: {}", e)))?;

        // Update the channel state
        let channel = wallet_manager.get_channel(&channel_id_array)
            .map_err(|e| JsValue::from_str(&format!("Failed to get channel: {}", e)))?;
        let mut channel = channel.write()
            .map_err(|e| JsValue::from_str(&format!("Channel lock error: {}", e)))?;
        channel.update_balance(new_state.balance);

        Ok(proof.proof_data)
    }

    pub async fn verify_proof(
        &self,
        proof_data: &[u8],
        public_inputs: &[u64],
    ) -> Result<bool, JsValue> {
        let client = self.client.read()
            .map_err(|e| JsValue::from_str(&format!("Client lock error: {}", e)))?;
        let wallet_manager = self.wallet_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Wallet manager lock error: {}", e)))?;
        let transaction_manager = self.transaction_manager.read()
            .map_err(|e| JsValue::from_str(&format!("Transaction manager lock error: {}", e)))?;

        let proof = ZkProof {
            proof_data: proof_data.to_vec(),
            public_inputs: public_inputs.to_vec(),
            merkle_root: vec![0; 32],
            timestamp: 0,
        };

        let result = client
            .verify_proof(
                &wallet_manager.get_wallet_address(),
                &[0u8; 32], // Default channel ID for verification
                &proof,
                &ChannelConfig::default(),
                &wallet_manager,
                &transaction_manager,
            )
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to verify proof: {}", e)))?;

        Ok(result)
    }
}