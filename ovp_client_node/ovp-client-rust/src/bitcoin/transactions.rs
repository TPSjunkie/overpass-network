// src/bitcoin/transactions.rs

use bitcoin::absolute::LockTime;
use bitcoin::address::NetworkChecked;
use bitcoin::blockdata::script::{Builder, ScriptBuf};
use bitcoin::blockdata::transaction::{OutPoint, Transaction, TxIn, TxOut};
use bitcoin::consensus::encode;
use bitcoin::opcodes::all as opcodes;
use bitcoin::secp256k1::{All, Secp256k1};
use bitcoin::Network;
use bitcoin::{Address, Amount};
use bitcoincore_rpc::{self, Client as RpcClient, RpcApi};
use std::str::FromStr;

type Error = Box<dyn std::error::Error>;

/// Manages transaction creation and signing with HTLC support.
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

    /// Constructs a new Bitcoin transaction with HTLC support.
    pub fn create_htlc_transaction(
        &self,
        sender_address: &Address<NetworkChecked>,
        _contract_address: &Address<NetworkChecked>,
        amount: Amount,
        timelock: u32,
        hash_lock: [u8; 32],
        fee_rate: Amount,
    ) -> Result<Transaction, Error> {
        // Convert address to script for RPC call
        let sender_script = sender_address.script_pubkey();
        let rpc_addr =
            bitcoincore_rpc::bitcoin::Address::from_script(&sender_script, self.network)?;

        let utxos = self
            .rpc_client
            .list_unspent(Some(1), None, Some(&[&rpc_addr]), None, None)?;

        if utxos.is_empty() {
            return Err("No UTXOs found".into());
        }
        let mut selected_utxos = Vec::new();
        let mut total_input_value = Amount::from_sat(0);

        for utxo in utxos {
            selected_utxos.push(utxo.clone());
            total_input_value += Amount::from_sat(utxo.amount.to_sat());

            if total_input_value >= amount + fee_rate {
                break;
            }
        }

        if total_input_value < amount + fee_rate {
            return Err("Insufficient funds".into());
        }

        let tx_ins: Vec<TxIn> = selected_utxos
            .iter()
            .map(|utxo| {
                let txid =
                    bitcoin::Txid::from_str(&utxo.txid.to_string()).expect("Failed to parse txid");
                TxIn {
                    previous_output: OutPoint::new(txid, utxo.vout),
                    script_sig: ScriptBuf::default(),
                    sequence: bitcoin::Sequence::MAX,
                    witness: bitcoin::Witness::default(),
                }
            })
            .collect();

        let change = total_input_value - amount - fee_rate;

        // Create HTLC output script
        let htlc_script = Builder::new()
            .push_opcode(opcodes::OP_HASH256)
            .push_slice(&hash_lock)
            .push_opcode(opcodes::OP_EQUALVERIFY)
            .push_int(timelock as i64)
            .push_opcode(opcodes::OP_CHECKMULTISIGVERIFY)
            .push_opcode(opcodes::OP_DROP)
            .into_script();
        let _htlc_script_pubkey = ScriptBuf::from_bytes(htlc_script.clone().into_bytes());
        let mut tx_outs = vec![TxOut {
            value: amount.to_sat(),
            script_pubkey: htlc_script,
        }];

        if change.to_sat() > 0 {
            tx_outs.push(TxOut {
                value: change.to_sat(),
                script_pubkey: sender_address.script_pubkey(),
            });
        }

        let transaction = Transaction {
            version: 2,
            lock_time: LockTime::from_height(timelock).unwrap(),
            input: tx_ins,
            output: tx_outs,
        };

        Ok(transaction)
    }
    /// Signs a Bitcoin transaction with the provided private key.
    pub fn sign_transaction(
        &self,
        transaction: &mut Transaction,
        private_key: &bitcoin::PrivateKey,
        utxos: &[bitcoincore_rpc::json::ListUnspentResultEntry],
    ) -> Result<(), Error> {
        for (input_index, utxo) in utxos.iter().enumerate() {
            let script_pubkey = ScriptBuf::from_bytes(utxo.script_pub_key.clone().into_bytes());
            let sighash_cache = bitcoin::sighash::SighashCache::new(&*transaction);

            // Convert u64 to u32 safely for amount
            let amount_u32: u32 = utxo
                .amount
                .to_sat()
                .try_into()
                .map_err(|_| "Amount too large for u32")?;

            let sighash =
                sighash_cache.legacy_signature_hash(input_index, &script_pubkey, amount_u32)?;

            let message =
                bitcoin::secp256k1::Message::from_slice(&sighash[..]).expect("32 bytes");
            let signature = self.secp.sign_ecdsa(&message, &private_key.inner);

            let mut sig_der = signature.serialize_der().to_vec();
            sig_der.push(bitcoin::sighash::EcdsaSighashType::All.to_u32() as u8);
            transaction.input[input_index].script_sig = ScriptBuf::from_bytes(sig_der);
        }
        Ok(())
    }

    /// Broadcasts a signed transaction to the Bitcoin network.
    pub fn broadcast_transaction(&self, transaction: &Transaction) -> Result<bitcoin::Txid, Error> {
        let tx_hex = encode::serialize_hex(transaction);
        let rpc_txid = self.rpc_client.send_raw_transaction(tx_hex)?;

        // Convert RPC txid to bitcoin::Txid
        let txid = bitcoin::Txid::from_str(&rpc_txid.to_string())?;
        Ok(txid)
    }
    /// Retrieves transaction details from the Bitcoin network.
    pub fn get_transaction_details(
        &self,
        txid: &bitcoin::Txid,
    ) -> Result<bitcoincore_rpc::jsonrpc::serde_json::Value, Error> {
        Ok(serde_json::to_value(
            self.rpc_client.get_raw_transaction_info(txid, None)?,
        )?)
    }

    /// Retrieves the balance of a Bitcoin address.
    pub fn get_address_balance(&self, address: &Address<NetworkChecked>) -> Result<Amount, Error> {
        let rpc_addr =
            bitcoincore_rpc::bitcoin::Address::from_str(&address.to_string())?.assume_checked();
        let utxos = self
            .rpc_client
            .list_unspent(None, None, Some(&[&rpc_addr]), None, None)?;

        let balance =
            utxos
                .iter()
                .try_fold(Amount::from_sat(0), |acc, utxo| -> Result<Amount, Error> {
                    Ok(acc + Amount::from_sat(utxo.amount.to_sat()))
                })?;

        Ok(balance)
    }

    /// Retrieves the UTXOs of a Bitcoin address.
    pub fn get_address_utxos(
        &self,
        address: &Address<NetworkChecked>,
    ) -> Result<Vec<bitcoincore_rpc::json::ListUnspentResultEntry>, Error> {
        let rpc_addr =
            bitcoincore_rpc::bitcoin::Address::from_str(&address.to_string())?.assume_checked();
        let utxos = self
            .rpc_client
            .list_unspent(None, None, Some(&[&rpc_addr]), None, None)?;

        Ok(utxos)
    }

    /// Verifies if a transaction has been confirmed.
    pub fn is_transaction_confirmed(&self, txid: &bitcoin::Txid) -> Result<bool, Error> {
        let rpc_txid = bitcoincore_rpc::bitcoin::Txid::from_str(&txid.to_string())?;
        let tx_info = self.rpc_client.get_transaction(&rpc_txid, None)?;
        Ok(tx_info.info.confirmations > 0)
    }

    /// Retrieves the current network fee rate.
    pub fn get_network_fee_rate(&self) -> Result<Amount, Error> {
        let fee_rate = self.rpc_client.estimate_smart_fee(6, None)?;
        if let Some(rate) = fee_rate.fee_rate {
            Ok(Amount::from_btc(rate.to_btc())?)
        } else {
            Err("No fee rate available".into())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bitcoincore_rpc::Auth;

    #[test]
    fn test_htlc_creation() {
        // Create test addresses with proper network checks
        let sender = Address::from_str("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4")
            .expect("Invalid address")
            .require_network(Network::Bitcoin)
            .expect("Invalid network");

        let contract = Address::from_str("bc1qw508d6qejxtdg4y5r3zarvaryvg6kdaj")
            .expect("Invalid address")
            .require_network(Network::Bitcoin)
            .expect("Invalid network");
        let rpc_auth = Auth::UserPass("user".to_string(), "pass".to_string());

        let manager = TransactionManager::new(
            RpcClient::new("http://localhost:8332", rpc_auth).unwrap(),
            Network::Bitcoin,
        );

        let amount = Amount::from_sat(1_000_000);
        let fee_rate = Amount::from_sat(1000);
        let timelock = 144; // 1 day in blocks
        let hash_lock = [0u8; 32];

        let result = manager
            .create_htlc_transaction(&sender, &contract, amount, timelock, hash_lock, fee_rate);

        assert!(result.is_ok());
    }
}
