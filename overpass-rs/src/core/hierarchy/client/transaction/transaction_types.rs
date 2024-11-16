// ./src/core/hierarchy/client/transaction/transaction_types.rs

use crate::core::hierarchy::client::wallet_extension::wallet_extension_types::WalletExtension;
use crate::core::zkps::plonky2::Plonky2SystemHandle;
use crate::core::zkps::proof::ZkProof;
use crate::core::error::errors::SystemError;
use crate::core::hierarchy::client::channel::channel_contract::ChannelContract;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ProofMetadata;

use ed25519_dalek::Signature;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

/// Represents a transaction in the Overpass Channels system
#[derive(Clone)]
pub struct TransactionOCData {
    pub transaction_id: [u8; 32],
    pub transaction_type: TransactionType,
    pub wallet_extension: Arc<RwLock<WalletExtension>>,
    pub channel_contract: Arc<RwLock<ChannelContract>>,
    pub proof: ZkProof,
}

impl TransactionOCData {
    /// Creates a new transaction in the Overpass Channels system
    pub fn new(
        transaction_id: [u8; 32],
        transaction_type: TransactionType,
        wallet_extension: Arc<RwLock<WalletExtension>>,
        channel_contract: Arc<RwLock<ChannelContract>>,
        proof: ZkProof,
    )