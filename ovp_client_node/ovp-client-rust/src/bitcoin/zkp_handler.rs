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
    pub fn new(config: BitcoinClientConfig) -> Self {
        let client = BitcoinClient::new(config)
            .expect("Failed to create BitcoinClient");
        
        ZkpHandler {
            client: Arc::new(RwLock::new(client)),
            wallet_manager: Arc::new(RwLock::new(WalletManager::new())),
            transaction_manager: Arc::new(RwLock::new(TransactionManager::new(
                Arc::new(RwLock::new(Plonky2SystemHandle::new())),
                Arc::new(RwLock::new(Arc::new(RwLock::new(STATEBOC::new())))),
            ))),
        }
    }

    pub async fn create_channel(        &self,
        sender: Vec<u8>,
        recipient: Vec<u8>,
        deposit: u64,
        config: ChannelConfig,
    ) -> Result<Vec<u8>, JsValue> {
        let sender_array: [u8; 32] = sender.try_into().map_err(|_| JsValue::from_str("Invalid sender length"))?;
        let recipient_array: [u8; 32] = recipient.try_into().map_err(|_| JsValue::from_str("Invalid recipient length"))?;

        let client = self.client.read().map_err(|_| JsValue::from_str("Lock error"))?;
        let wallet_manager = self.wallet_manager.read().map_err(|_| JsValue::from_str("Lock error"))?;
        let transaction_manager = self.transaction_manager.read().map_err(|_| JsValue::from_str("Lock error"))?;

        let channel_id = client
            .create_and_sign_transaction(
                &sender_array,
                recipient_array,
                deposit,
                &config,
                &wallet_manager,
                &transaction_manager,
            )
            .await?;
        Ok(channel_id.to_vec())
    }

    pub async fn update_channel_state(
        &self,
        channel_id: Vec<u8>,
        new_state: JsValue,
    ) -> Result<Vec<u8>, JsValue> {
        let channel_id_array: [u8; 32] = channel_id.try_into().map_err(|_| JsValue::from_str("Invalid channel_id length"))?;
        let new_state: ChannelState = serde_wasm_bindgen::from_value(new_state)?;

        let client = self.client.read().map_err(|_| JsValue::from_str("Lock error"))?;
        let wallet_manager = self.wallet_manager.read().map_err(|_| JsValue::from_str("Lock error"))?;
        let transaction_manager = self.transaction_manager.read().map_err(|_| JsValue::from_str("Lock error"))?;

        let proof = client
            .update_channel_state(
                channel_id_array,
                new_state,
                &wallet_manager,
                &transaction_manager,
            )
            .await?;
        Ok(proof)
    }
   pub async fn update_channel_state(
        &self,
        channel_id: [u8; 32],
        new_state: ChannelState,
    ) -> Result<Vec<u8>, JsValue> {
        let client = self.client.read().await;
        let wallet_manager = self.wallet_manager.read().await;
        let transaction_manager = self.transaction_manager.read().await;

        let proof = client
            .create_and_sign_transaction(
                &wallet_manager.get_wallet_address(),
                channel_id,
                new_state.balance,
                &ChannelConfig::default(),
                wallet_manager,
                transaction_manager,
            )
            .await?;

        let channel = wallet_manager.get_channel(&channel_id)?;
        let mut channel = channel.write().await;
        channel.update_balance(new_state.balance);

        Ok(proof.proof_data)
    }

    pub async fn verify_proof(
        &self,
        proof_data: &[u8],
        public_inputs: &[u64],
    ) -> Result<bool, JsValue> {
        let client = self.client.read().await;
        let wallet_manager = self.wallet_manager.read().await;
        let transaction_manager = self.transaction_manager.read().await;

        let proof = ZkProof {
            proof_data: proof_data.to_vec(),
            public_inputs: public_inputs.to_vec(),
            merkle_root: vec![0; 32],
            timestamp: 0,
        };

        let channel = wallet_manager.get_channel(&[0u8; 32])?;
        let mut channel = channel.write().await;
        channel.update_balance(1000);

        let result = client
            .verify_proof(
                &wallet_manager.get_wallet_address(),
                &[0u8; 32],
                &proof,
                &ChannelConfig::default(),
                wallet_manager,
                transaction_manager,
            )
            .await?;

        Ok(result)
    }
}