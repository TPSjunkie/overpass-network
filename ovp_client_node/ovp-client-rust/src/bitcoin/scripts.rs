// ./bitcoin/scripts.rs

use bitcoin::opcodes::all;
use crate::bitcoin::bitcoin_types::{StealthAddress, HTLCParameters, OpReturnMetadata};
use bitcoin::{
    address::{Address, NetworkChecked},
    blockdata::script::{Script, ScriptBuf, Builder},
    key::{PrivateKey, PublicKey},
    secp256k1::{All, Secp256k1, SecretKey, Message, KeyPair},
    Network, Transaction, TxOut, Amount,
    hashes::{sha256, sha256d, hash160, Hash, HashEngine},
    absolute::LockTime,
};
use std::collections::HashMap;
use thiserror::Error;

const MIN_SECURITY_BITS: usize = 128;

#[derive(Error, Debug)]
pub enum ScriptError {
    #[error("Bitcoin error: {0}")]
    BitcoinError(#[from] bitcoin::Error),
    
    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(#[from] bitcoin::secp256k1::Error),
    
    #[error("Script verification failed: {0}")]
    ScriptVerificationError(String),
    
    #[error("Stealth address error: {0}")]
    StealthAddressError(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Security error: {0}")]
    SecurityError(String),

    #[error("Cross-chain error: {0}")]
    CrossChainError(String),
}

pub type Result<T> = std::result::Result<T, ScriptError>;

pub struct ScriptManager {
    secp: Secp256k1<All>,
    network: Network,
    script_cache: HashMap<[u8; 32], ScriptBuf>,
    security_bits: usize,
}

impl ScriptManager {
    pub fn new(network: Network) -> Self {
        Self {
            secp: Secp256k1::new(),
            network,
            script_cache: HashMap::new(),
            security_bits: MIN_SECURITY_BITS,
        }
    }

    pub fn create_htlc_script(
        &self,
        htlc_params: &HTLCParameters,
        stealth_address: &StealthAddress,
    ) -> Result<ScriptBuf> {
        // Validate security parameter
        if self.security_bits < MIN_SECURITY_BITS {
            return Err(ScriptError::SecurityError(
                format!("Security parameter λ must be at least {} bits", MIN_SECURITY_BITS)
            ));
        }

        let builder = Builder::new()
            // Hash timelock branch
            .push_opcode(all::OP_IF)
                // Hash verification
                .push_opcode(all::OP_SHA256)
                .push_slice(&htlc_params.hash_lock)
                .push_opcode(all::OP_EQUALVERIFY)
                // Stealth payment verification
                .push_slice(&stealth_address.spend_pubkey)
                .push_opcode(all::OP_CHECKSIG)
                .push_slice(&stealth_address.view_tag)
                .push_opcode(all::OP_DROP)
            // Timeout refund branch
            .push_opcode(all::OP_ELSE)
                .push_int(htlc_params.timeout_height as i64)
                .push_opcode(all::OP_CHECKLOCKTIMEVERIFY)
                .push_opcode(all::OP_DROP)
                .push_slice(&htlc_params.refund_pubkey.serialize())
                .push_opcode(all::OP_CHECKSIG)
            .push_opcode(all::OP_ENDIF);

        Ok(builder.into_script())
    }

    pub fn create_stealth_payment_script(
        &self,
        stealth_address: &StealthAddress,
        ephemeral_key: &SecretKey,
    ) -> Result<ScriptBuf> {
        // Calculate shared secret
        let shared_point = stealth_address.scan_pubkey
            .mul_tweak(&self.secp, &(*ephemeral_key).into())?;
        
        // Generate deterministic nonce
        let mut hasher = sha256::HashEngine::default();
        hasher.input(&shared_point.serialize());
        hasher.input(&stealth_address.view_tag);
        let nonce = sha256::Hash::from_engine(hasher);

        // Derive payment key
        let payment_point = stealth_address.spend_pubkey
            .combine(&self.secp, &PublicKey::from_secret_key(&self.secp, &nonce.into_inner().into()))?;

        let builder = Builder::new()
            .push_slice(&payment_point.serialize())
            .push_opcode(all::OP_CHECKSIG)
            // Add view tag for scanning
            .push_slice(&stealth_address.view_tag)
            .push_opcode(all::OP_DROP);

        Ok(builder.into_script())
    }

    pub fn create_cross_chain_script(
        &self,
        htlc_params: &HTLCParameters,
        stealth_address: &StealthAddress,
        op_return_data: &OpReturnMetadata,
    ) -> Result<(ScriptBuf, ScriptBuf)> {
        // Create HTLC script
        let htlc_script = self.create_htlc_script(htlc_params, stealth_address)?;

        // Create OP_RETURN script with bridge metadata
        let op_return_script = Builder::new()
            .push_opcode(all::OP_RETURN)
            .push_slice(&op_return_data.encode_for_bitcoin()?)
            .into_script();

        Ok((htlc_script, op_return_script))
    }

    pub fn verify_cross_chain_spend(
        &self,
        htlc_script: &Script,
        stealth_script: &Script,
        preimage: &[u8],
        signature: &[u8],
        stealth_key: &SecretKey,
    ) -> Result<bool> {
        // Verify HTLC preimage
        let hash = sha256::Hash::hash(preimage);
        if hash != htlc_params.hash_lock {
            return Ok(false);
        }

        // Verify stealth signature
        let pubkey = PublicKey::from_secret_key(&self.secp, stealth_key);
        let msg = Message::from_slice(&sha256d::Hash::hash(&stealth_script[..])[..])?;
        self.secp.verify_schnorr(
            &msg,
            &signature.try_into().map_err(|_| ScriptError::SecurityError("Invalid signature".into()))?,
            &pubkey.into(),
        )?;

        Ok(true)
    }

    pub fn create_claim_transaction(
        &self,
        htlc_outpoint: bitcoin::OutPoint,
        preimage: [u8; 32],
        stealth_key: &SecretKey,
        amount: Amount,
        fee: Amount,
    ) -> Result<Transaction> {
        let pubkey = PublicKey::from_secret_key(&self.secp, stealth_key);
        
        // Create stealth payment script
        let claim_script = Builder::new()
            .push_slice(&pubkey.serialize())
            .push_opcode(all::OP_CHECKSIG)
            .into_script();

        // Create transaction
        let tx = Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: vec![bitcoin::TxIn {
                previous_output: htlc_outpoint,
                script_sig: ScriptBuf::new(),
                sequence: bitcoin::Sequence::MAX,
                witness: bitcoin::Witness::new(),
            }],
            output: vec![TxOut {
                value: amount.to_sat() - fee.to_sat(),
                script_pubkey: claim_script,
            }],
        };

        Ok(tx)
    }

    pub fn scan_for_stealth_payments(
        &self,
        tx: &Transaction,
        stealth_address: &StealthAddress,
        scan_key: &SecretKey,
    ) -> Result<Vec<(usize, TxOut)>> {
        let mut found_outputs = Vec::new();

        for (index, output) in tx.output.iter().enumerate() {
            if self.is_stealth_payment(&output.script_pubkey, stealth_address, scan_key)? {
                found_outputs.push((index, output.clone()));
            }
        }

        Ok(found_outputs)
    }

    pub fn verify_merkle_proof(
        &self,
        leaf: &[u8],
        root: &[u8],
        proof: &[[u8; 32]],
    ) -> Result<bool> {
        let mut current = sha256d::Hash::hash(leaf);

        for sibling in proof {
            let mut hasher = sha256d::Hash::engine();
            if current[..] <= sibling[..] {
                hasher.input(&current[..]);
                hasher.input(sibling);
            } else {
                hasher.input(sibling);
                hasher.input(&current[..]);
            }
            current = sha256d::Hash::from_engine(hasher);
        }

        Ok(current[..] == root[..])
    }

    fn is_stealth_payment(
        &self,
        script: &Script,
        stealth_address: &StealthAddress,
        scan_key: &SecretKey,
    ) -> Result<bool> {
        if !script.is_p2pkh() && !script.is_v0_p2wpkh() {
            return Ok(false);
        }

        let scan_pubkey = PublicKey::from_secret_key(&self.secp, scan_key);
        let shared_secret = stealth_address.scan_pubkey
            .mul_tweak(&self.secp, &(*scan_key).into())?;

        let payment_key = stealth_address.spend_pubkey
            .combine(&self.secp, &PublicKey::from_secret_key(&self.secp, &sha256::Hash::hash(&shared_secret.serialize()).into_inner().into()))?;

        let payment_hash = hash160::Hash::hash(&payment_key.serialize());
        
        Ok(script.as_bytes().windows(20).any(|window| window == payment_hash[..]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::rand::{thread_rng, RngCore};

    #[test]
    fn test_htlc_script_creation() {
        let manager = ScriptManager::new(Network::Regtest);
        
        // Generate keys
        let secret_key = SecretKey::new(&mut thread_rng());
        let public_key = PublicKey::from_secret_key(&manager.secp, &secret_key);

        // Create stealth address
        let mut view_tag = [0u8; 32];
        thread_rng().fill_bytes(&mut view_tag);
        let stealth_address = StealthAddress {
            scan_pubkey: public_key.inner,
            spend_pubkey: public_key.inner,
            view_tag,
        };

        // Create HTLC parameters
        let mut hash_lock = [0u8; 32];
        thread_rng().fill_bytes(&mut hash_lock);
        let htlc_params = HTLCParameters {
            hash_lock,
            timeout_height: 144,
            refund_pubkey: public_key,
            amount: Amount::from_sat(100_000),
        };

        let script = manager.create_htlc_script(&htlc_params, &stealth_address).unwrap();
        assert!(!script.is_empty());
    }

    #[test]
    fn test_cross_chain_script() {
        let manager = ScriptManager::new(Network::Regtest);
        
        // Generate keys
        let secret_key = SecretKey::new(&mut thread_rng());
        let public_key = PublicKey::from_secret_key(&manager.secp, &secret_key);

        // Create stealth address
        let mut view_tag = [0u8; 32];
        thread_rng().fill_bytes(&mut view_tag);
        let stealth_address = StealthAddress {
            scan_pubkey: public_key.inner,
            spend_pubkey: public_key.inner,
            view_tag,
        };

        // Create HTLC parameters
        let mut hash_lock = [0u8; 32];
        thread_rng().fill_bytes(&mut hash_lock);
        let htlc_params = HTLCParameters {
            hash_lock,
            timeout_height: 144,
            refund_pubkey: public_key,
            amount: Amount::from_sat(100_000),
        };

        let metadata = OpReturnMetadata {
            bridge_id: [0u8; 32],
            merkle_root: [0u8; 32],
            data: vec![],
        };

        let (htlc_script, op_return_script) = manager
            .create_cross_chain_script(&htlc_params, &stealth_address, &metadata)
            .unwrap();

        assert!(!htlc_script.is_empty());
        assert!(!op_return_script.is_empty());
        assert!(op_return_script.is_op_return());
    }

    #[test]
    fn test_stealth_payment_scanning() {
        let manager = ScriptManager::new(Network::Regtest);
        
        // Generate keys
        let scan_key = SecretKey::new(&mut thread_rng());
        let spend_key = SecretKey::new(&mut thread_rng());
        let scan_pubkey = PublicKey::from_secret_key(&manager.secp, &scan_key);
        let spend_pubkey = PublicKey::from_secret_key(&manager.secp, &spend_key);

        // Create stealth address
        let mut view_tag = [0u8; 32];
        thread_rng().fill_bytes(&mut view_tag);
        let stealth_address = StealthAddress {
            scan_pubkey: scan_pubkey.inner,
            spend_pubkey: spend_pubkey.inner,
            view_tag,
        };

        // Create transaction with stealth payment
        let ephemeral_key = SecretKey::new(&mut thread_rng());
        let stealth_script = manager.create_stealth_payment_script(&stealth_address, &ephemeral_key).unwrap();
        
        let tx = Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![TxOut {
                value: 100_000,
                script_pubkey: stealth_script,
            }],
        };

        let found = manager.scan_for_stealth_payments(&tx, &stealth_address, &scan_key).unwrap();
        assert_eq!(found.len(), 1);
    }
}
