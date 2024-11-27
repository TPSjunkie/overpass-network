// ./bitcoin/client.rs

use crate::bitcoin::transactions::BridgeConfig;
use rand::RngCore;
use std::collections::HashMap;
use std::sync::{RwLock, Arc};
use bitcoin::{
    hashes::{sha256d, Hash},
    secp256k1::{PublicKey, Secp256k1, SecretKey},
    Amount, Network, Transaction, TxOut, Txid, OutPoint,
    absolute::LockTime,
};
use bitcoincore_rpc::{Auth, Client as RpcClient, RpcApi};
use rand::rngs::OsRng;
use plonky2::plonk::proof::Proof;

use crate::{
    bitcoin::{
        bitcoin_types::{BitcoinLockState, HTLCParameters, OpReturnMetadata, StealthAddress},
        scripts::ScriptManager,
    }, 
    core::zkps::zkp,
};

const MIN_SECURITY_BITS: u32 = 128;

#[derive(Debug)]
pub struct BitcoinClient {
    rpc: RpcClient,
    script_manager: ScriptManager,
    network: Network,
    state_cache: Arc<RwLock<HashMap<Txid, BitcoinLockState>>>,
    proof_system: zkp::ProofSystem,
    bridge_config: BridgeConfig,
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
        let proof_system = zkp::ProofSystem::new(MIN_SECURITY_BITS);
        
        let bridge_config = BridgeConfig {
            min_confirmation_depth: 6,
            max_timelock_duration: 144,
            min_value_sat: 546,
            security_level: MIN_SECURITY_BITS,
            network,
        };
        
        Ok(Self {
            rpc,
            script_manager,
            network,
            state_cache: Arc::new(RwLock::new(HashMap::new())),
            proof_system,
            bridge_config,
        })
    }

    pub async fn create_cross_chain_htlc(
        &self,
        amount: Amount,
        recipient_pubkey: &PublicKey,
        stealth_address: &StealthAddress,
        timelock: u32,
    ) -> Result<(Transaction, Proof<u32>), Box<dyn std::error::Error>> {
        // Generate hash lock with additional entropy
        let mut hash_lock = [0u8; 32];
        OsRng.fill_bytes(&mut hash_lock);
        
        // Create HTLC parameters
        let htlc_params = HTLCParameters::new(
            amount.to_sat(),
            recipient_pubkey.serialize()[..20].try_into()?,
            hash_lock,
            self.rpc.get_block_count()? as u32 + timelock,
            Some(stealth_address.clone()),
        );

        // Generate cross-chain proof
        let htlc_proof = self.proof_system.generate_cross_chain_proof(
            amount.to_sat(),
            recipient_pubkey,
            &hash_lock,
            timelock,
            stealth_address,
        )?;

        // Create HTLC and OP_RETURN scripts
        let (htlc_script, op_return_script) = self.script_manager.create_cross_chain_script(
            &htlc_params,
            stealth_address,
            &OpReturnMetadata::new(hash_lock, Some(stealth_address.clone()), 0x01),
        )?;

        // Create transaction outputs
        let mut outputs = vec![
            TxOut {
                value: amount.to_sat(),
                script_pubkey: htlc_script,
            },
            TxOut {
                value: 0,
                script_pubkey: op_return_script,
            },
        ];

        // Add change output if needed
        if let Some(change_amount) = self.calculate_change(amount)? {
            outputs.push(TxOut {
                value: change_amount.to_sat(),
                script_pubkey: self.get_change_address()?.script_pubkey(),
            });
        }

        // Create and sign transaction
        let tx = Transaction {
            version: 2,
            lock_time: LockTime::from_height(timelock)?,
            input: self.select_utxos(amount)?,
            output: outputs,
        };

        let signed_tx = self.sign_transaction(&tx)?;

        // Cache state with proof
        let state = BitcoinLockState::new(
            amount.to_sat(),
            sha256d::Hash::hash(&htlc_script.as_bytes()).to_byte_array(),
            self.rpc.get_block_count()? as u64,
            recipient_pubkey.serialize()[..20].try_into()?,
            timelock,
            Some(htlc_params),
            Some(htlc_proof.clone()),
        )?;

        let mut cache = self.state_cache.write().unwrap();
        cache.insert(signed_tx.txid(), state);

        Ok((signed_tx, htlc_proof))
    }
    pub async fn claim_cross_chain_htlc(
        &self,
        txid: &Txid,
        preimage: &[u8],
        recipient_key: &SecretKey,
        stealth_key: &SecretKey,
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

        // Create stealth payment script
        let recipient_pubkey = PublicKey::from_secret_key(&Secp256k1::new(), recipient_key);
        let stealth_script = self.script_manager.create_stealth_payment_script(
            &htlc_params.stealth_address.as_ref().ok_or("No stealth address")?,
            stealth_key,
        )?;

        // Calculate fee
        let fee = self.estimate_fee(1, 1)?;

        // Create claim transaction
        let claim_tx = self.script_manager.create_claim_transaction(
            OutPoint::new(*txid, 0),
            preimage.try_into()?,
            stealth_key,
            Amount::from_sat(htlc_params.amount),
            fee,
        )?;

        // Sign with stealth key
        self.sign_stealth_transaction(&claim_tx, stealth_key)?;

        Ok(claim_tx)
    }

    pub async fn verify_cross_chain_proof(
        &self,
        proof: &Proof,
        htlc_params: &HTLCParameters,
        merkle_root: &[u8; 32],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Verify security parameters
        if self.proof_system.security_bits() < MIN_SECURITY_BITS {
            return Err("Insufficient security bits".into());
        }

        // Verify proof
        self.proof_system.verify_cross_chain_proof(
            proof,
            htlc_params,
            merkle_root,
        )?;

        // Verify merkle inclusion
        if let Some(merkle_proof) = &htlc_params.merkle_proof {
            self.script_manager.verify_merkle_proof(
                &htlc_params.hash_lock,
                merkle_root,
                merkle_proof,
            )?;
        }

        Ok(true)
    }

    pub async fn scan_for_stealth_payments(
        &self,
        stealth_address: &StealthAddress,
        scan_key: &SecretKey,
    ) -> Result<Vec<TxOut>, Box<dyn std::error::Error>> {
        let mut found_outputs = Vec::new();
        let current_height = self.rpc.get_block_count()?;
        let start_height = current_height.saturating_sub(100);

        for height in start_height..=current_height {
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

    // ... existing helper methods remain unchanged ...

    pub fn configure_bridge(&mut self, config: BridgeConfig) -> Result<(), Box<dyn std::error::Error>> {
        if config.security_level < MIN_SECURITY_BITS.try_into().unwrap() {
            return Err("Insufficient security level".into());
        }
        self.bridge_config = config;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_chain_htlc() {
        let client = BitcoinClient::new(
            "http://localhost:8332",
            "user",
            "pass",
            Network::Regtest,
        ).await.unwrap();

        // Generate keys
        let recipient_key = SecretKey::new(&mut OsRng);
        let recipient_pubkey = PublicKey::from_secret_key(&Secp256k1::new(), &recipient_key);

        // Generate stealth address
        let scan_key = SecretKey::new(&mut OsRng);
        let spend_key = SecretKey::new(&mut OsRng);
        let stealth_address = StealthAddress::new(&scan_key, &spend_key, &Secp256k1::new()).unwrap();

        // Create HTLC
        let (tx, proof) = client.create_cross_chain_htlc(
            Amount::from_sat(100_000),
            &recipient_pubkey,
            &stealth_address,
            144,
        ).await.unwrap();

        assert!(tx.output.len() >= 2);
        assert!(proof.verify().unwrap());
    }
}