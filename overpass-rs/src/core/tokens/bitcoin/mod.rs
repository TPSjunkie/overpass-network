// ./src/core/tokens/bitcoin/mod.rs
// ./src/core/tokens/bitcoin/mod.rs

pub mod bitcoin_integration;
pub mod bitcoin_transaction;
pub mod bitcoin_manager;
pub mod bitcoin_zkp_manager;  // Zero-knowledge proof manager
pub mod bitcoin_types;
pub mod bitcoin_proof;
pub mod bitcoin_zkp_circuits;

pub use bitcoin_integration::{Bitcoin, BitcoinConfig};
pub use bitcoin_transaction::BitcoinTransaction;
pub use bitcoin_proof::BitcoinProofBoc;
pub use bitcoin_zkp_manager::BitcoinZkpManager;  // Export the ZKP manager
pub use bitcoin_types::{
    BitcoinNetwork, 
    BitcoinError, 
    BitcoinTransactionData, 
    BitcoinAccountData
};
pub use bitcoin_zkp_circuits::BitcoinZkpCircuits;

