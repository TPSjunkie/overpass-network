// src/bitcoin/client.rs

use bitcoincore_rpc::{Auth, Client as RpcClient, RpcApi};
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::{Transaction, Txid};
use std::time::Duration;
use std::thread::sleep;
use log::{info, error};

/// Handles RPC client setup and network communication with Bitcoin nodes.
pub struct BitcoinRpcClient {
    rpc_client: RpcClient,
}

impl BitcoinRpcClient {
    /// Initializes a new RPC client.
    pub fn new(rpc_url: &str, rpc_user: &str, rpc_password: &str) -> Result<Self, bitcoincore_rpc::Error> {
        let auth = Auth::UserPass(rpc_user.to_string(), rpc_password.to_string());
        let rpc_client = RpcClient::new(rpc_url.to_string(), auth)?;

        Ok(BitcoinRpcClient { rpc_client })
    }

    /// Retrieves blockchain information.
    pub fn get_blockchain_info(&self) -> Result<bitcoincore_rpc::jsonrpc::serde_json::Value, bitcoincore_rpc::Error> {
        self.rpc_client.call("getblockchaininfo", &[])
    }

    /// Broadcasts a signed transaction to the Bitcoin network.
    pub fn broadcast_transaction(&self, transaction: &Transaction) -> Result<Txid, bitcoincore_rpc::Error> {
        let tx_hex = bitcoin::consensus::encode::serialize_hex(transaction);
        let txid = self.rpc_client.send_raw_transaction(&transaction)?;

        info!("Transaction broadcasted with txid: {}", txid);
        Ok(txid)
    }

    /// Monitors transaction confirmations until a specified number is reached.
    pub fn wait_for_confirmation(&self, txid: &Txid, confirmations_required: u32) -> Result<(), bitcoincore_rpc::Error> {
        loop {
            let tx_info = self.rpc_client.get_transaction(txid, Some(true))?;

            if let Some(confirmations) = tx_info.info.confirmations {
                if confirmations >= confirmations_required as i64 {
                    info!("Transaction {} has {} confirmations.", txid, confirmations);
                    break;
                } else {
                    info!("Transaction {} has {} confirmations. Waiting for {} confirmations...", txid, confirmations, confirmations_required);
                }
            } else {
                info!("Transaction {} is not yet confirmed. Waiting...", txid);
            }

            sleep(Duration::from_secs(30)); // Wait before checking again
        }

        Ok(())
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

    // Create and sign a transaction
    let txid = client.create_and_sign_transaction(&address, &amount)?;
    println!("Transaction created with Txid: {}", txid);

    // Broadcast the transaction to the network
    let txid = client.broadcast_transaction(&txid)?;
    println!("Transaction broadcasted with Txid: {}", txid);    

    Ok(())
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

    // Create a new transaction and sign it
    let txid = client.create_and_sign_transaction("bc1q5p5v8p6z9z5u0a2l3q8u9y0v5t4r3e2w1t0r9p8e7q6r5t4e3w2n1u0z9v0", 100_000_000)?;
    println!("Transaction created with Txid: {}", txid);    

    // Broadcast the transaction to the network
    let txid = client.broadcast_transaction(&txid)?;
    println!("Transaction broadcasted with Txid: {}", txid);

    // Wait for the transaction to be confirmed
    client.wait_for_confirmation(&txid, 6)?;
    // Retrieve the transaction details
    let transaction_details = client.get_transaction_details(&txid)?;    
    println!("Transaction details: {:?}", transaction_details); 
    

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_get_blockchain_info() {
        let rpc_url = env::var("RPC_URL").unwrap_or("http://localhost:18332".to_string());
        let rpc_user = env::var("RPC_USER").expect("RPC_USER not set");
        let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD not set");

        let client = BitcoinRpcClient::new(&rpc_url, &rpc_user, &rpc_password)
            .expect("Failed to create Bitcoin RPC client");

        let info = client.get_blockchain_info().expect("Failed to get blockchain info");
        assert!(info.is_object());
    }

    // Additional tests for other methods...
}
