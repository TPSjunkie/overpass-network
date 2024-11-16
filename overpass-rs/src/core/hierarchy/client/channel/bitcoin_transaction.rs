use bitcoin_hashes::{
    Transaction, TxIn, TxOut, OutPoint, Script, SigHashType, PublicKey,
    consensus::encode::serialize, BlockHash, TxId
};
use secp256k1::{SecretKey, Signature, Secp256k1, Message};
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

use crate::core::hierarchy::client::channel::bitcoin_state::{
    BitcoinChannelState, HTLCState, HTLCStatus
};
use crate::core::zkps::bitcoin::bitcoin_proof::{BitcoinProofBundle};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinTransaction {
    pub tx_data: Transaction,
    pub signatures: Vec<TransactionSignature>,
    pub metadata: TransactionMetadata,
    pub proof_bundle: Option<BitcoinProofBundle>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionSignature {
    pub pubkey: PublicKey,
    pub signature: Signature,
    pub input_index: u32,
    pub sighash_type: SigHashType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionMetadata {
    pub tx_type: TransactionType,
    pub channel_id: [u8; 32],
    pub sequence: u64,
    pub block_height: u32,
    pub confirmation_height: Option<u32>,
    pub confirmation_hash: Option<BlockHash>,
    pub related_txids: Vec<TxId>,
    pub fee: u64,
    pub status: TransactionStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Funding,
    StateUpdate,
    HTLCCreate,
    HTLCExecute,
    HTLCRefund,
    Dispute,
    ChannelClose,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Created,
    Signed,
    Broadcasted,
    Confirmed,
    Failed,
}

#[derive(Debug)]
pub enum TransactionError {
    InsufficientFunds,
    InvalidSignature,
    InvalidScript,
    InvalidState,
    InvalidProof,
    InvalidAmount,
    InvalidSequence,
    TransactionNotFound,
    UTXONotFound,
}

pub struct TransactionBuilder {
    secp: Secp256k1<secp256k1::All>,
    secret_key: SecretKey,
    public_key: PublicKey,
    utxo_set: HashMap<OutPoint, TxOut>,
    network: bitcoin::Network,
}

impl TransactionBuilder {
    pub fn new(
        secret_key: SecretKey,
        network: bitcoin::Network,
    ) -> Self {
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        Self {
            secp,
            secret_key,
            public_key,
            utxo_set: HashMap::new(),
            network,
        }
    }

    pub fn create_funding_transaction(
        &self,
        channel_state: &BitcoinChannelState,
        inputs: Vec<OutPoint>,
        fee: u64,
    ) -> Result<BitcoinTransaction, TransactionError> {
        // Verify input UTXOs exist
        let total_input = self.verify_inputs(&inputs)?;
        
        if total_input < channel_state.balance + fee {
            return Err(TransactionError::InsufficientFunds);
        }

        // Create funding transaction
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: inputs.into_iter().map(|outpoint| TxIn {
                previous_output: outpoint,
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }).collect(),
            output: vec![
                TxOut {
                    value: channel_state.balance,
                    script_pubkey: self.create_funding_script(channel_state),
                },
                TxOut {
                    value: total_input - channel_state.balance - fee,
                    script_pubkey: Script::new_p2pkh(&self.public_key.pubkey_hash()),
                },
            ],
        };

        // Sign transaction
        let signatures = self.sign_transaction(&tx)?;

        Ok(BitcoinTransaction {
            tx_data: tx,
            signatures,
            metadata: TransactionMetadata {
                tx_type: TransactionType::Funding,
                channel_id: channel_state.channel_id,
                sequence: channel_state.sequence,
                block_height: channel_state.block_height,
                confirmation_height: None,
                confirmation_hash: None,
                related_txids: Vec::new(),
                fee,
                status: TransactionStatus::Created,
            },
            proof_bundle: None,
        })
    }

    pub fn create_htlc_transaction(
        &self,
        channel_state: &BitcoinChannelState,
        htlc_state: &HTLCState,
        fee: u64,
    ) -> Result<BitcoinTransaction, TransactionError> {
        // Create HTLC output script
        let htlc_script = self.create_htlc_script(
            htlc_state,
            channel_state.block_height + htlc_state.timelock,
        )?;

        // Create transaction
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: channel_state.metadata.closure_txid
                        .ok_or(TransactionError::TransactionNotFound)?.into(),
                    vout: 0,
                },
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            output: vec![
                TxOut {
                    value: htlc_state.amount,
                    script_pubkey: htlc_script,
                },
                TxOut {
                    value: channel_state.balance - htlc_state.amount - fee,
                    script_pubkey: Script::new_p2pkh(&self.public_key.pubkey_hash()),
                },
            ],
        };

        // Sign transaction
        let signatures = self.sign_transaction(&tx)?;

        Ok(BitcoinTransaction {
            tx_data: tx,
            signatures,
            metadata: TransactionMetadata {
                tx_type: TransactionType::HTLCCreate,
                channel_id: channel_state.channel_id,
                sequence: channel_state.sequence,
                block_height: channel_state.block_height,
                confirmation_height: None,
                confirmation_hash: None,
                related_txids: Vec::new(),
                fee,
                status: TransactionStatus::Created,
            },
            proof_bundle: None,
        })
    }

    pub fn create_htlc_execution_transaction(
        &self,
        channel_state: &BitcoinChannelState,
        htlc_state: &HTLCState,
        preimage: &[u8; 32],
        fee: u64,
    ) -> Result<BitcoinTransaction, TransactionError> {
        if htlc_state.status != HTLCStatus::Pending {
            return Err(TransactionError::InvalidState);
        }

        // Verify preimage
        let hash = hash_preimage(preimage);
        if Some(hash) != htlc_state.preimage_hash {
            return Err(TransactionError::InvalidProof);
        }

        // Create execution transaction
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: htlc_state.execution_txid
                        .ok_or(TransactionError::TransactionNotFound)?.into(),
                    vout: 0,
                },
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            output: vec![TxOut {
                value: htlc_state.amount - fee,
                script_pubkey: Script::new_p2pkh(
                    &htlc_state.recipient_pubkey
                        .ok_or(TransactionError::InvalidScript)?
                        .pubkey_hash()
                ),
            }],
        };

        // Sign transaction
        let signatures = self.sign_transaction(&tx)?;

        Ok(BitcoinTransaction {
            tx_data: tx,
            signatures,
            metadata: TransactionMetadata {
                tx_type: TransactionType::HTLCExecute,
                channel_id: channel_state.channel_id,
                sequence: channel_state.sequence,
                block_height: channel_state.block_height,
                confirmation_height: None,
                confirmation_hash: None,
                related_txids: Vec::new(),
                fee,
                status: TransactionStatus::Created,
            },
            proof_bundle: None,
        })
    }

    pub fn create_htlc_refund_transaction(
        &self,
        channel_state: &BitcoinChannelState,
        htlc_state: &HTLCState,
        fee: u64,
    ) -> Result<BitcoinTransaction, TransactionError> {
        if htlc_state.status != HTLCStatus::Pending {
            return Err(TransactionError::InvalidState);
        }

        // Verify timelock has expired
        if channel_state.block_height < htlc_state.timelock {
            return Err(TransactionError::InvalidState);
        }

        // Create refund transaction
        let tx = Transaction {
            version: 2,
            lock_time: channel_state.block_height,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: htlc_state.execution_txid
                        .ok_or(TransactionError::TransactionNotFound)?.into(),
                    vout: 0,
                },
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            output: vec![TxOut {
                value: htlc_state.amount - fee,
                script_pubkey: Script::new_p2pkh(
                    &htlc_state.refund_pubkey
                        .ok_or(TransactionError::InvalidScript)?
                        .pubkey_hash()
                ),
            }],
        };

        // Sign transaction
        let signatures = self.sign_transaction(&tx)?;

        Ok(BitcoinTransaction {
            tx_data: tx,
            signatures,
            metadata: TransactionMetadata {
                tx_type: TransactionType::HTLCRefund,
                channel_id: channel_state.channel_id,
                sequence: channel_state.sequence,
                block_height: channel_state.block_height,
                confirmation_height: None,
                confirmation_hash: None,
                related_txids: Vec::new(),
                fee,
                status: TransactionStatus::Created,
            },
            proof_bundle: None,
        })
    }

    pub fn create_closing_transaction(
        &self,
        channel_state: &BitcoinChannelState,
        recipient_script: Script,
        fee: u64,
    ) -> Result<BitcoinTransaction, TransactionError> {
        // Create closing transaction
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![TxIn {
                previous_output: OutPoint {
                    txid: channel_state.metadata.closure_txid
                        .ok_or(TransactionError::TransactionNotFound)?.into(),
                    vout: 0,
                },
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            }],
            output: vec![TxOut {
                value: channel_state.balance - fee,
                script_pubkey: recipient_script,
            }],
        };

        // Sign transaction
        let signatures = self.sign_transaction(&tx)?;

        Ok(BitcoinTransaction {
            tx_data: tx,
            signatures,
            metadata: TransactionMetadata {
                tx_type: TransactionType::ChannelClose,
                channel_id: channel_state.channel_id,
                sequence: channel_state.sequence,
                block_height: channel_state.block_height,
                confirmation_height: None,
                confirmation_hash: None,
                related_txids: Vec::new(),
                fee,
                status: TransactionStatus::Created,
            },
            proof_bundle: None,
        })
    }

    fn verify_inputs(&self, inputs: &[OutPoint]) -> Result<u64, TransactionError> {
        let mut total = 0;
        for input in inputs {
            if let Some(utxo) = self.utxo_set.get(input) {
                total += utxo.value;
            } else {
                return Err(TransactionError::UTXONotFound);
            }
        }
        Ok(total)
    }

    fn sign_transaction(
        &self,
        tx: &Transaction,
    ) -> Result<Vec<TransactionSignature>, TransactionError> {
        let mut signatures = Vec::new();

        for (index, _) in tx.input.iter().enumerate() {
            let sighash = tx.signature_hash(
                index,
                &Script::new(),
                SigHashType::All as u32,
            );

            let message = Message::from_slice(&sighash)
                .map_err(|_| TransactionError::InvalidSignature)?;

            let signature = self.secp.sign(&message, &self.secret_key);

            signatures.push(TransactionSignature {
                pubkey: self.public_key,
                signature,
                input_index: index as u32,
                sighash_type: SigHashType::All,
            });
        }

        Ok(signatures)
    }

    fn create_funding_script(&self, channel_state: &BitcoinChannelState) -> Script {
        Script::new_p2pkh(&self.public_key.pubkey_hash())
    }

    fn create_htlc_script(
        &self,
        htlc_state: &HTLCState,
        timelock: u32,
    ) -> Result<Script, TransactionError> {
        let recipient_pubkey = htlc_state.recipient_pubkey
            .ok_or(TransactionError::InvalidScript)?;
        let refund_pubkey = htlc_state.refund_pubkey
            .ok_or(TransactionError::InvalidScript)?;

        Ok(Script::builder()
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_IF)
            // Recipient's path
            .push_slice(&recipient_pubkey.serialize())
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_ELSE)
            // Sender's refund path with timelock
            .push_int(timelock as i64)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKLOCKTIMEVERIFY)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_DROP)
            .push_slice(&refund_pubkey.serialize())
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_ENDIF)
            .into_script())
    }
 }
 
 fn hash_preimage(preimage: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(preimage);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
 }
 
 #[cfg(test)]
 mod tests {
    use super::*;
    use bitcoin::Network;
 
    fn create_test_builder() -> TransactionBuilder {
        let secret_key = SecretKey::new(&mut rand::thread_rng());
        TransactionBuilder::new(secret_key, Network::Testnet)
    }
 
    fn create_test_channel_state() -> BitcoinChannelState {
        BitcoinChannelState::new(
            [0u8; 32],
            1_000_000,
            100_000,
        )
    }
 
    fn create_test_htlc_state(amount: u64, timelock: u32) -> HTLCState {
        let secp = Secp256k1::new();
        let recipient_key = SecretKey::new(&mut rand::thread_rng());
        let refund_key = SecretKey::new(&mut rand::thread_rng());
 
        HTLCState {
            status: HTLCStatus::Pending,
            amount,
            timelock,
            preimage_hash: Some([0u8; 32]),
            preimage: Some([0u8; 32]),
            recipient_pubkey: Some(PublicKey::from_secret_key(&secp, &recipient_key)),
            refund_pubkey: Some(PublicKey::from_secret_key(&secp, &refund_key)),
            signature: None,
            execution_txid: Some([0u8; 32]),
        }
    }
 
    #[test]
    fn test_funding_transaction_creation() {
        let builder = create_test_builder();
        let channel_state = create_test_channel_state();
 
        // Add test UTXO
        let outpoint = OutPoint {
            txid: [0u8; 32].into(),
            vout: 0,
        };
        builder.utxo_set.insert(outpoint, TxOut {
            value: 2_000_000,
            script_pubkey: Script::new(),
        });
 
        let result = builder.create_funding_transaction(
            &channel_state,
            vec![outpoint],
            1000,
        );
 
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.metadata.tx_type, TransactionType::Funding);
        assert_eq!(tx.tx_data.output[0].value, channel_state.balance);
    }
 
    #[test]
    fn test_htlc_transaction_creation() {
        let builder = create_test_builder();
        let mut channel_state = create_test_channel_state();
        let htlc_state = create_test_htlc_state(500_000, 144);
 
        // Set closure txid
        channel_state.metadata.closure_txid = Some([0u8; 32]);
 
        let result = builder.create_htlc_transaction(
            &channel_state,
            &htlc_state,
            1000,
        );
 
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.metadata.tx_type, TransactionType::HTLCCreate);
        assert_eq!(tx.tx_data.output[0].value, htlc_state.amount);
    }
 
    #[test]
    fn test_htlc_execution() {
        let builder = create_test_builder();
        let mut channel_state = create_test_channel_state();
        let htlc_state = create_test_htlc_state(500_000, 144);
        let preimage = [0u8; 32];
 
        // Set required txid
        channel_state.metadata.closure_txid = Some([0u8; 32]);
 
        let result = builder.create_htlc_execution_transaction(
            &channel_state,
            &htlc_state,
            &preimage,
            1000,
        );
 
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.metadata.tx_type, TransactionType::HTLCExecute);
    }
 
    #[test]
    fn test_htlc_refund() {
        let builder = create_test_builder();
        let mut channel_state = create_test_channel_state();
        let mut htlc_state = create_test_htlc_state(500_000, 144);
 
        // Set block height past timelock
        channel_state.block_height = htlc_state.timelock + 1;
        htlc_state.execution_txid = Some([0u8; 32]);
 
        let result = builder.create_htlc_refund_transaction(
            &channel_state,
            &htlc_state,
            1000,
        );
 
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.metadata.tx_type, TransactionType::HTLCRefund);
    }
 
    #[test]
    fn test_closing_transaction() {
        let builder = create_test_builder();
        let mut channel_state = create_test_channel_state();
        
        // Set closure txid
        channel_state.metadata.closure_txid = Some([0u8; 32]);
 
        let result = builder.create_closing_transaction(
            &channel_state,
            Script::new(),
            1000,
        );
 
        assert!(result.is_ok());
        let tx = result.unwrap();
        assert_eq!(tx.metadata.tx_type, TransactionType::ChannelClose);
        assert_eq!(tx.tx_data.output[0].value, channel_state.balance - 1000);
    }
 
    #[test]
    fn test_insufficient_funds() {
        let builder = create_test_builder();
        let channel_state = create_test_channel_state();
 
        // Add test UTXO with insufficient funds
        let outpoint = OutPoint {
            txid: [0u8; 32].into(),
            vout: 0,
        };
        builder.utxo_set.insert(outpoint, TxOut {
            value: 100_000, // Less than channel balance
            script_pubkey: Script::new(),
        });
 
        let result = builder.create_funding_transaction(
            &channel_state,
            vec![outpoint],
            1000,
        );
 
        assert!(matches!(result, Err(TransactionError::InsufficientFunds)));
    }
 
    #[test]
    fn test_invalid_htlc_timelock() {
        let builder = create_test_builder();
        let mut channel_state = create_test_channel_state();
        let htlc_state = create_test_htlc_state(500_000, 144);
 
        // Set block height before timelock
        channel_state.block_height = htlc_state.timelock - 1;
        channel_state.metadata.closure_txid = Some([0u8; 32]);
 
        let result = builder.create_htlc_refund_transaction(
            &channel_state,
            &htlc_state,
            1000,
        );
 
        assert!(matches!(result, Err(TransactionError::InvalidState)));
    }
 }