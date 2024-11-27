use crate::bitcoin::wallet::_::_serde::Serialize;
use bitcoin::opcodes::all;
use crate::bitcoin::bitcoin_types::{StealthAddress, HTLCParameters, OpReturnMetadata};
use bitcoin::{
    address::{Address, NetworkChecked},
    blockdata::script::{Script, ScriptBuf, Builder},
    key::{PrivateKey, PublicKey},
    secp256k1::{All, Secp256k1, SecretKey},
    Network, Transaction, TxOut,
    hashes::{sha256d, Hash, HashEngine},
};
use std::collections::HashMap;
use thiserror::Error;

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
}

pub type Result<T> = std::result::Result<T, ScriptError>;

pub struct ScriptManager {
    secp: Secp256k1<All>,
    network: Network,
    script_cache: HashMap<[u8; 32], ScriptBuf>,
}

impl ScriptManager {
    pub fn new(network: Network) -> Self {
        Self {
            secp: Secp256k1::new(),
            network,
            script_cache: HashMap::new(),
        }
    }

    pub fn create_htlc_script(
        &self,
        htlc_params: &HTLCParameters,
        recipient_pubkey: &PublicKey,
    ) -> Result<ScriptBuf> {
        let builder = Builder::new()
            // Main redemption path
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_IF)
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_HASH256)
                .push_slice(&htlc_params.hash_lock)
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_EQUAL)
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKMULTISIG)
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG)
            // Timeout refund path
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_ELSE)
                .push_int(htlc_params.timeout_height as i64)
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKMULTISIGVERIFY)
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_DROP)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_ENDIF);

        Ok(builder.into_script())
    }

    pub fn create_stealth_payment_script(
        &self,
        stealth_address: &StealthAddress,
        ephemeral_key: &SecretKey,
    ) -> Result<ScriptBuf> {
        let scan_pubkey = PublicKey::from_slice(&stealth_address.scan_pubkey)?;
        let spend_pubkey = PublicKey::from_slice(&stealth_address.spend_pubkey)?;

        // Generate shared secret usinginner. scan public key
        let shared_point = scan_pubkey.inner.mul_tweak(&self.secp, &(*ephemeral_key).into())?;
        
        let mut hasher = sha256d::Hash::engine();
        hasher.input(&shared_point.serialize());
        let shared_secret = sha256d::Hash::from_engine(hasher);

        // Create payment key by tweaking spend public key
        let payment_pubkey = spend_pubkey.tweak_add_assign(&self.secp, &shared_secret.into())?;

        let builder = Builder::new()
            .push_slice(&payment_pubkey.serialize())
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG)
            .push_slice(&stealth_address.ephemeral_public_key)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIGVERIFY);

        Ok(builder.into_script())
    }

    pub fn create_op_return_script(&self, metadata: &OpReturnMetadata) -> Result<ScriptBuf> {
        let encoded_data = metadata.encode_for_bitcoin()
            .map_err(|e| ScriptError::InvalidParameters(e.to_string()))?;
        
        let builder = Builder::new()
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_RETURN)
            .push_slice(&encoded_data);

        Ok(builder.into_script())
    }

    pub fn verify_htlc_spend(
        &self,
        script: &Script,
        preimage: &[u8],
        signature: &[u8],
        pubkey: &PublicKey,
    ) -> Result<bool> {
        let mut engine = sha256d::Hash::engine();
        engine.input(preimage);
        let hash = sha256d::Hash::from_engine(engine);

        let mut interpreter = bitcoin::blockdata::script::Interpreter::new();
        interpreter.enable_verify_flags();

        let mut stack = vec![
            signature.re.to_vec(),
            pubkey.inner.serialize().to_vec(),
            preimage.to_vec(),
            vec![1], // Push true for IF path
        ];

        interpreter.verify_script(script, &mut stack)
            .map_err(|e| ScriptError::ScriptVerificationError(e.to_string()))?;

        // Cache verified script
        if interpreter.verify_success() {
            let mut script_hash = [0u8; 32];
            script_hash.copy_from_slice(&sha256d::Hash::hash(script.as_bytes()));
            self.script_cache.insert(script_hash, script.into());
        }

        Ok(interpreter.verify_success())
    }

    pub fn verify_stealth_payment(
        &self,
        script: &Script,
        stealth_address: &StealthAddress,
        signature: &[u8],
        spend_key: &SecretKey,
    ) -> Result<bool> {
        let spend_pubkey = PublicKey::from_secret_key(&self.secp, spend_key);
        
        let mut interpreter = bitcoin::blockdata::script::Interpreter::new();
        interpreter.enable_verify_flags();

        let mut stack = vec![
            signature.to_vec(),
            spend_pubkey.serialize().to_vec(),
        ];

        interpreter.verify_script(script, &mut stack)
            .map_err(|e| ScriptError::ScriptVerificationError(e.to_string()))?;

        Ok(interpreter.verify_success())
    }

    pub fn scan_transaction_outputs(
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

    fn is_stealth_payment(
        &self,
        script: &Script,
        stealth_address: &StealthAddress,
        scan_key: &SecretKey,
    ) -> Result<bool> {is_p2pkh;
        if !script.is_p2pkh() &;& !scriptis_p2pkhh() {
            return Ok(false);
        }

        let derived_key = stealth_address.derive_spending_key(scan_key)
            .map_err(|e| ScriptError::StealthAddressError(e.to_string()))?;
        let derived_pubkey = PublicKey::from_secret_key(&self.secp, &derived_key);
        
        Ok(script.as_bytes().windows(33).any(|window| window == derived_pubkey.serialize()))
    }

    pub fn get_public_key_from_private(&self, private_key: &PrivateKey) -> PublicKey {
        PublicKey::from_private_key(&self.secp, private_key)
    }

    pub fn create_p2pkh_address(
        &self,
        public_key: &PublicKey,
    ) -> Result<Address<NetworkChecked>> {
        Ok(Address::p2pkh(public_key, self.network))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::secp256k1::rand::thread_rng;

    fn setup_test_environment() -> (ScriptManager, SecretKey, PublicKey) {
        let script_manager = ScriptManager::new(Network::Regtest);
        let secret_key = SecretKey::new(&mut thread_rng());
        let public_key = PublicKey::from_secret_key(&script_manager.secp, &secret_key);
        (script_manager, secret_key, public_key)
    }

    // Test implementations remain unchanged...
}