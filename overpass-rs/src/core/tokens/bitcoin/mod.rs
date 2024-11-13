// ./src/core/tokens/bitcoin/mod.rs
// ./src/core/tokens/bitcoin/mod.rs

pub mod bitcoin_integration;
pub mod bitcoin_transaction;
pub mod bitcoin_manager;
pub mod bitcoin_zkp_manager;  // Zero-knowledge proof manager
pub mod bitcoin_types;
pub mod bitcoin_zkp_circuits;

pub use bitcoin_integration::{Bitcoin, BitcoinConfig};
pub use bitcoin_transaction::BitcoinTransaction;
pub use bitcoin_manager::BitcoinManager;
pub use bitcoin_zkp_manager::BitcoinZkpManager;  // Export the ZKP manager
pub use bitcoin_types::{
    BitcoinNetwork, 
    BitcoinError, 
    BitcoinTransactionData, 
    BitcoinAccountData
};
pub use bitcoin_zkp_circuits::BitcoinZkpCircuits;

// Re-export proof-related types for easier access
pub use crate::core::zkps::proof::ZkProof;
pub use crate::core::zkps::plonky2::Plonky2SystemHandle;

pub use crate::core::zkps::proof::{
    ProofMetadata,
    ProofType};

pub use crate::core::zkps::plonky2::Plonky2System;  // Export Plonky2 system



// Each file's primary responsibilities:

// 1. bitcoin_integration.rs
// - BitcoinConfig trait
// - BitcoinNetwork enum
// - Core Bitcoin struct
// - Basic Bitcoin operations (deposit, withdraw, slash)
// - Network selection and configuration

// 2. bitcoin_transaction.rs
// - Transaction structures and traits
// - Transaction validation
// - Transaction signing
// - Transaction metadata

// 3. bitcoin_manager.rs
// - High-level Bitcoin operations
// - Balance management
// - User account operations
// - Network interface

// 4. bitcoin_zkp_manager.rs
// - Integration of Bitcoin ops with ZK proofs
// - Proof generation for transactions
// - Proof verification
// - State transition validation

// 5. bitcoin_types.rs
// - Common types used across Bitcoin modules
// - Balance types
// - Account types
// - Network types
// - Error types

// 6. bitcoin_zkp_circuits.rs
// - Circuits for Bitcoin zero-knowledge proofs
// - Helper functions for cryptographic operations


// Usage in other files: