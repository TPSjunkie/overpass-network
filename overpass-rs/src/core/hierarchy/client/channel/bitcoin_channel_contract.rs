// src/core/hierarchy/client/channel/bitcoin_channel_manager.rs

use bitcoin::{Network, Transaction};
use bitcoin::hashes::{sha256d, Hash};
use secp256k1::{SecretKey, PublicKey, Secp256k1};

use crate::core::error::errors::SystemError;
use crate::core::hierarchy::client::wallet_extension::bitcoin::bitcoin_wallet_extension_types::BitcoinChannelConfig;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// The current channel state
#[derive(Debug, Clone)]
pub struct BitcoinChannelState {
    pub balance_msat: u64,
    pub commit_num: u64,
    pub funding_txid: [u8; 32],
    pub funding_output_index: u32,
    pub funding_amount: u64,
    pub our_reserve_balance: u64,
    pub their_reserve_balance: u64,
    pub pending_htlcs: Vec<HTLC>,
    pub commitment_tx: Option<Transaction>,
    pub revocation_basepoint: PublicKey,
    pub payment_basepoint: PublicKey,
    pub delayed_payment_basepoint: PublicKey,
    pub htlc_basepoint: PublicKey,
}

/// Represents a Hashed Time-Locked Contract
#[derive(Debug, Clone)]
pub struct HTLC {
    pub amount_msat: u64,
    pub payment_hash: [u8; 32],
    pub cltv_expiry: u32,
    pub direction: HTLCDirection,
    pub state: HTLCState,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HTLCDirection {
    Offered,
    Received,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HTLCState {
    Pending,
    Committed,
    Revoked,
    Failed,
}

pub struct BitcoinChannelManager {
    network: Network,
    secp_ctx: Secp256k1<secp256k1::All>,
    channels: HashMap<[u8; 32], Arc<RwLock<BitcoinChannel>>>,
    config: BitcoinChannelConfig,
}

pub struct BitcoinChannel {
    pub channel_id: [u8; 32],
    pub state: BitcoinChannelState,
    pub config: BitcoinChannelConfig,
    pub status: ChannelStatus,
    pub funding_tx: Option<Transaction>,
    pub latest_commitment_tx: Option<Transaction>,
    pub revocation_keys: Vec<SecretKey>,
    pub pending_updates: Vec<ChannelUpdate>,
}

#[derive(Debug, Clone)]
pub enum ChannelStatus {
    Opening,
    Active,
    Closing,
    ForceClosing,
    Closed,
}

#[derive(Debug, Clone)]
pub enum ChannelUpdate {
    AddHTLC(HTLC),
    FulfillHTLC {
        htlc_id: u64,
        payment_preimage: [u8; 32],
    },
    FailHTLC {
        htlc_id: u64,
        reason: Vec<u8>,
    },
    CommitmentSigned {
        commitment_tx: Transaction,
        htlc_signatures: Vec<Vec<u8>>,
    },
}

impl BitcoinChannelManager {
    pub fn new(network: Network, config: BitcoinChannelConfig) -> Self {
        Self {
            network,
            secp_ctx: Secp256k1::new(),
            channels: HashMap::new(),
            config,
        }
    }

    /// Opens a new payment channel with the given parameters
    pub fn open_channel(
        &mut self,
        counterparty_pubkey: PublicKey,
        funding_amount: u64,
        push_amount_msat: u64,
    ) -> Result<[u8; 32], SystemError> {
        // Validate parameters
        if funding_amount < self.config.channel_reserve_satoshis * 2 {
            return Err(SystemError::new_string("Funding amount too low"));
        }
        if push_amount_msat >= funding_amount * 1000 {
            return Err(SystemError::new_string("Push amount too high"));
        }

        // Generate channel ID
        let mut channel_id = [0u8; 32];
        let mut hasher = sha256d::Hash::engine();
        hasher.input(&counterparty_pubkey.serialize());
        hasher.input(&funding_amount.to_le_bytes());
        channel_id.copy_from_slice(&sha256d::Hash::from_engine(hasher).into_inner());

        // Create initial channel state
        let state = BitcoinChannelState {
            balance_msat: (funding_amount * 1000).saturating_sub(push_amount_msat),
            commit_num: 0,
            funding_txid: [0; 32],
            funding_output_index: 0,
            funding_amount,
            our_reserve_balance: self.config.channel_reserve_satoshis,
            their_reserve_balance: self.config.channel_reserve_satoshis,
            pending_htlcs: Vec::new(),
            commitment_tx: None,
            revocation_basepoint: self.generate_basepoint()?,
            payment_basepoint: self.generate_basepoint()?,
            delayed_payment_basepoint: self.generate_basepoint()?,
            htlc_basepoint: self.generate_basepoint()?,
        };

        // Create and store the channel
        let channel = BitcoinChannel {
            channel_id,
            state,
            config: self.config.clone(),
            status: ChannelStatus::Opening,
            funding_tx: None,
            latest_commitment_tx: None,
            revocation_keys: Vec::new(),
            pending_updates: Vec::new(),
        };

        self.channels.insert(
            channel_id,
            Arc::new(RwLock::new(channel)),
        );

        Ok(channel_id)
    }

    /// Generates a new basepoint for channel keys
    fn generate_basepoint(&self) -> Result<PublicKey, SystemError> {
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        Ok(PublicKey::from_secret_key(&self.secp_ctx, &secret_key))
    }

    /// Gets a mutable reference to a channel
    fn get_channel_mut(&self, channel_id: &[u8; 32]) -> Result<Arc<RwLock<BitcoinChannel>>, SystemError> {
        self.channels.get(channel_id)
            .cloned()
            .ok_or_else(|| SystemError::new_string("Channel not found"))
    }

    /// Accepts an incoming channel open request
    pub fn accept_channel(
        &mut self,
        channel_id: [u8; 32],
        their_funding_pubkey: PublicKey,
        their_basepoints: ChannelBasepoints,
    ) -> Result<(), SystemError> {
        let channel = self.get_channel_mut(&channel_id)?;
        
        let mut channel_guard = channel.write().map_err(|_| SystemError::new_string("Channel lock poisoned"))?;
        
        if channel_guard.status != ChannelStatus::Opening {
            return Err(SystemError::new_string("Channel not in opening state"));
        }

        // Verify their basepoints
        if !self.verify_basepoints(&their_basepoints)? {
            return Err(SystemError::new_string("Invalid basepoints"));
        }

        // Update channel state
        channel_guard.status = ChannelStatus::Active;

        Ok(())
    }

    /// Verifies the validity of channel basepoints
    fn verify_basepoints(&self, basepoints: &ChannelBasepoints) -> Result<bool, SystemError> {
        // Verify each basepoint is a valid public key on the secp256k1 curve
        if !basepoints.payment.is_valid() {
            return Ok(false);
        }
        if !basepoints.delayed_payment.is_valid() {
            return Ok(false);
        }
        if !basepoints.htlc.is_valid() {
            return Ok(false);
        }
        if !basepoints.revocation.is_valid() {
            return Ok(false);
        }

        // Verify basepoints are unique
        let points = vec![
            &basepoints.payment,
            &basepoints.delayed_payment,
            &basepoints.htlc,
            &basepoints.revocation,
        ];
        
        for i in 0..points.len() {
            for j in i+1..points.len() {
                if points[i] == points[j] {
                    return Ok(false);
                }
            }
        }

        // Verify points are not at infinity
        if basepoints.payment.is_infinity() ||
           basepoints.delayed_payment.is_infinity() ||
           basepoints.htlc.is_infinity() ||
           basepoints.revocation.is_infinity() {
            return Ok(false);
        }

        Ok(true)
    }
    /// Updates the channel state with a new commitment transaction
    pub fn update_commitment(
        &mut self,
        channel_id: [u8; 32],
        new_commitment: Transaction,
    ) -> Result<(), SystemError> {
        let channel = self.get_channel_mut(&channel_id)?;
        let mut channel_guard = channel.write().map_err(|_| SystemError::new_string("Channel lock poisoned"))?;
        
        channel_guard.latest_commitment_tx = Some(new_commitment);
        channel_guard.state.commit_num += 1;
        
        Ok(())
    }

    /// Closes a channel cooperatively
    pub fn close_channel(
        &mut self,
        channel_id: [u8; 32],
    ) -> Result<Transaction, SystemError> {
        let channel = self.get_channel_mut(&channel_id)?;
        let mut channel_guard = channel.write().map_err(|_| SystemError::new_string("Channel lock poisoned"))?;
        
        if channel_guard.status != ChannelStatus::Active {
            return Err(SystemError::new_string("Channel not active"));
        }

        channel_guard.status = ChannelStatus::Closing;
        
        // Create closing transaction
        // This would typically create a transaction that pays out the channel balance
        // to both parties according to the latest channel state
        let closing_tx = Transaction { version: 2, lock_time: 0, input: vec![], output: vec![] };
        
        Ok(closing_tx)
    }

    /// Forces a channel closure
    pub fn force_close_channel(
        &mut self,
        channel_id: [u8; 32],
    ) -> Result<Transaction, SystemError> {
        let channel = self.get_channel_mut(&channel_id)?;
        let mut channel_guard = channel.write().map_err(|_| SystemError::new_string("Channel lock poisoned"))?;
        
        channel_guard.status = ChannelStatus::ForceClosing;
        
        // Create force-closing transaction
        // This would typically broadcast the latest commitment transaction
        let force_closing_tx = channel_guard.latest_commitment_tx
            .clone()
            .ok_or_else(|| SystemError::new_string("No commitment transaction available"))?;
        
        Ok(force_closing_tx)
    }
}

pub struct ChannelBasepoints {
    pub revocation: PublicKey,
    pub payment: PublicKey,
    pub delayed_payment: PublicKey,
    pub htlc: PublicKey,
}