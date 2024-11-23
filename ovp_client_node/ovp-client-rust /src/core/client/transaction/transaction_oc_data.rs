// ./src/core/hierarchy/client/transaction/transaction_oc_data.rs

use crate::core::client::channel::channel_contract::ChannelContract;
use crate::core::client::transaction::transaction_types::TransactionType;
use crate::core::zkps::proof::ZkProof;
use std::sync::{Arc, RwLock};

/// Represents a transaction in the Overpass Channels system
#[derive(Clone)]
pub struct TransactionOCData<WalletExtension> {
    pub transaction_id: [u8; 32],
    pub transaction_type: TransactionType,
    pub wallet_extension: Arc<RwLock<WalletExtension>>,
    pub channel_contract: Arc<RwLock<ChannelContract>>,
    pub proof: ZkProof,
}

impl<WalletExtension> TransactionOCData<WalletExtension> {
    /// Creates a new transaction in the Overpass Channels system
    pub fn new(
        transaction_id: [u8; 32],
        transaction_type: TransactionType,
        wallet_extension: Arc<RwLock<WalletExtension>>,
        channel_contract: Arc<RwLock<ChannelContract>>,
        proof: ZkProof,
    ) -> Self {
        Self {
            transaction_id,
            transaction_type,
            wallet_extension,
            channel_contract,
            proof,
        }
    }
    /// Gets the transaction ID
    pub fn get_transaction_id(&self) -> [u8; 32] {
        self.transaction_id
    }

    /// Gets the transaction type
    pub fn get_transaction_type(&self) -> TransactionType {
        self.transaction_type.clone()
    }

    /// Gets the wallet extension
    pub fn get_wallet_extension(&self) -> Arc<RwLock<WalletExtension>> {
        self.wallet_extension.clone()
    }
}
