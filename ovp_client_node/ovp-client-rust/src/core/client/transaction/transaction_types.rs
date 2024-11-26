// ./src/core/hierarchy/client/transaction/transaction_types.rs

use crate::core::client::channel::channel_contract::ChannelContract;
use crate::core::client::wallet_extension::wallet_extension_types::WalletExtension;
use crate::core::zkps::proof::ZkProof;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents the different types of transactions that can occur in the system
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionType {
    /// Opening a new channel
    ChannelOpen,
    /// Closing an existing channel
    ChannelClose,
    /// Making a payment through the channel
    Payment,
    /// Updating channel state
    StateUpdate,
    /// Dispute resolution
    Dispute,
    /// Emergency channel closure
    EmergencyClose,
}

/// Represents the status of a transaction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction has been confirmed
    Confirmed,
    /// Transaction failed
    Failed,
    /// Transaction was rejected
    Rejected,
    /// Transaction is being processed
    Processing,
}

/// Represents a transaction in the Overpass Channels system
#[derive(Clone)]
pub struct TransactionOCData {
    /// Unique identifier for the transaction
    pub transaction_id: [u8; 32],
    /// Type of transaction
    pub transaction_type: TransactionType,
    /// Reference to the wallet extension
    pub wallet_extension: Arc<RwLock<WalletExtension>>,
    /// Reference to the channel contract
    pub channel_contract: Arc<RwLock<ChannelContract>>,
    /// Zero-knowledge proof associated with the transaction
    pub proof: ZkProof,
    /// Timestamp when the transaction was created
    pub timestamp: u64,
    /// Current status of the transaction
    pub status: TransactionStatus,
    /// Number of confirmations received
    pub confirmations: u32,
    /// Gas price used for the transaction
    pub gas_price: u64,
    /// Gas limit for the transaction
    pub gas_limit: u64,
    /// Actual gas used by the transaction
    pub gas_used: Option<u64>,
    /// Transaction nonce
    pub nonce: u64,
    /// Transaction amount in smallest unit
    pub amount: u128,
    /// Transaction fee
    pub fee: u64,
}
impl TransactionOCData {
    /// Creates a new transaction in the Overpass Channels system
    pub fn new(
        transaction_id: [u8; 32],
        transaction_type: TransactionType,
        wallet_extension: Arc<RwLock<WalletExtension>>,
        channel_contract: Arc<RwLock<ChannelContract>>,
        proof: ZkProof,
        amount: u128,
        gas_price: u64,
        gas_limit: u64,
        nonce: u64,
        fee: u64,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            transaction_id,
            transaction_type,
            wallet_extension,
            channel_contract,
            proof,
            timestamp,
            status: TransactionStatus::Pending,
            confirmations: 0,
            gas_price,
            gas_limit,
            gas_used: None,
            nonce,
            amount,
            fee,
        }
    }

    /// Updates the transaction status
    pub fn update_status(&mut self, status: TransactionStatus) {
        self.status = status;
    }

    /// Increments the confirmation count
    pub fn increment_confirmations(&mut self) {
        self.confirmations = self.confirmations.saturating_add(1);
    }

    /// Sets the amount of gas used
    pub fn set_gas_used(&mut self, gas_used: u64) {
        self.gas_used = Some(gas_used);
    }

    /// Calculates the total transaction cost including gas and fees
    pub fn total_cost(&self) -> Option<u128> {
        self.gas_used.map(|gas| {
            (gas as u128)
                .saturating_mul(self.gas_price as u128)
                .saturating_add(self.fee as u128)
        })
    }

    /// Validates the transaction
    pub fn validate(&self) -> bool {
        // Basic validation checks
        if self.gas_limit == 0 || self.gas_price == 0 {
            return false;
        }

        // Ensure the fee is reasonable
        if self.fee > self.gas_limit.saturating_mul(self.gas_price) {
            return false;
        }

        // Additional validation based on transaction type
        match self.transaction_type {
            TransactionType::Payment => {
                // Ensure payment amount is greater than zero
                if self.amount == 0 {
                    return false;
                }
            }
            TransactionType::ChannelOpen => {
                // Specific validation for channel opening
                if self.amount == 0 {
                    return false;
                }
            }
            _ => {}
        }

        true
    }

    /// Checks if the transaction is finalized
    pub fn is_finalized(&self) -> bool {
        matches!(
            self.status,
            TransactionStatus::Confirmed | TransactionStatus::Failed | TransactionStatus::Rejected
        )
    }

    /// Gets the age of the transaction in seconds
    pub fn age(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(self.timestamp)
    }
}

impl std::fmt::Debug for TransactionOCData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransactionOCData")
            .field("transaction_id", &hex::encode(self.transaction_id))
            .field("transaction_type", &self.transaction_type)
            .field("timestamp", &self.timestamp)
            .field("status", &self.status)
            .field("confirmations", &self.confirmations)
            .field("gas_price", &self.gas_price)
            .field("gas_limit", &self.gas_limit)
            .field("gas_used", &self.gas_used)
            .field("nonce", &self.nonce)
            .field("amount", &self.amount)
            .field("fee", &self.fee)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::client::channel::channel_contract;
    use crate::core::client::transaction::transaction_types::WalletExtension;
    use crate::core::zkps::proof::ZkProof;
    use std::sync::Arc;
    use std::sync::RwLock;

    // Helper function to create a dummy transaction for testing
    fn create_test_transaction() -> TransactionOCData {
        let wallet_extension = Arc::new(RwLock::new(WalletExtension::default()));
        let channel_contract = Arc::new(RwLock::new(channel_contract::ChannelContract::new(
            &hex::encode([0u8; 32]),
        )));
        let proof = ZkProof::default();

        TransactionOCData::new(
            [0u8; 32],
            TransactionType::Payment,
            wallet_extension,
            channel_contract,
            proof,
            1000,
            20_000_000_000, // 20 Gwei
            21000,          // Standard gas limit
            0,              // Nonce
            1000000,        // Fee
        )
    }
    #[test]
    fn test_transaction_validation() {
        let transaction = create_test_transaction();
        assert!(transaction.validate());
    }

    #[test]
    fn test_transaction_status_update() {
        let mut transaction = create_test_transaction();
        assert_eq!(transaction.status, TransactionStatus::Pending);

        transaction.update_status(TransactionStatus::Confirmed);
        assert_eq!(transaction.status, TransactionStatus::Confirmed);
    }

    #[test]
    fn test_confirmation_increment() {
        let mut transaction = create_test_transaction();
        assert_eq!(transaction.confirmations, 0);

        transaction.increment_confirmations();
        assert_eq!(transaction.confirmations, 1);
    }

    #[test]
    fn test_total_cost_calculation() {
        let mut transaction = create_test_transaction();
        transaction.set_gas_used(21000);

        let total_cost = transaction.total_cost().unwrap();
        let expected_cost = (21000u128 * 20_000_000_000u128) + 1000000;
        assert_eq!(total_cost, expected_cost);
    }

    #[test]
    fn test_transaction_age() {
        let transaction = create_test_transaction();
        assert!(transaction.age() > 0);
    }
}
