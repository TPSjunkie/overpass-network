// ./src/core/hierarchy/client/channel/bitcoin_channel.rs

use bitcoin_hashes::{Hash};
use secp256k1::{PublicKey, Secp256k1};
use bitcoin::{Network, Transaction, TxIn, TxOut, Script};
use secp256k1::{SecretKey, Secp256k1};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use crate::core::hierarchy::client::channel::bitcoin_channel::{
    BitcoinChannel, BitcoinChannelState, ChannelError, ChannelStatus
};
use crate::core::zkps::bitcoin::bitcoin_proof::{
    BitcoinProofBundle, BitcoinProofType, BitcoinZkProof
};
use crate::core::hierarchy::client::channel::merkle::HierarchicalSparseMerkleTree;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinWalletState {
    pub balance: u64,
    pub nonce: u64,
    pub locked_balance: u64,
    pub merkle_root: [u8; 32],
    pub last_update: u64,
    pub utxo_set: HashSet<BitcoinUTXO>,
    pub block_height: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct BitcoinUTXO {
    pub txid: [u8; 32],
    pub vout: u32,
    pub amount: u64,
    pub script_pubkey: Vec<u8>,
    pub confirmation_height: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinWalletConfig {
    pub network: Network,
    pub min_confirmation: u32,
    pub dust_threshold: u64,
    pub fee_rate: u64,
    pub max_channel_size: u64,
    pub min_channel_size: u64,
    pub max_channels: u32,
    pub dispute_period: u32,
}

pub struct BitcoinWallet {
    pub wallet_id: [u8; 32],
    pub network: Network,
    pub pubkey: PublicKey,
    pub state: RwLock<BitcoinWalletState>,
    pub channels: RwLock<HashMap<[u8; 32], Arc<RwLock<BitcoinChannel>>>>,
    pub state_tree: Arc<RwLock<HierarchicalSparseMerkleTree>>,
    pub config: BitcoinWalletConfig,
    pub nullifier_set: RwLock<HashSet<[u8; 32]>>,
}

#[derive(Debug)]
pub enum WalletError {
    InsufficientFunds,
    InvalidState,
    InvalidProof,
    ChannelError(ChannelError),
    InvalidTransaction,
    InvalidSignature,
    ChannelNotFound,
    UTXONotFound,
    InvalidAmount,
    MaxChannelsReached,
    InvalidChannelSize,
}

impl BitcoinWallet {
    pub fn new(
        secret_key: &SecretKey,
        network: Network,
        config: BitcoinWalletConfig,
        initial_block_height: u32,
    ) -> Result<Self, WalletError> {
        let secp = Secp256k1::new();
        let pubkey = PublicKey::from_secret_key(&secp, secret_key);

        // Generate wallet ID
        let mut wallet_id = [0u8; 32];
        let mut hasher = sha2::Sha256::new();
        hasher.update(pubkey.serialize());
        wallet_id.copy_from_slice(&hasher.finalize());

        // Initialize state tree
        let state_tree = HierarchicalSparseMerkleTree::new(32);
        let merkle_root = state_tree.root();

        let initial_state = BitcoinWalletState {
            balance: 0,
            nonce: 0,
            locked_balance: 0,
            merkle_root,
            last_update: current_timestamp(),
            utxo_set: HashSet::new(),
            block_height: initial_block_height,
        };

        Ok(Self {
            wallet_id,
            network,
            pubkey,
            state: RwLock::new(initial_state),
            channels: RwLock::new(HashMap::new()),
            state_tree: Arc::new(RwLock::new(state_tree)),
            config,
            nullifier_set: RwLock::new(HashSet::new()),
        })
    }

    pub fn create_channel(
        &self,
        receiver_pubkey: &PublicKey,
        capacity: u64,
        fee: u64,
    ) -> Result<([u8; 32], Transaction), WalletError> {
        // Validate channel size
        if capacity < self.config.min_channel_size || capacity > self.config.max_channel_size {
            return Err(WalletError::InvalidChannelSize);
        }

        // Check if max channels reached
        let channels = self.channels.read().unwrap();
        if channels.len() >= self.config.max_channels as usize {
            return Err(WalletError::MaxChannelsReached);
        }
        drop(channels);

        // Check available balance
        let state = self.state.read().unwrap();
        let required_amount = capacity + fee;
        if state.balance < required_amount {
            return Err(WalletError::InsufficientFunds);
        }

        // Create channel
        let channel = BitcoinChannel::new(
            self.network,
            capacity,
            &self.get_secret_key()?, // You'll need to implement secure key management
            receiver_pubkey,
            state.block_height,
        ).map_err(WalletError::ChannelError)?;

        // Create funding transaction
        let funding_tx = self.create_funding_transaction(&channel, capacity, fee)?;

        // Store channel
        let channel_id = channel.channel_id;
        let mut channels = self.channels.write().unwrap();
        channels.insert(channel_id, Arc::new(RwLock::new(channel)));

        // Update wallet state
        let mut state = self.state.write().unwrap();
        state.balance -= required_amount;
        state.locked_balance += capacity;

        Ok((channel_id, funding_tx))
    }

    pub fn update_channel(
        &self,
        channel_id: [u8; 32],
        new_state: BitcoinChannelState,
        proof_bundle: BitcoinProofBundle,
    ) -> Result<(), WalletError> {
        // Verify channel exists
        let channels = self.channels.read().unwrap();
        let channel = channels.get(&channel_id)
            .ok_or(WalletError::ChannelNotFound)?;

        // Get channel lock
        let mut channel = channel.write().unwrap();

        // Update channel state
        channel.update_state(new_state, proof_bundle)
            .map_err(WalletError::ChannelError)?;

        Ok(())
    }

    pub fn close_channel(
        &self,
        channel_id: [u8; 32],
        recipient_script: Script,
    ) -> Result<Transaction, WalletError> {
        let channels = self.channels.read().unwrap();
        let channel = channels.get(&channel_id)
            .ok_or(WalletError::ChannelNotFound)?;
        
        let channel = channel.read().unwrap();
        if channel.status != ChannelStatus::Active {
            return Err(WalletError::InvalidState);
        }

        // Calculate fee
        let fee = self.calculate_transaction_fee(1, 1); // 1 input, 1 output

        // Create closure transaction
        let closure_tx = channel.create_closure_transaction(fee, recipient_script)
            .map_err(WalletError::ChannelError)?;

        Ok(closure_tx)
    }

    pub fn process_utxo(
        &self,
        utxo: BitcoinUTXO,
        proof_bundle: BitcoinProofBundle,
    ) -> Result<(), WalletError> {
        // Verify proof
        self.verify_utxo_proof(&utxo, &proof_bundle)?;

        // Add UTXO to set
        let mut state = self.state.write().unwrap();
        state.utxo_set.insert(utxo.clone());
        state.balance += utxo.amount;

        Ok(())
    }

    pub fn create_transaction(
        &self,
        recipient: &PublicKey,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction, WalletError> {
        let state = self.state.read().unwrap();
        
        if amount + fee > state.balance {
            return Err(WalletError::InsufficientFunds);
        }

        // Select UTXOs
        let selected_utxos = self.select_utxos(amount + fee)?;
        let total_input = selected_utxos.iter().map(|u| u.amount).sum::<u64>();

        // Create transaction
        let transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: selected_utxos.iter().map(|utxo| TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: utxo.txid.into(),
                    vout: utxo.vout,
                },
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }).collect(),
            output: vec![
                TxOut {
                    value: amount,
                    script_pubkey: Script::new_p2pkh(&recipient.pubkey_hash()),
                },
                // Change output
                TxOut {
                    value: total_input - amount - fee,
                    script_pubkey: Script::new_p2pkh(&self.pubkey.pubkey_hash()),
                },
            ],
        };

        Ok(transaction)
    }

    fn create_funding_transaction(
        &self,
        channel: &BitcoinChannel,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction, WalletError> {
        let selected_utxos = self.select_utxos(amount + fee)?;
        let total_input = selected_utxos.iter().map(|u| u.amount).sum::<u64>();

        let transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: selected_utxos.iter().map(|utxo| TxIn {
                previous_output: bitcoin::OutPoint {
                    txid: utxo.txid.into(),
                    vout: utxo.vout,
                },
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }).collect(),
            output: vec![
                TxOut {
                    value: amount,
                    script_pubkey: channel.htlc_script.clone(),
                },
                // Change output
                TxOut {
                    value: total_input - amount - fee,
                    script_pubkey: Script::new_p2pkh(&self.pubkey.pubkey_hash()),
                },
            ],
        };

        Ok(transaction)
    }

    fn select_utxos(&self, required_amount: u64) -> Result<Vec<BitcoinUTXO>, WalletError> {
        let state = self.state.read().unwrap();
        let mut selected = Vec::new();
        let mut total = 0;

        for utxo in &state.utxo_set {
            if utxo.confirmation_height + self.config.min_confirmation <= state.block_height {
                selected.push(utxo.clone());
                total += utxo.amount;
                if total >= required_amount {
                    break;
                }
            }
        }

        if total < required_amount {
            return Err(WalletError::InsufficientFunds);
        }

        Ok(selected)
    }

    fn verify_utxo_proof(
        &self,
        utxo: &BitcoinUTXO,
        proof_bundle: &BitcoinProofBundle,
    ) -> Result<(), WalletError> {
        // Verify proof type
        if proof_bundle.metadata.proof_type != BitcoinProofType::StateTransition {
            return Err(WalletError::InvalidProof);
        }

        // Verify nullifier not used
        let nullifier_set = self.nullifier_set.read().unwrap();
        if nullifier_set.contains(&proof_bundle.proof.nullifier) {
            return Err(WalletError::InvalidProof);
        }

        // Verify proof
        // This would use your proof verification system
        self.verify_proof(&proof_bundle.proof)?;

        Ok(())
    }

    fn verify_proof(&self, proof: &BitcoinZkProof) -> Result<(), WalletError> {
        // Implement actual proof verification
        // This would connect to your proof verification system
        Ok(())
    }

    fn calculate_transaction_fee(&self, inputs: u32, outputs: u32) -> u64 {
        let tx_size = (inputs * 148 + outputs * 34 + 10) as u64;
        tx_size * self.config.fee_rate
    }

    fn get_secret_key(&self) -> Result<SecretKey, WalletError> {
        // Implement secure key management
        unimplemented!("Secure key management not implemented")
    }
}

fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;
    use secp256k1::{SecretKey, Secp256k1};

    fn create_test_wallet() -> (BitcoinWallet, SecretKey) {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        
        let config = BitcoinWalletConfig {
            network: Network::Testnet,
            min_confirmation: 6,
            dust_threshold: 546,
            fee_rate: 1,
            max_channel_size: 1_000_000_000,
            min_channel_size: 100_000,
            max_channels: 100,
            dispute_period: 144,
        };

        let wallet = BitcoinWallet::new(
            &secret_key,
            Network::Testnet,
            config,
            100_000,
        ).unwrap();

        (wallet, secret_key)
    }

    #[test]
    fn test_wallet_creation() {
        let (wallet, _) = create_test_wallet();
        assert_eq!(wallet.network, Network::Testnet);
        
        let state = wallet.state.read().unwrap();
        assert_eq!(state.balance, 0);
        assert_eq!(state.locked_balance, 0);
    }

    #[test]
    fn test_channel_creation() {
        let (wallet, _) = create_test_wallet();
        let receiver_secret_key = SecretKey::new(&mut rand::thread_rng());
        let receiver_pubkey = PublicKey::from_secret_key(
            &Secp256k1::new(),
            &receiver_secret_key,
        );

// Add some balance first
let mut state = wallet.state.write().unwrap();
state.balance = 1_000_000;
drop(state);

let result = wallet.create_channel(
    &receiver_pubkey,
    500_000,
    1000,
);

assert!(result.is_ok());
let (channel_id, funding_tx) = result.unwrap();

let channels = wallet.channels.read().unwrap();
assert!(channels.contains_key(&channel_id));
assert_eq!(funding_tx.output[0].value, 500_000);
}

#[test]
fn test_utxo_processing() {
    let (wallet, _) = create_test_wallet();

    let utxo = BitcoinUTXO {
        txid: [0u8; 32],
        vout: 0,
        amount: 100_000,
        script_pubkey: vec![],
        confirmation_height: 100_000,
    };

    let proof_bundle = BitcoinProofBundle {
        proof: BitcoinZkProof {
            proof_data: vec![],
            public_inputs: vec![],
            merkle_root: [0u8; 32],
            timestamp: current_timestamp(),
            btc_block_height: 100_000,
            funding_txid: [0u8; 32],
            output_index: 0,
            htlc_script: vec![],
            nullifier: [0u8; 32],
        },
        metadata: ProofBundleMetadata {
            proof_type: BitcoinProofType::StateTransition,
            channel_id: [0u8; 32],
            created_at: current_timestamp(),
            verified_at: None,
            btc_block_height: 100_000,
            htlc_timelock: 0,
            commitment_nullifier: [0u8; 32],
            merkle_root: [0u8; 32],
            height_bounds: (0, 0),
        },
    };

    let result = wallet.process_utxo(utxo.clone(), proof_bundle);
    assert!(result.is_ok());

    let state = wallet.state.read().unwrap();
    assert!(state.utxo_set.contains(&utxo));
    assert_eq!(state.balance, 100_000);
}
#[test]
fn test_transaction_creation() {
let (wallet, _) = create_test_wallet();
let recipient_secret_key = SecretKey::new(&mut rand::thread_rng());
let recipient_pubkey = PublicKey::from_secret_key(
    &Secp256k1::new(),
    &recipient_secret_key,
);

// Add UTXO
let mut state = wallet.state.write().unwrap();
state.utxo_set.insert(BitcoinUTXO {
    txid: [0u8; 32],
    vout: 0,
    amount: 100_000,
    script_pubkey: vec![],
    confirmation_height: 100_000 - 7, // Confirmed
});
state.balance = 100_000;
drop(state);

let result = wallet.create_transaction(
    &recipient_pubkey,
    50_000,
    1000,
);

assert!(result.is_ok());
let tx = result.unwrap();
assert_eq!(tx.output[0].value, 50_000);
assert_eq!(tx.output[1].value, 49_000); // Change minus fee
}
}

#[test]
fn test_channel_closure() {
    let (wallet, secret_key) = super::create_test_wallet();
    let receiver_secret_key = SecretKey::new(&mut rand::thread_rng());
    let receiver_pubkey = PublicKey::from_secret_key(
        &Secp256k1::new(),
        &receiver_secret_key,
    );

    // Create channel first
    let mut state = wallet.state.write().unwrap();
    state.balance = 1_000_000;
    drop(state);

    let (channel_id, _) = wallet.create_channel(
        &receiver_pubkey,
        500_000,
        1000,
    ).unwrap();

    // Create closure transaction
    let recipient_script = Script::new_p2pkh(&receiver_pubkey.pubkey_hash());
    let result = wallet.close_channel(channel_id, recipient_script);

    assert!(result.is_ok());
    let closure_tx = result.unwrap();
    assert_eq!(closure_tx.input.len(), 1);
    assert_eq!(closure_tx.output.len(), 1);
}
   #[test]
   fn test_utxo_selection() {
       let (wallet, _) = create_test_wallet();

       // Add multiple UTXOs
       let mut state = wallet.state.write().unwrap();
       state.utxo_set.insert(BitcoinUTXO {
           txid: [1u8; 32],
           vout: 0,
           amount: 50_000,
           script_pubkey: vec![],
           confirmation_height: 100_000 - 7,
       });
       state.utxo_set.insert(BitcoinUTXO {
           txid: [2u8; 32],
           vout: 1,
           amount: 30_000,
           script_pubkey: vec![],
           confirmation_height: 100_000 - 7,
       });
       state.balance = 80_000;
       drop(state);

       let result = wallet.select_utxos(70_000);
       assert!(result.is_ok());
       let selected = result.unwrap();
       assert_eq!(selected.len(), 2);
       assert!(selected.iter().map(|u| u.amount).sum::<u64>() >= 70_000);
   }

   #[test]
   fn test_insufficient_funds() {
       let (wallet, _) = create_test_wallet();
       let receiver_secret_key = SecretKey::new(&mut rand::thread_rng());
       let receiver_pubkey = PublicKey::from_secret_key(
           &Secp256k1::new(),
           &receiver_secret_key,
       );

       let result = wallet.create_transaction(
           &receiver_pubkey,
           100_000, // More than available
           1000,
       );

       assert!(matches!(result, Err(WalletError::InsufficientFunds)));
   }

   #[test]
   fn test_invalid_channel_size() {
       let (wallet, _) = create_test_wallet();
       let receiver_secret_key = SecretKey::new(&mut rand::thread_rng());
       let receiver_pubkey = PublicKey::from_secret_key(
           &Secp256k1::new(),
           &receiver_secret_key,
       );

       // Add balance
       let mut state = wallet.state.write().unwrap();
       state.balance = 2_000_000_000;
       drop(state);

       // Try to create channel larger than max size
       let result = wallet.create_channel(
           &receiver_pubkey,
           1_500_000_000,
           1000,
       );

       assert!(matches!(result, Err(WalletError::InvalidChannelSize)));
   }

   #[test]
   fn test_max_channels() {
       let (wallet, _) = create_test_wallet();
       let receiver_secret_key = SecretKey::new(&mut rand::thread_rng());
       let receiver_pubkey = PublicKey::from_secret_key(
           &Secp256k1::new(),
           &receiver_secret_key,
       );

       // Add balance
       let mut state = wallet.state.write().unwrap();
       state.balance = 10_000_000_000;
       drop(state);

       // Create max number of channels
       for _ in 0..wallet.config.max_channels {
           let result = wallet.create_channel(
               &receiver_pubkey,
               500_000,
               1000,
           );
           assert!(result.is_ok());
       }

       // Try to create one more channel
       let result = wallet.create_channel(
           &receiver_pubkey,
           500_000,
           1000,
       );

       assert!(matches!(result, Err(WalletError::MaxChannelsReached)));
   }
   