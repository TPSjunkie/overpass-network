use bitcoin::{
    Address, Amount, Network, OutPoint, Transaction, TxIn, TxOut, Txid,
    blockdata::script::{Script, ScriptBuf},
    consensus::encode::serialize,
    hashes::{sha256d, Hash},
    secp256k1::{SecretKey, PublicKey, Secp256k1},
};
use bitcoincore_rpc::{Auth, Client as RpcClient, RpcApi};
use crate::bitcoin::{
    bitcoin_types::{HTLCParameters, StealthAddress, OpReturnMetadata, BitcoinLockState},
    scripts::ScriptManager,
};
use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};
use tokio::sync::RwLock;
use log::{info, warn};

pub struct BitcoinClient {
    rpc: RpcClient,
    script_manager: ScriptManager,
    network: Network,
    state_cache: Arc<RwLock<HashMap<Txid, BitcoinLockState>>>,
}

impl BitcoinClient {
    pub async fn new(
        url: &str,
        user: &str,
        password: &str,
        network: Network,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let auth = Auth::UserPass(user.to_string(), password.to_string());
        let rpc = RpcClient::new(url, auth)?;
        let script_manager = ScriptManager::new(network);
        
        Ok(Self {
            rpc,
            script_manager,
            network,
            state_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn create_htlc_transaction(
        &self,
        amount: u64,
        recipient_pubkey: &PublicKey,
        hash_lock: [u8; 32],
        timeout_blocks: u32,
        stealth_address: Option<StealthAddress>,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        let htlc_params = HTLCParameters::new(
            amount,
            recipient_pubkey.serialize_uncompressed()[..20].try_into()?,
            hash_lock,
            self.rpc.get_block_count()? as u32 + timeout_blocks,
            stealth_address,
        );

        let script = self.script_manager.create_htlc_script(&htlc_params, recipient_pubkey)?;
        let mut tx_builder = self.rpc.create_raw_transaction_hex(
            &[],
            &[(script.as_script().to_string(), Amount::from_sat(amount))],
            None,
            None,
        )?;

        // Add OP_RETURN output if stealth address is present
        if let Some(stealth) = &htlc_params.stealth_address {
            let metadata = OpReturnMetadata::new(
                hash_lock,
                Some(stealth.clone()),
                0x01, // Enable rebalancing
            );
            let op_return = self.script_manager.create_op_return_script(&metadata)?;
            tx_builder = self.rpc.add_output_to_transaction(&tx_builder, op_return, 0)?;
        }

        let signed_tx = self.rpc.sign_raw_transaction_with_wallet(**tx_builder, None, None)?;
        let transaction = bitcoin::consensus::encode::deserialize(&signed_tx.hex)?;

        // Cache the HTLC state
        let state = BitcoinLockState::new(
            amount,
            sha256d::Hash::hash(&script.as_bytes()).to_byte_array(),
            self.rpc.get_block_count()? as u64,
            recipient_pubkey.serialize_uncompressed()[..20].try_into()?,
            0xFFFFFFFF,
            Some(htlc_params),
            None,
        )?;

        let mut cache = self.state_cache.write().await;
        cache.insert(transaction.txid(), state);

        Ok(transaction)
    }

    pub async fn spend_htlc(
        &self,
        txid: &Txid,
        preimage: &[u8],
        recipient_key: &SecretKey,
    ) -> Result<Transaction, Box<dyn std::error::Error>> {
        let cache = self.state_cache.read().await;
        let state = cache.get(txid)
            .ok_or("HTLC state not found")?;

        let htlc_params = state.htlc_params.as_ref()
            .ok_or("No HTLC parameters found")?;

        // Verify preimage
        if !htlc_params.verify_hashlock(preimage)? {
            return Err("Invalid preimage".into());
        }

        let recipient_pubkey = PublicKey::from_secret_key(&Secp256k1::new(), recipient_key);
        let script = self.script_manager.create_htlc_script(htlc_params, &recipient_pubkey)?;

        // Create spending transaction
        let mut tx_builder = self.rpc.create_raw_transaction_hex(
            &[OutPoint { txid: *txid, vout: 0 }],
            &[(
                Address::p2pkh(&recipient_pubkey, self.network).to_string(),
                Amount::from_sat(htlc_params.amount),
            )],
            None,
            None,
        )?;

        // Sign transaction
        let signature = self.rpc.sign_raw_transaction_with_key(
            &*tx_builder,
            &[recipient_key.display_secret().to_string()],
            &[script.as_script()],
            None,
        )?;

        Ok(bitcoin::consensus::encode::deserialize(&signature.hex)?)
    }

    pub async fn scan_for_stealth_payments(
        &self,
        stealth_address: &StealthAddress,
        scan_key: &SecretKey,
    ) -> Result<Vec<TxOut>, Box<dyn std::error::Error>> {
        let mut found_outputs = Vec::new();
        let block_height = self.rpc.get_block_count()?;

        for height in (block_height - 100)..=block_height {
            let block_hash = self.rpc.get_block_hash(height)?;
            let block = self.rpc.get_block(&block_hash)?;

            for tx in block.txdata {
                let outputs = self.script_manager.scan_transaction_outputs(
                    &tx,
                    stealth_address,
                    scan_key,
                )?;
                found_outputs.extend(outputs.into_iter().map(|(_, output)| output));
            }
        }

        Ok(found_outputs)
    }

    pub async fn broadcast_transaction(&self, transaction: &Transaction) -> Result<Txid, Box<dyn std::error::Error>> {
        let txid = self.rpc.send_raw_transaction(&serialize(transaction))?;
        info!("Transaction broadcasted with txid: {}", txid);
        Ok(txid)
    }

    pub async fn wait_for_confirmation(&self, txid: &Txid, confirmations: u32) -> Result<(), Box<dyn std::error::Error>> {
        let start_time = SystemTime::now();
        let timeout = Duration::from_secs(3600); // 1 hour timeout

        while SystemTime::now().duration_since(start_time)? < timeout {
            let tx_info = self.rpc.get_raw_transaction_info(txid, None)?;
            
            if let Some(confirms) = tx_info.confirmations {
                if confirms >= (confirmations as i32).try_into().unwrap() {
                    info!("Transaction {} confirmed with {} confirmations", txid, confirms);
                    return Ok(());
                }
                info!("Waiting for {} more confirmations for tx {}", confirmations - confirms as u32, txid);
            }

            tokio::time::sleep(Duration::from_secs(30)).await;
        }

        Err("Transaction confirmation timeout".into())
    }

    pub async fn verify_transaction(&self, txid: &Txid) -> Result<bool, Box<dyn std::error::Error>> {
        let tx_info = self.rpc.get_raw_transaction_info(txid, None)?;
        let cache = self.state_cache.read().await;
        
        if let Some(state) = cache.get(txid) {
            if let Some(htlc) = &state.htlc_params {
                let current_height = self.rpc.get_block_count()? as u32;
                return Ok(htlc.verify_timelock(current_height));
            }
        }

        Ok(tx_info.confirmations.unwrap_or(0) > 0)
    }
}

