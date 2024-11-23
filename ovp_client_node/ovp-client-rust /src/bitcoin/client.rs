// src/bitcoin/client.rs

use bitcoincore_rpc::bitcoin::{Address, Amount, Transaction};
use bitcoincore_rpc::{Auth, Client as RpcClient, RpcApi};
use log::info;
use std::collections::HashMap;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

/// Handles RPC client setup and network communication with Bitcoin nodes.
pub struct BitcoinRpcClient {
    rpc_client: RpcClient,
}

impl BitcoinRpcClient {
    /// Initializes a new RPC client.
    pub fn new(
        rpc_url: &str,
        rpc_user: &str,
        rpc_password: &str,
    ) -> Result<Self, bitcoincore_rpc::Error> {
        let auth = Auth::UserPass(rpc_user.to_string(), rpc_password.to_string());
        let rpc_client = RpcClient::new(rpc_url, auth)?;

        Ok(BitcoinRpcClient { rpc_client })
    }

    /// Retrieves blockchain information.
    pub fn get_blockchain_info(
        &self,
    ) -> Result<bitcoincore_rpc::jsonrpc::serde_json::Value, bitcoincore_rpc::Error> {
        self.rpc_client.call("getblockchaininfo", &[])
    }

    /// Creates and signs a transaction to send Bitcoin to a specified address.
    pub fn create_and_sign_transaction(
        &self,
        address: &str,
        amount: u64,
    ) -> Result<Transaction, bitcoincore_rpc::Error> {
        let recipient_address = Address::from_str(address).map_err(|e| {
            bitcoincore_rpc::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e.to_string(),
            ))
        })?;
        let amount = Amount::from_sat(amount);

        // Create a raw transaction with HTLC
        let mut outputs = HashMap::new();
        let htlc_contract_address = self.rpc_client.get_new_address(None, None)?;

        // Add HTLC parameters to script
        let htlc_script = htlc_contract_address
            .require_network(bitcoin::Network::Testnet)
            .map_err(|e| {
                bitcoincore_rpc::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e.to_string(),
                ))
            })?
            .script_pubkey();
        outputs.insert(htlc_script.to_string(), amount);

        // Create a PSBT (partially signed Bitcoin transaction) with HTLC
        let psbt = self.rpc_client.wallet_create_funded_psbt(
            &[],        // Inputs
            &outputs,   // Outputs with HTLC script
            None,       // Locktime
            None,       // Replaceable
            Some(true), // All outputs must be confirmed
        )?;
        // Sign the transaction
        let signed_psbt = self
            .rpc_client
            .wallet_process_psbt(&psbt.psbt, None, None, Some(true))?
            .psbt;

        // Finalize and extract the transaction
        let final_tx = self.rpc_client.finalize_psbt(&signed_psbt, Some(true))?;

        let transaction: Transaction =
            bitcoin::consensus::encode::deserialize(&final_tx.hex.unwrap()).map_err(|e| {
                bitcoincore_rpc::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                ))
            })?;

        Ok(transaction)
    }
    /// Broadcasts a signed transaction to the Bitcoin network.
    pub fn broadcast_transaction(
        &self,
        transaction: &Transaction,
    ) -> Result<bitcoincore_rpc::bitcoin::Txid, bitcoincore_rpc::Error> {
        let tx_hex = bitcoin::consensus::encode::serialize_hex(transaction);
        let txid = self.rpc_client.send_raw_transaction(&*tx_hex)?;

        info!("Transaction broadcasted with txid: {}", txid);
        Ok(txid)
    }

    /// Monitors transaction confirmations until a specified number is reached.
    pub fn wait_for_confirmation(
        &self,
        txid: &bitcoincore_rpc::bitcoin::Txid,
        confirmations_required: u32,
    ) -> Result<(), bitcoincore_rpc::Error> {
        loop {
            let tx_info = self.rpc_client.get_transaction(txid, Some(true))?;
            let confirmations = tx_info.info.confirmations;
            if confirmations >= confirmations_required as i32 {
                break;
            } else {
                info!(
                    "Transaction {} has {} confirmations. Waiting for {} confirmations...",
                    txid, confirmations, confirmations_required
                );
                sleep(Duration::from_secs(30)); // Wait before checking again
            }
        }
        Ok(())
    }

    /// Retrieves detailed information about a transaction.
    pub fn get_transaction_details(
        &self,
        txid: &bitcoincore_rpc::bitcoin::Txid,
    ) -> Result<bitcoincore_rpc::jsonrpc::serde_json::Value, bitcoincore_rpc::Error> {
        Ok(serde_json::to_value(
            self.rpc_client.get_raw_transaction_info(txid, None)?,
        )?)
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();

    // Load RPC credentials from environment variables
    let rpc_url = std::env::var("RPC_URL").unwrap_or("http://localhost:18332".to_string());
    let rpc_user = std::env::var("RPC_USER")?;
    let rpc_password = std::env::var("RPC_PASSWORD")?;

    // Create the Bitcoin RPC client
    let client = BitcoinRpcClient::new(&rpc_url, &rpc_user, &rpc_password)?;

    // Retrieve blockchain information
    let blockchain_info = client.get_blockchain_info()?;
    println!("Blockchain info: {:?}", blockchain_info);

    // Example of creating, broadcasting, and monitoring a transaction
    let transaction = client.create_and_sign_transaction(
        "bc1q5p5v8p6z9z5u0a2l3q8u9y0v5t4r3e2w1t0r9p8e7q6r5t4e3w2n1u0z9v0",
        100_000_000,
    )?;
    println!("Transaction created");

    let txid = client.broadcast_transaction(&transaction)?;
    println!("Transaction broadcasted with Txid: {}", txid);

    client.wait_for_confirmation(&txid, 6)?;
    let transaction_details = client.get_transaction_details(&txid)?;
    println!("Transaction details: {:?}", transaction_details);

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::str::FromStr;

    #[test]
    fn test_get_blockchain_info() {
        let rpc_url = env::var("RPC_URL").unwrap_or("http://localhost:18332".to_string());
        let rpc_user = env::var("RPC_USER").expect("RPC_USER not set");
        let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD not set");

        let client = BitcoinRpcClient::new(&rpc_url, &rpc_user, &rpc_password)
            .expect("Failed to create Bitcoin RPC client");

        let info = client
            .get_blockchain_info()
            .expect("Failed to get blockchain info");
        assert!(info.is_object());
    }

    #[test]
    fn test_create_and_sign_transaction() {
        let rpc_url = env::var("RPC_URL").unwrap_or("http://localhost:18332".to_string());
        let rpc_user = env::var("RPC_USER").expect("RPC_USER not set");
        let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD not set");

        let client = BitcoinRpcClient::new(&rpc_url, &rpc_user, &rpc_password)
            .expect("Failed to create Bitcoin RPC client");

        let result = client.create_and_sign_transaction(
            "bc1q5p5v8p6z9z5u0a2l3q8u9y0v5t4r3e2w1t0r9p8e7q6r5t4e3w2n1u0z9v0",
            10_000,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_transaction_details() {
        let rpc_url = env::var("RPC_URL").unwrap_or("http://localhost:18332".to_string());
        let rpc_user = env::var("RPC_USER").expect("RPC_USER not set");
        let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD not set");

        let client = BitcoinRpcClient::new(&rpc_url, &rpc_user, &rpc_password)
            .expect("Failed to create Bitcoin RPC client");

        // Note: This test requires a valid transaction from the blockchain
        let txid = bitcoincore_rpc::bitcoin::Txid::from_str(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let result = client.get_transaction_details(&txid);
        assert!(result.is_err()); // Expected to fail with invalid txid
    }
}
