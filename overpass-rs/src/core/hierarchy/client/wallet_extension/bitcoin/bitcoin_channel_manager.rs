// src/core/hierarchy/client/channel/bitcoin_channel_manager.rs
use bitcoin::{
    Network, OutPoint, Script, Transaction, TxIn, TxOut,
    psbt::Psbt,
};
use bitcoin::hashes::{sha256d, Hash};
use secp256k1::{SecretKey, PublicKey, Secp256k1};

use crate::core::error::errors::SystemError;
use crate::core::hierarchy::client::wallet_extension::bitcoin::bitcoin_wallet_extension_types::{
    BitcoinChannelConfig, BitcoinTxStatus
};
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

    /// Accepts an incoming channel open request
    pub fn accept_channel(
        &mut self,
        channel_id: [u8; 32],
        their_funding_pubkey: PublicKey,
        their_basepoints: ChannelBasepoints,
    ) -> Result<(), SystemError> {
        let channel = self.get_channel_mut(&channel_id)?;
        
        if channel.status != ChannelStatus::Opening {
            return Err(SystemError::new_string("Channel not in opening state"));
        }

        // Verify their basepoints
        if !self.verify_basepoints(&their_basepoints)? {
            return Err(SystemError::new_string("Invalid basepoints"));
        }

        // Update channel state with their information
        // ... implementation continues in next section

        Ok(())
    }
}

pub struct ChannelBasepoints {
    pub revocation: PublicKey,
    pub payment: PublicKey,
    pub delayed_payment: PublicKey,
    pub htlc: PublicKey,
}

// src/core/hierarchy/client/channel/bitcoin_channel_manager.rs (continued)

impl BitcoinChannelManager {
    /// Continues accepting channel open request by setting up commitment transactions
    fn setup_initial_commitments(
        &mut self,
        channel_id: &[u8; 32],
        their_funding_pubkey: &PublicKey,
    ) -> Result<(Transaction, Transaction), SystemError> {
        let channel = self.get_channel_mut(channel_id)?;
        
        // Generate initial commitment transactions for both sides
        let (our_commit_tx, our_commit_sig) = self.create_commitment_transaction(
            &channel,
            true, // is_local
            None, // no HTLCs yet
        )?;

        let (their_commit_tx, their_commit_sig) = self.create_commitment_transaction(
            &channel,
            false, // is_remote
            None,  // no HTLCs yet
        )?;

        // Store commitment transactions
        {
            let mut channel = channel.write().map_err(|_| {
                SystemError::new_string("Failed to acquire channel write lock")
            })?;
            channel.latest_commitment_tx = Some(our_commit_tx.clone());
            
            // Store revocation key for this commitment
            let revocation_key = self.generate_revocation_key(&channel.state.revocation_basepoint)?;
            channel.revocation_keys.push(revocation_key);
        }

        Ok((our_commit_tx, their_commit_tx))
    }

    /// Creates a commitment transaction for the channel
    fn create_commitment_transaction(
        &self,
        channel: &Arc<RwLock<BitcoinChannel>>,
        is_local: bool,
        htlcs: Option<Vec<HTLC>>,
    ) -> Result<(Transaction, Vec<u8>), SystemError> {
        let channel = channel.read().map_err(|_| {
            SystemError::new_string("Failed to acquire channel read lock")
        })?;

        // Get funding outpoint
        let funding_outpoint = OutPoint {
            txid: bitcoin::Txid::from_slice(&channel.state.funding_txid)
                .map_err(|_| SystemError::new_string("Invalid funding txid"))?,
            vout: channel.state.funding_output_index,
        };

        // Create transaction input spending from funding tx
        let input = TxIn {
            previous_output: funding_outpoint,
            script_sig: Script::new(),
            sequence: 0xFFFFFFFF,
            witness: vec![],
        };

        // Create to_local output
        let mut outputs = Vec::new();
        let to_local_amount = if is_local {
            channel.state.balance_msat / 1000
        } else {
            channel.state.funding_amount - (channel.state.balance_msat / 1000)
        };

        if to_local_amount >= channel.config.dust_limit_satoshis {
            outputs.push(TxOut {
                value: to_local_amount,
                script_pubkey: self.create_to_local_script(&channel, is_local)?,
            });
        }

        // Add HTLC outputs
        if let Some(htlcs) = htlcs {
            for htlc in htlcs {
                if htlc.amount_msat / 1000 >= channel.config.dust_limit_satoshis {
                    outputs.push(self.create_htlc_output(&channel, &htlc, is_local)?);
                }
            }
        }

        // Create the transaction
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![input],
            output: outputs,
        };


        // Sign the transaction
        let sig = self.sign_commitment_transaction(&channel, &tx, is_local)?;

        Ok((tx, sig))
    }

    /// Creates the script for the to_local output in a commitment transaction
    fn create_to_local_script(&self, channel: &BitcoinChannel, is_local: bool) -> Result<Script, SystemError> {
        // Get remote basepoints
        let revocation_pubkey = if is_local {
            channel.state.revocation_basepoint
        } else {
            channel
                .state
                .remote_revocation_basepoint
                .ok_or_else(|| SystemError::new_string("Missing remote revocation basepoint"))?
        };
        let delayed_pubkey = if is_local {
            channel.state.delayed_payment_basepoint
        } else {
            channel.state.remote_delayed_payment_basepoint.ok_or_else(|| 
                SystemError::new_string("Missing remote delayed basepoint"))?
        };
        // Create CSV-based revocable output script
        let script = Script::from(vec![0x76, 0x88, 0xAC]);

            .push_int(channel.config.to_self_delay as i64)
            .push_opcode(bitcoin::opcodes::all::OP_CHECKSEQUENCEVERIFY)
            .push_opcode(bitcoin::opcodes::all::OP_DROP)
            .push_key(&delayed_pubkey)
            .push_opcode(bitcoin::opcodes::all::OP_CHECKSIGVERIFY)
            .push_key(&revocation_pubkey)
            .push_opcode(bitcoin::opcodes::all::OP_CHECKSIG)
            .into_script();
        Ok(script)
    }

    /// Creates an output for an HTLC in a commitment transaction    
    fn create_htlc_output(&self, channel: &BitcoinChannel, htlc: &HTLC, is_local: bool) -> Result<TxOut, SystemError> {
        let htlc_pubkey = if is_local {
            channel.state.htlc_basepoint
        } else {
            channel.state.remote_htlc_basepoint.ok_or_else(|| 
                SystemError::new_string("Missing remote HTLC basepoint"))?
        };
        let script = match htlc.direction {
            HTLCDirection::Offered => self.create_offered_htlc_script(htlc, &htlc_pubkey)?,
            HTLCDirection::Received => self.create_received_htlc_script(htlc, &htlc_pubkey)?,
        };
        Ok(TxOut {
            value: htlc.amount_msat / 1000,
            script_pubkey: script,
        })
    }
    /// Creates the script for an offered HTLC
    fn create_offered_htlc_script(&self, htlc: &HTLC, htlc_pubkey: &PublicKey) -> Result<Script, SystemError> {
        // Create script
        let script = Script::builder()
            .push_opcode(bitcoin::opcodes::all::OP_DUP)
            .push_opcode(bitcoin::opcodes::all::OP_HASH160)
            .push_slice(&htlc.payment_hash)
            .push_opcode(bitcoin::opcodes::all::OP_EQUAL)
            .push_opcode(bitcoin::opcodes::all::OP_IF)
            // Payment path
            .push_key(htlc_pubkey)
            .push_opcode(bitcoin::opcodes::all::OP_ELSE)
            // Timeout path
            .push_int(htlc.cltv_expiry as i64)
            .push_opcode(bitcoin::opcodes::all::OP_CHECKLOCKTIMEVERIFY)
            .push_opcode(bitcoin::opcodes::all::OP_DROP)
            .push_key(htlc_pubkey)
            .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
            .push_opcode(bitcoin::opcodes::all::OP_CHECKSIG)
            .into_script();
        Ok(script)
    }
        /// Creates the script for a received HTLC
    fn create_received_htlc_script(&self, htlc: &HTLC, htlc_pubkey: &PublicKey) -> Result<Script, SystemError> {
        // Create script
        let script = Script::builder()  
            .push_opcode(bitcoin::opcodes::all::OP_IF) // Payment path
            .push_slice(&htlc.payment_hash[..]) // Payment path
            .push_opcode(bitcoin::opcodes::all::OP_ELSE) // Payment path
            .push_int(htlc.cltv_expiry as i64) // Payment path
            .push_opcode(bitcoin::opcodes::all::OP_CHECKLOCKTIMEVERIFY) // Payment path
            .push_opcode(bitcoin::opcodes::all::OP_DROP) // Payment path
            .push_key(htlc_pubkey) // Payment path
            .push_opcode(bitcoin::opcodes::all::OP_ENDIF) // Payment path
            .push_opcode(bitcoin::opcodes::all::OP_CHECKSIG) // Payment path
            .into_script();
        Ok(script)
    }
      /// Signs a transaction using the channel keys    
    fn sign_transaction(
        &self,
        channel: &BitcoinChannel,
        tx: &Transaction,
        input_index: usize,
    ) -> Result<Signature, SystemError> {
        // Create sighash cache
        let mut sighash = SighashCache::new(tx);
        
        // Get the sighash
        let sighash = sighash
            .legacy_signature_hash(
                input_index,
                &channel.state.funding_txid,
                EcdsaSighashType::All,
            )
            .map_err(|_| SystemError::new_string("Failed to compute signature hash"))?;

        // Sign with channel key
        let message = Message::from_slice(&sighash)
            .map_err(|_| SystemError::new_string("Failed to create message from sighash"))?;
            
        let signature = self.secp.sign_ecdsa(&message, &channel.state.local_keys.funding_key)
            .map_err(|_| SystemError::new_string("Failed to create signature"))?;

        Ok(signature)
    }
    /// Creates a PSBT from a transaction
    fn create_psbt(
        &self,
        tx: &Transaction,
        inputs: Vec<TxIn>,
        outputs: Vec<TxOut>,
        fee: u64,
    ) -> Result<Psbt, SystemError> {
        // Create PSBT
        let psbt = Psbt::new();
        psbt.set_version(2);
        psbt.set_tx_in(inputs);
        psbt.set_tx_out(outputs);
        psbt.set_locktime(0);
        psbt.set_fee(fee);
        psbt.set_tx_name("Overpass Channels");
        Ok(psbt)
    }   
    /// Signs a PSBT
    fn sign_psbt(
        &self,
        psbt: &mut Psbt,
        input_index: usize,
        key: &SecretKey,
    ) -> Result<(), SystemError> {
        // Sign PSBT
        psbt.sign(&input_index, &key)?;
        Ok(())
    }
    /// Finalizes a PSBT
    fn finalize_psbt(
        &self,
        psbt: &mut Psbt,
    ) -> Result<Psbt, SystemError> {
        // Finalize PSBT
        psbt.finalize()
            .map_err(|e| SystemError::new_string(&format!("Failed to finalize PSBT: {}", e)))?;
        Ok(psbt.clone())
    }
    /// Broadcasts a transaction
    fn broadcast_transaction(
        &self,
        tx: &Transaction,
    ) -> Result<(), SystemError> {
        // Serialize transaction
        let tx_bytes = bitcoin::consensus::encode::serialize(tx);
        
        // Send transaction to Bitcoin network via RPC
        self.rpc_client
            .send_raw_transaction(&tx_bytes)
            .map_err(|e| SystemError::new_string(&format!("Failed to broadcast transaction: {}", e)))?;
            
        // Wait for confirmation
        self.rpc_client
            .wait_for_confirmation(&tx.txid(), 1)
            .map_err(|e| SystemError::new_string(&format!("Failed to confirm transaction: {}", e)))?;
            
        Ok(())
    }  
/// Creates a PSBT from a transaction
fn create_psbt(
    &self,
    tx: &Transaction,
    inputs: Vec<TxIn>,
    outputs: Vec<TxOut>,
    fee: u64,
) -> Result<Psbt, SystemError> {
    // Create PSBT
    let psbt = Psbt::new();
    psbt.set_version(2);
    psbt.set_tx_in(inputs);
    psbt.set_tx_out(outputs);
    psbt.set_locktime(0);
    psbt.set_fee(fee);
    psbt.set_tx_name("Overpass Channels");
    Ok(psbt)
}   
/// Signs a PSBT
fn sign_psbt(
    &self,
    psbt: &Psbt,
    input_index: usize,
    key: &SecretKey,
) -> Result<Psbt, SystemError> {
    // Sign PSBT
    psbt.sign(&input_index, &key)?;
    Ok(())
}
/// Finalizes a PSBT
fn finalize_psbt(
    &self,
    psbt: &Psbt,
) -> Result<Psbt, SystemError> {
    // Finalize PSBT
    psbt.finalize();
    Ok(psbt)
}
/// Broadcasts a transaction
fn broadcast_transaction(
    &self,
    tx: &Transaction,
) -> Result<(), SystemError> {
    // Serialize transaction to bytes
    let tx_bytes = tx.serialize();
    
    // Send transaction to Bitcoin network via RPC
    self.rpc_client
        .send_raw_transaction(&tx_bytes)
        .map_err(|e| SystemError::new_string(&format!("Failed to broadcast transaction: {}", e)))?;
        
    // Wait for confirmation
    self.rpc_client
        .wait_for_confirmation(&tx.txid(), 1)
        .map_err(|e| SystemError::new_string(&format!("Failed to confirm transaction: {}", e)))?;
        
    Ok(())
}       
}