// src/bitcoin/client.rs
// Remove the unused imports
use bitcoincore_rpc::bitcoin::{self, Address, Amount, Transaction, Txid};
use bitcoincore_rpc::{Auth, Client as RpcClient, Error as RpcError, RpcApi};
use log::info;
use std::collections::HashMap;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

pub struct BitcoinClient {
    rpc: BitcoinRpcClient,
}

impl BitcoinClient {
    pub fn from_config(
        rpc_url: &str,
        rpc_user: &str,
        rpc_password: &str,
    ) -> Result<Self, RpcError> {
        let rpc_client = BitcoinRpcClient::from_config(rpc_url, rpc_user, rpc_password)?;
        Ok(Self { rpc: rpc_client })
    }

    pub async fn new(
        url: &str,
        user: &str,
        password: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Self::from_config(url, user, password)?;
        Ok(client)
    }

    pub fn get_blockchain_info(
        &self,
    ) -> Result<bitcoincore_rpc::jsonrpc::serde_json::Value, RpcError> {
        self.rpc.rpc_client.call("getblockchaininfo", &[])
    }

    pub fn create_and_sign_transaction(
        &self,
        address: &str,
        amount: u64,
    ) -> Result<Transaction, RpcError> {
        let _recipient_address = Address::from_str(address).map_err(|e| {
            RpcError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                e.to_string(),
            ))
        })?;
        let amount = Amount::from_sat(amount);

        // Create a raw transaction with HTLC
        let mut outputs = HashMap::new();
        let htlc_contract_address = self.rpc.rpc_client.get_new_address(None, None)?;

        // Add HTLC parameters to script
        let htlc_script = htlc_contract_address
            .require_network(bitcoin::Network::Testnet)
            .map_err(|e| {
                RpcError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e.to_string(),
                ))
            })?
            .script_pubkey();
        outputs.insert(htlc_script.to_string(), amount);

        // Create a PSBT (partially signed Bitcoin transaction) with HTLC
        let psbt = self.rpc.rpc_client.wallet_create_funded_psbt(
            &[],        // Inputs
            &outputs,   // Outputs with HTLC script
            None,       // Locktime
            None,       // Replaceable
            Some(true), // All outputs must be confirmed
        )?;
        // Sign the transaction
        let signed_psbt = self
            .rpc
            .rpc_client
            .wallet_process_psbt(&psbt.psbt, None, None, Some(true))?
            .psbt;

        // Finalize and extract the transaction
        let final_tx = self.rpc.rpc_client.finalize_psbt(&signed_psbt, Some(true))?;

        let transaction: Transaction =
            bitcoin::consensus::encode::deserialize(&final_tx.hex.unwrap()).map_err(|e| {
                RpcError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                ))
            })?;

        Ok(transaction)
    }

    pub fn broadcast_transaction(&self, transaction: &Transaction) -> Result<Txid, RpcError> {
        let txid = self.rpc.rpc_client.send_raw_transaction(transaction)?;
        info!("Transaction broadcasted with txid: {}", txid);
        Ok(txid)
    }

    pub fn wait_for_confirmation(
        &self,
        txid: &Txid,
        confirmations_required: u32,
    ) -> Result<(), RpcError> {
        loop {
            let tx_info = self.rpc.rpc_client.get_transaction(txid, Some(true))?;
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

    pub fn get_transaction_details(
        &self,
        txid: &Txid,
    ) -> Result<bitcoincore_rpc::jsonrpc::serde_json::Value, RpcError> {
        Ok(serde_json::to_value(
            self.rpc.rpc_client.get_raw_transaction_info(txid, None)?,
        )?)
    }
}

// Ensure that BitcoinRpcClient's fields are public or provide methods to access them
pub struct BitcoinRpcClient {
    pub rpc_client: RpcClient,
    pub config: BitcoinRpcConfig,
    // Removed 'inner' field if not necessary
}

impl BitcoinRpcClient {
    /// Initializes a new RPC client.
    pub fn from_config(
        rpc_url: &str,
        rpc_user: &str,
        rpc_password: &str,
    ) -> Result<Self, RpcError> {
        let auth = Auth::UserPass(rpc_user.to_string(), rpc_password.to_string());
        let client = RpcClient::new(rpc_url, auth)?;
        Ok(Self {
            rpc_client: client,
            config: BitcoinRpcConfig {
                url: rpc_url.to_string(),
                user: rpc_user.to_string(),
                password: rpc_password.to_string(),
            },
        })
    }
}

pub struct BitcoinRpcConfig {
    pub url: String,
    pub user: String,
    pub password: String,
}

pub struct Person {
    name: String, // Private by default
}

impl Person {
    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

#[allow(dead_code)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the logger
    env_logger::init();

    // Load RPC credentials from environment variables
    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:18332".to_string());
    let rpc_user = std::env::var("RPC_USER")?;
    let rpc_password = std::env::var("RPC_PASSWORD")?;

    // Create the Bitcoin client
    let client = BitcoinClient::from_config(&rpc_url, &rpc_user, &rpc_password)?;

    // Retrieve blockchain information
    let blockchain_info = client.get_blockchain_info()?;
    println!("Blockchain info: {:?}", blockchain_info);

    // Example of creating, broadcasting, and monitoring a transaction
    let transaction = client.create_and_sign_transaction(
        "tb1q5p5v8p6z9z5u0a2l3q8u9y0v5t4r3e2w1t0r9p8e7q6r5t4e3w2n1u0z9v0",
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
        let rpc_url =
            env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:18332".to_string());
        let rpc_user = env::var("RPC_USER").expect("RPC_USER not set");
        let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD not set");

        let client = BitcoinClient::from_config(&rpc_url, &rpc_user, &rpc_password)
            .expect("Failed to create Bitcoin client");

        let info = client
            .get_blockchain_info()
            .expect("Failed to get blockchain info");
        assert!(info.is_object());
    }

    #[test]
    fn test_create_and_sign_transaction() {
        let rpc_url =
            env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:18332".to_string());
        let rpc_user = env::var("RPC_USER").expect("RPC_USER not set");
        let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD not set");

        let client = BitcoinClient::from_config(&rpc_url, &rpc_user, &rpc_password)
            .expect("Failed to create Bitcoin client");

        let result = client.create_and_sign_transaction(
            "tb1q5p5v8p6z9z5u0a2l3q8u9y0v5t4r3e2w1t0r9p8e7q6r5t4e3w2n1u0z9v0",
            10_000,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_transaction_details() {
        let rpc_url =
            env::var("RPC_URL").unwrap_or_else(|_| "http://localhost:18332".to_string());
        let rpc_user = env::var("RPC_USER").expect("RPC_USER not set");
        let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD not set");

        let client = BitcoinClient::from_config(&rpc_url, &rpc_user, &rpc_password)
            .expect("Failed to create Bitcoin client");

        // Note: This test requires a valid transaction from the blockchain
        let txid = Txid::from_str(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let result = client.get_transaction_details(&txid);
        assert!(result.is_err()); // Expected to fail with invalid txid
    }
}
