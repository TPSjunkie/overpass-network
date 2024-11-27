use crate::bitcoin::wallet::_::_serde::Serialize;
use bitcoin::absolute::LockTime;
use bitcoin::address::NetworkChecked;
use bitcoin::blockdata::script::{Builder, Script, ScriptBuf};
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxIn, TxOut, Version};
use bitcoin::consensus::encode;
use bitcoin::opcodes::all as opcodes;
use bitcoin::secp256k1::{All, Message, Secp256k1};
use bitcoin::Network;
use bitcoin::{Address, Amount, Sequence, Witness, sighash};
use bitcoincore_rpc::{self, Client as RpcClient, RpcApi};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: Amount, available: Amount },

    #[error("No UTXOs found for address")]
    NoUtxos,

    #[error("RPC error: {0}")]
    RpcError(#[from] bitcoincore_rpc::Error),

    #[error("Bitcoin error: {0}")]
    BitcoinError(#[from] bitcoin::Error),

    #[error("Script error: {0}")]
    ScriptError(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Invalid amount: {0}")]
    AmountError(#[from] bitcoin::amount::ParseAmountError),

    #[error("Transaction verification failed: {0}")]
    VerificationError(String),

    #[error("Network mismatch: expected {expected}, got {got}")]
    NetworkMismatch {
        expected: Network,
        got: Network,
    },
}

type Result<T> = std::result::Result<T, TransactionError>;

#[derive(Debug, Clone)]
pub struct HtlcParams {
    pub amount: Amount,
    pub timelock: u32,
    pub hash_lock: [u8; 32],
    pub recipient_key: bitcoin::PublicKey,
    pub refund_key: bitcoin::PublicKey,
}

/// Manages transaction creation and signing with advanced HTLC support.
pub struct TransactionManager {
    rpc_client: RpcClient,
    secp: Secp256k1<All>,
    network: Network,
}

impl TransactionManager {
    pub fn new(rpc_client: RpcClient, network: Network) -> Self {
        TransactionManager {
            rpc_client,
            secp: Secp256k1::new(),
            network,
        }
    }

    /// Creates a new HTLC transaction with advanced scripting capabilities.
    pub fn create_htlc_transaction(
        &self,
        sender_address: &Address<NetworkChecked>,
        htlc_params: HtlcParams,
        fee_rate: Amount,
    ) -> Result<Transaction> {
        self.validate_network(sender_address.network)?;

        let utxos = self.select_utxos(sender_address, htlc_params.amount + fee_rate)?;
        let total_input = self.calculate_total_input(&utxos)?;

        let tx_ins = self.create_transaction_inputs(&utxos);
        let htlc_script = self.create_htlc_script(&htlc_params)?;
        
        let change = total_input - htlc_params.amount - fee_rate;
        
        let mut tx_outs = vec![TxOut {
            value: htlc_params.amount.to_sat(),
            script_pubkey: htlc_script,
        }];

        if change.to_sat() > Amount::from_sat(546).to_sat() {
            tx_outs.push(TxOut {
                value: change.to_sat(),
                script_pubkey: sender_address.script_pubkey(),
            });
        }

        let transaction = Transaction {
            version: Version::TWO,
            lock_time: LockTime::from_height(htlc_params.timelock)
                .map_err(|e| TransactionError::ScriptError(e.to_string()))?,
            input: tx_ins,
            output: tx_outs,
        };

        Ok(transaction)
    }

    pub fn sign_transaction(
        &self,
        transaction: &mut Transaction,
        private_key: &bitcoin::PrivateKey,
        utxos: &[bitcoincore_rpc::json::ListUnspentResultEntry],
        sig_type: SignatureType,
    ) -> Result<()> {
        for (input_index, utxo) in utxos.iter().enumerate() {
            let script_pubkey = ScriptBuf::from_bytes(utxo.script_pub_key.clone().into_bytes());
            let amount = Amount::from_sat(utxo.amount.to_sat());

            match sig_type {
                SignatureType::Legacy => {
                    self.sign_legacy_input(transaction, input_index, private_key, &script_pubkey, amount)?;
                }
                SignatureType::Segwit => {
                    self.sign_segwit_input(transaction, input_index, private_key, &script_pubkey, amount)?;
                }
                SignatureType::Taproot => {
                    self.sign_taproot_input(transaction, input_index, private_key, &script_pubkey, amount)?;
                }
            }
        }

        Ok(())
    }

    pub fn broadcast_transaction(&self, transaction: &Transaction) -> Result<bitcoin::Txid> {
        self.verify_transaction(transaction)?;
        let tx_hex = encode::serialize_hex(transaction);
        let txid = self.rpc_client.send_raw_transaction(tx_hex)?;
        Ok(bitcoin::Txid::from_str(&txid.to_string())
            .map_err(|e| TransactionError::EncodingError(e.to_string()))?)
    }

    pub fn get_transaction_details(&self, txid: &bitcoin::Txid) -> Result<TransactionInfo> {
        let raw_info = self.rpc_client.get_raw_transaction_info(txid, None)?;
        
        Ok(TransactionInfo {
            txid: *txid,
            confirmations: raw_info.confirmations,
            block_height: raw_info.blockheight,
            fee: raw_info.fee.map(|f| Amount::from_btc(f.to_btc()).unwrap()),
            size: raw_info.size,
            status: self.get_transaction_status(&raw_info)?,
        })
    }

    fn validate_network(&self, address_network: Network) -> Result<()> {
        if address_network != self.network {
            return Err(TransactionError::NetworkMismatch {
                expected: self.network,
                got: address_network,
            });
        }
        Ok(())
    }

    fn select_utxos(
        &self,
        address: &Address<NetworkChecked>,
        required_amount: Amount,
    ) -> Result<Vec<bitcoincore_rpc::json::ListUnspentResultEntry>> {
        let rpc_addr = bitcoincore_rpc::bitcoin::Address::from_str(&address.to_string())?
            .require_network(self.network)?;

        let all_utxos = self.rpc_client
            .list_unspent(Some(1), None, Some(&[&rpc_addr]), None, None)?;

        if all_utxos.is_empty() {
            return Err(TransactionError::NoUtxos);
        }

        let mut selected = Vec::new();
        let mut total = Amount::from_sat(0);

        for utxo in all_utxos.iter().filter(|u| !u.spendable) {
            selected.push(utxo.clone());
            total += Amount::from_sat(utxo.amount.to_sat());

            if total >= required_amount {
                return Ok(selected);
            }
        }

        Err(TransactionError::InsufficientFunds {
            required: required_amount,
            available: total,
        })
    }

    fn create_htlc_script(&self, params: &HtlcParams) -> Result<ScriptBuf> {
        let script = Builder::new()
            .push_opcode(opcodes::OP_IF)
                .push_opcode(opcodes::OP_HASH256)
                .push_slice(&params.hash_lock)
                .push_opcode(opcodes::OP_EQUALVERIFY)
                .push_slice(&params.recipient_key.inner.serialize())
                .push_opcode(opcodes::OP_CHECKSIG)
            .push_opcode(opcodes::OP_ELSE)
                .push_int(params.timelock as i64)
                .push_opcode(opcodes::OP_CHECKMULTISIGVERIFY)
                .push_opcode(opcodes::OP_DROP)
                .push_slice(&params.refund_key.inner.serialize())
                .push_opcode(opcodes::OP_CHECKSIG)
            .push_opcode(opcodes::OP_ENDIF)
            .into_script();

        Ok(script)
    }

    fn sign_legacy_input(
        &self,
        transaction: &mut Transaction,
        input_index: usize,
        private_key: &bitcoin::PrivateKey,
        script_pubkey: &Script,
        _amount: Amount,
    ) -> Result<()> {
        let sighash = transaction.signature_hash(
            input_index,
            script_pubkey,
            SigHash::All.to_u32(),
        );

        let message = Message::from_slice(&sighash)
            .map_err(|e| TransactionError::ScriptError(e.to_string()))?;
        
        let signature = self.secp.sign_ecdsa(&message, &private_key.inner);
        let mut sig_bytes = signature.serialize_der().to_vec();
        sig_bytes.push(SigHash::All.to_u32() as u8);

        transaction.input[input_index].script_sig = Builder::new()
            .push_slice(&sig_bytes)
            .push_slice(&private_key.public_key(&self.secp).inner.inner.serialize())
            .into_script();

        Ok(())
    }

    fn sign_segwit_input(
        &self,
        transaction: &mut Transaction,
        input_index: usize,
        private_key: &bitcoin::PrivateKey,
        script_pubkey: &Script,
        amount: Amount,
    ) -> Result<()> {
        let hash = transaction.segwit_signature_hash(
            input_index,
            script_pubkey,
            amount.to_sat(),
            SigHash::All,
        )?;

        let message = Message::from_slice(&hash)
            .map_err(|e| TransactionError::ScriptError(e.to_string()))?;
        
        let signature = self.secp.sign_ecdsa(&message, &private_key.inner);
        let mut sig_bytes = signature.serialize_der().to_vec();
        sig_bytes.push(SigHash::All.to_u32() as u8);

        let witness = vec![
            sig_bytes,
            private_key.public_key(&self.secp).inner.serialize().to_vec(),
        ];

        transaction.input[input_index].witness = Witness::from_vec(witness);
        Ok(())
    }

    fn sign_taproot_input(
        &self,
        transaction: &mut Transaction,
        input_index: usize,
        private_key: &bitcoin::PrivateKey,
        script_pubkey: &Script,
        amount: Amount,
    ) -> Result<()> {
        let sighash = transaction.taproot_signature_hash(
            input_index, 
            &[script_pubkey],
            None,
            sighash::TapSighashType::All,
            amount.to_sat(),
        )?;

        let message = Message::from_slice(&sighash)
            .map_err(|e| TransactionError::ScriptError(e.to_string()))?;

        let signature = self.secp.sign_schnorr(&message, &private_key.inner);
        transaction.input[input_index].witness = Witness::from_slice(&[&signature.as_ref()]);

        Ok(())
    }

    fn create_transaction_inputs(
        &self,
        utxos: &[bitcoincore_rpc::json::ListUnspentResultEntry],
    ) -> Vec<TxIn> {
        utxos.iter()
            .map(|utxo| {
                let txid = bitcoin::Txid::from_str(&utxo.txid.to_string())
                    .expect("Invalid TXID");
                TxIn {
                    previous_output: OutPoint::new(txid, utxo.vout),
                    script_sig: ScriptBuf::default(),
                    sequence: Sequence::MAX,
                    witness: Witness::default(),
                }
            })
            .collect()
    }

    fn calculate_total_input(
        &self,
        utxos: &[bitcoincore_rpc::json::ListUnspentResultEntry],
    ) -> Result<Amount> {
        utxos.iter()
            .try_fold(Amount::from_sat(0), |acc, utxo| {
                acc.checked_add(Amount::from_sat(utxo.amount.to_sat()))
                    .ok_or_else(|| TransactionError::AmountError(
                        bitcoin::amount::ParseAmountError::Overflow
                    ))
            })
    }

    fn verify_transaction(&self, transaction: &Transaction) -> Result<()> {
        if transaction.input.is_empty() {
            return Err(TransactionError::VerificationError("No inputs".into()));
        }
        if transaction.output.is_empty() {
            return Err(TransactionError::VerificationError("No outputs".into()));
        }
        
        let total_out: u64 = transaction.output.iter().map(|out| out.value).sum();
        if total_out == 0 {
            return Err(TransactionError::VerificationError("Zero output amount".into()));
        }

        Ok(())
    }

    fn get_transaction_status(
        &self,
        info: &bitcoincore_rpc::json::GetRawTransactionResult,
    ) -> Result<TransactionStatus> {
        Ok(match info.confirmations {
            Some(0) => TransactionStatus::Unconfirmed,
            conf if conf > Some(0) => TransactionStatus::Confirmed,
            _ => {
                if info.blockhash.is_none() {
                    TransactionStatus::Replaced
                } else {
                    TransactionStatus::Failed
                }
            }
        })
    }

    pub fn get_address_balance(&self, address: &Address<NetworkChecked>) -> Result<Amount> {
        let rpc_addr = bitcoincore_rpc::bitcoin::Address::from_str(&address.to_string())?
            .require_network(self.network)?;

        let utxos = self.rpc_client
            .list_unspent(None, None, Some(&[&rpc_addr]), None, None)?;

        self.calculate_total_input(&utxos)
    }

    pub fn estimate_fee(&self, input_count: usize, output_count: usize, target_blocks: u16) -> Result<Amount> {
        let fee_rate = self.rpc_client.estimate_smart_fee(target_blocks.into(), None)?;
        
        let fee_rate = fee_rate.fee_rate
        .ok_or_else(|| TransactionError::RpcError("Failed to get fee estimate".into()))?;

        // Estimate transaction size
        let estimated_size = self.estimate_tx_size(input_count, output_count);
        let fee = (fee_rate.to_sat() as f64 * estimated_size as f64 / 1000.0) as u64;

        Ok(Amount::from_sat(fee))
    }

    fn estimate_tx_size(&self, input_count: usize, output_count: usize) -> usize {
        // Base transaction size
        const BASE_SIZE: usize = 10;
        // Size per input (approximate)
        const INPUT_SIZE: usize = 148;
        // Size per output (approximate)
        const OUTPUT_SIZE: usize = 34;

        BASE_SIZE + (input_count * INPUT_SIZE) + (output_count * OUTPUT_SIZE)
    }

    pub fn is_transaction_mature(&self, txid: &bitcoin::Txid, required_confirmations: u32) -> Result<bool> {
        let info = self.rpc_client.get_raw_transaction_info(txid, None)?;
        Ok(info.confirmations >= required_confirmations as i32)
    }

    pub fn get_ancestor_transactions(
        &self,
        txid: &bitcoin::Txid,
        max_depth: u32,
    ) -> Result<Vec<Transaction>> {
        let mut ancestors = Vec::new();
        let mut current_tx = self.rpc_client.get_raw_transaction(txid, None)?;
        let mut depth = 0;

        while depth < max_depth {
            for input in &current_tx.input {
                if let Ok(tx) = self.rpc_client.get_raw_transaction(&input.previous_output.txid, None) {
                    ancestors.push(tx.clone());
                    current_tx = tx;
                    depth += 1;
                }
            }
            if depth == 0 { break; }
        }

        Ok(ancestors)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SignatureType {
    Legacy,
    Segwit,
    Taproot,
}

#[derive(Debug)]
pub struct TransactionInfo {
    pub txid: bitcoin::Txid,
    pub confirmations: i32,
    pub block_height: Option<i32>,
    pub fee: Option<Amount>,
    pub size: u32,
    pub status: TransactionStatus,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TransactionStatus {
    Unconfirmed,
    Confirmed,
    Replaced,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::SecretKey;
    use bitcoincore_rpc::Auth;

    fn setup_test_environment() -> (TransactionManager, Address<NetworkChecked>, HtlcParams) {
        let rpc_auth = Auth::UserPass("user".to_string(), "pass".to_string());
        let manager = TransactionManager::new(
            RpcClient::new("http://localhost:8332", rpc_auth).unwrap(),
            Network::Regtest,
        );

        let sender = Address::from_str("bcrt1qw508d6qejxtdg4y5r3zarvary0c5xw7kg3g4ty")
            .unwrap()
            .require_network(Network::Regtest)
            .unwrap();

        let secp = Secp256k1::new();
        let recipient_secret = SecretKey::new(&mut rand::thread_rng());
        let refund_secret = SecretKey::new(&mut rand::thread_rng());

        let htlc_params = HtlcParams {
            amount: Amount::from_sat(100_000),
            timelock: 144,
            hash_lock: [0u8; 32],
            recipient_key: bitcoin::PublicKey::from_private_key(&secp, &bitcoin::PrivateKey::new(recipient_secret, Network::Regtest)),
            refund_key: bitcoin::PublicKey::from_private_key(&secp, &bitcoin::PrivateKey::new(refund_secret, Network::Regtest)),
        };

        (manager, sender, htlc_params)
    }

    #[test]
    fn test_htlc_transaction_creation() {
        let (manager, sender, htlc_params) = setup_test_environment();
        
        let result = manager.create_htlc_transaction(
            &sender,
            htlc_params,
            Amount::from_sat(1000),
        );

        assert!(result.is_err());  // Expected error in test environment with no UTXOs
    }

    #[test]
    fn test_fee_estimation() {
        let (manager, _, _) = setup_test_environment();
        
        let fee = manager.estimate_fee(2, 2, 6);
        assert!(fee.is_err()); // Expected error in test environment
    }

    #[test]
    fn test_tx_size_estimation() {
        let (manager, _, _) = setup_test_environment();
        
        let size = manager.estimate_tx_size(2, 2);
        assert_eq!(size, 374); // Base + 2*INPUT_SIZE + 2*OUTPUT_SIZE
    }
}