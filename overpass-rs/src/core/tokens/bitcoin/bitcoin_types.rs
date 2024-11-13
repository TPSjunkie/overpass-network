// ./src/core/tokens/bitcoin/bitcoin_types.rs
use codec::{Decode, Encode};
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
    DispatchError,
};
use scale_info::TypeInfo;

/// Network types supported by the Bitcoin integration
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum BitcoinNetwork {
    Bitcoin,
    BitcoinTestnet,
    BitcoinRegtest,
    BitcoinSignet,
    BitcoinSimnet,
}

impl Default for BitcoinNetwork {
    fn default() -> Self {
        BitcoinNetwork::Bitcoin
    }
}

impl From<u8> for BitcoinNetwork {
    fn from(network_id: u8) -> Self {
        match network_id {
            0 => BitcoinNetwork::Bitcoin,
            1 => BitcoinNetwork::BitcoinTestnet,
            2 => BitcoinNetwork::BitcoinRegtest,
            3 => BitcoinNetwork::BitcoinSignet,
            4 => BitcoinNetwork::BitcoinSimnet,
            _ => BitcoinNetwork::Bitcoin,
        }
    }
}

/// Common constants for Bitcoin networks
pub const BITCOIN_MAINNET: u8 = 0;
pub const BITCOIN_TESTNET: u8 = 1;
pub const BITCOIN_REGTEST: u8 = 2;
pub const BITCOIN_SIGNET: u8 = 3;
pub const BITCOIN_SIMNET: u8 = 4;

/// Bitcoin transaction data structure
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct BitcoinTransactionData {
    pub sender: Vec<u8>,
    pub recipient: Vec<u8>,
    pub amount: u64,
    pub nonce: u64,
    pub network: BitcoinNetwork,
    pub timestamp: u64,
    pub metadata: Vec<u8>,
}

/// Bitcoin account information
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct BitcoinAccountData {
    pub address: Vec<u8>,
    pub balance: u64,
    pub nonce: u64,
    pub network: BitcoinNetwork,
}

/// Error types specific to Bitcoin operations
#[derive(Debug)]
pub enum BitcoinError {
    InsufficientBalance(String),
    InvalidNetwork(String),
    InvalidTransaction(String),
    InvalidAccount(String),
    ProofVerificationFailed(String),
}

impl From<BitcoinError> for DispatchError {
    fn from(error: BitcoinError) -> Self {
        match error {
            BitcoinError::InsufficientBalance(msg) => DispatchError::Other(msg.as_str()),
            BitcoinError::InvalidNetwork(msg) => DispatchError::Other(msg.as_str()),
            BitcoinError::InvalidTransaction(msg) => DispatchError::Other(msg.as_str()),
            BitcoinError::InvalidAccount(msg) => DispatchError::Other(msg.as_str()),
            BitcoinError::ProofVerificationFailed(msg) => DispatchError::Other(msg.as_str()),
        }
    }
}

/// Trait for Bitcoin balance operations
pub trait BitcoinBalance: AtLeast32BitUnsigned + MaybeSerializeDeserialize {
    fn zero() -> Self;
    fn max_value() -> Self;
    fn is_zero(&self) -> bool;
}

impl<T: AtLeast32BitUnsigned + MaybeSerializeDeserialize> BitcoinBalance for T {
    fn zero() -> Self {
        Self::zero()
    }

    fn max_value() -> Self {
        Self::max_value()
    }

    fn is_zero(&self) -> bool {
        self == &Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitcoin_network_from() {
        assert_eq!(BitcoinNetwork::from(0), BitcoinNetwork::Bitcoin);
        assert_eq!(BitcoinNetwork::from(1), BitcoinNetwork::BitcoinTestnet);
        assert_eq!(BitcoinNetwork::from(2), BitcoinNetwork::BitcoinRegtest);
        assert_eq!(BitcoinNetwork::from(3), BitcoinNetwork::BitcoinSignet);
        assert_eq!(BitcoinNetwork::from(4), BitcoinNetwork::BitcoinSimnet);
        assert_eq!(BitcoinNetwork::from(5), BitcoinNetwork::Bitcoin); // Default case
    }

    #[test]
    fn test_bitcoin_transaction_data() {
        let tx = BitcoinTransactionData {
            sender: vec![1, 2, 3],
            recipient: vec![4, 5, 6],
            amount: 100,
            nonce: 1,
            network: BitcoinNetwork::Bitcoin,
            timestamp: 12345,
            metadata: vec![],
        };

        assert_eq!(tx.amount, 100);
        assert_eq!(tx.network, BitcoinNetwork::Bitcoin);
    }

    #[test]
    fn test_bitcoin_account_data() {
        let account = BitcoinAccountData {
            address: vec![1, 2, 3],
            balance: 1000,
            nonce: 5,
            network: BitcoinNetwork::BitcoinTestnet,
        };

        assert_eq!(account.balance, 1000);
        assert_eq!(account.network, BitcoinNetwork::BitcoinTestnet);
    }
}