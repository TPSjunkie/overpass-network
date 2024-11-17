// src/bitcoin/transactions.rs

use bitcoin::consensus::encode::serialize;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::util::address::Address;
use bitcoin::util::amount::Amount;
use bitcoin::util::key::PrivateKey;
use bitcoin::blockdata::transaction::{Transaction, TxIn, TxOut};
use bitcoin::blockdata::script::Script;
use bitcoin::hash_types::Txid;
use bitcoin::Network;
use bitcoin::secp256k1::{Secp256k1, SecretKey, All};
use bitcoincore_rpc::{Client as RpcClient, RpcApi};
use std::collections::HashMap;
use log::{info, error};

use super::client::BitcoinRpcClient;

/// Manages transaction creation and signing.
pub struct TransactionManager {
    rpc_client: RpcClient,
    secp: Secp256k1<All>,
    network: Network,
}

impl TransactionManager {
    /// Creates a new TransactionManager instance.
    pub fn new(rpc_client: RpcClient, network: Network) -> Self {
        Self {
            rpc_client,
            secp: Secp256k1::new(),
            network,
        }
    }

    /// Constructs a new Bitcoin transaction.
    pub fn create_transaction(
        &self,
        from_address: &Address,
        to_address: &Address,
        amount: Amount,
        fee_rate: Amount,
    ) -> Result<Transaction, bitcoincore_rpc::Error> {
        // Step 1: List unspent outputs (UTXOs) for the from_address
        let utxos = self.rpc_client.list_unspent(
            Some(1),
            None,
            Some(vec![from_address.clone()]),
            None,
            None,
        )?;

        // Step 2: Select UTXOs to cover the amount + fee
        let mut selected_utxos = Vec::new();
        let mut total_input_value = Amount::from_sat(0);

        for utxo in utxos {
            selected_utxos.push(utxo.clone());
            total_input_value += utxo.amount;

            if total_input_value >= amount + fee_rate {
                break;
            }
        }

        if total_input_value < amount + fee_rate {
            return Err(bitcoincore_rpc::Error::UnexpectedStructure(
                "Insufficient funds".to_string(),
            ));
        }

        // Step 3: Build the transaction inputs
        let tx_ins: Vec<TxIn> = selected_utxos
            .iter()
            .map(|utxo| TxIn {
                previous_output: utxo.outpoint,
                script_sig: Script::new(),
                sequence: 0xFFFFFFFF,
                witness: Vec::new(),
            })
            .collect();

        // Step 4: Calculate the change and build outputs
        let change = total_input_value - amount - fee_rate;

        let mut tx_outs = vec![TxOut {
            value: amount.as_sat(),
            script_pubkey: to_address.script_pubkey(),
        }];

        if change.as_sat() > 0 {
            // Send change back to from_address
            tx_outs.push(TxOut {
                value: change.as_sat(),
                script_pubkey: from_address.script_pubkey(),
            });
        }

        // Step 5: Create the unsigned transaction
        let transaction = Transaction {
            version: 2,
            lock_time: 0,
            input: tx_ins,
            output: tx_outs,
        };

        Ok(transaction)
    }

    /// Signs transaction inputs.
    pub fn sign_transaction(
        &self,
        transaction: &Transaction,
        from_address: &Address,
        private_key: &PrivateKey,
    ) -> Result<Transaction, bitcoincore_rpc::Error> {
        let mut signed_transaction = transaction.clone();

        // For each input, we need to provide the corresponding scriptPubKey and amount
        let utxos = self.rpc_client.list_unspent(
            Some(1),
            None,
            Some(vec![from_address.clone()]),
            None,
            None,
        )?;

        let utxo_map: HashMap<Txid, bitcoincore_rpc::json::ListUnspentResultEntry> = utxos
            .into_iter()
            .map(|utxo| (utxo.outpoint.txid, utxo))
            .collect();

        for (i, txin) in signed_transaction.input.iter_mut().enumerate() {
            let prev_txid = txin.previous_output.txid;

            let utxo = utxo_map.get(&prev_txid).ok_or_else(|| {
                bitcoincore_rpc::Error::UnexpectedStructure(
                    "UTXO not found for signing".to_string(),
                )
            })?;

            // Create the signature hash
            let sighash = bitcoin::util::sighash::SighashCache::new(&signed_transaction)
                .signature_hash(
                    i,
                    &utxo.script_pub_key,
                    utxo.amount.as_sat(),
                    bitcoin::EcdsaSighashType::All,
                );

            let msg = bitcoin::secp256k1::Message::from_slice(&sighash[..]).map_err(|e| {
                bitcoincore_rpc::Error::UnexpectedStructure(format!(
                    "Failed to create secp256k1 message: {}",
                    e
                ))
            })?;

            let sig = self.secp.sign_ecdsa(&msg, &private_key.inner);

            let mut sig_der = sig.serialize_der().to_vec();
            sig_der.push(bitcoin::EcdsaSighashType::All as u8);

            // For P2PKH, scriptSig = <signature> <public key>
            txin.script_sig = bitcoin::blockdata::script::Builder::new()
                .push_slice(&sig_der)
                .push_slice(&private_key.public_key(&self.secp).to_bytes())
                .into_script();
        }

        Ok(signed_transaction)
    }

    /// Combines creation, signing, and broadcasting of a transaction.
    pub fn create_and_broadcast_lock_tx(
        &self,
        from_address: &Address,
        to_address: &Address,
        amount: Amount,
        fee_rate: Amount,
        private_key: &PrivateKey,
    ) -> Result<Txid, bitcoincore_rpc::Error> {
        // Step 1: Create the transaction
        let transaction = self.create_transaction(from_address, to_address, amount, fee_rate)?;

        // Step 2: Sign the transaction
        let signed_transaction = self.sign_transaction(&transaction, from_address, private_key)?;

        // Step 3: Broadcast the transaction
        let txid = self.rpc_client.send_raw_transaction(&signed_transaction)?;

        info!("Transaction broadcasted with txid: {}", txid);

        Ok(txid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::util::address::Address;
    use bitcoin::util::key::PrivateKey;
    use bitcoin::Network;
    use bitcoincore_rpc::{Auth, Client};

    #[test]
    fn test_create_transaction() {
        let rpc_client = setup_rpc_client();
        let network = Network::Testnet;
        let tx_manager = TransactionManager::new(rpc_client, network);

        let from_address = Address::from_str("from_address_here").unwrap();
        let to_address = Address::from_str("to_address_here").unwrap();

        let amount = Amount::from_btc(0.001).unwrap();
        let fee_rate = Amount::from_sat(1000);

        let tx = tx_manager
            .create_transaction(&from_address, &to_address, amount, fee_rate)
            .expect("Failed to create transaction");

        assert!(!tx.input.is_empty());
        assert!(!tx.output.is_empty());
    }

    // Additional tests...

    fn setup_rpc_client() -> RpcClient {
        let rpc_url = std::env::var("RPC_URL").unwrap_or("http://localhost:18332".to_string());
        let rpc_user = std::env::var("RPC_USER").expect("RPC_USER not set");
        let rpc_password = std::env::var("RPC_PASSWORD").expect("RPC_PASSWORD not set");
        let auth = Auth::UserPass(rpc_user, rpc_password);
        RpcClient::new(rpc_url, auth).expect("Failed to create RPC client")
    }
}

