use bitcoin::Network;
use bitcoin::bip32::ExtendedPrivKey;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use tokio::sync::RwLock;    
use crate::common::types::ops::{OpCode, WalletOpCode};
use bitcoin::{
    hashes::{sha256d, Hash, HashEngine},
    secp256k1::{PublicKey, SecretKey, Secp256k1},
    Transaction, TxOut,
};
use thiserror::Error;
use std::{collections::HashMap, sync::Arc};
#[derive(Error, Debug)]
pub enum BitcoinStateError {
    #[error("Invalid lock amount: {0}")]
    InvalidLockAmount(String),
    #[error("Invalid script hash: {0}")]
    InvalidScriptHash(String),
    #[error("Invalid lock height: {0}")]
    InvalidLockHeight(String),
    #[error("Invalid sequence: {0}")]
    InvalidSequence(String),
    #[error("Invalid stealth address: {0}")]
    InvalidStealthAddress(String),
    #[error("Invalid HTLC parameters: {0}")]
    InvalidHTLC(String),
    #[error("Invalid preimage: {0}")]
    InvalidPreimage(String),
    #[error("Verification failed: {0}")]
    VerificationError(String),
    #[error("Encoding error: {0}")]
    EncodingError(String),
    #[error("Crypto error: {0}")]
    CryptoError(String),
    #[error("State error: {0}")]
    StateError(String),
}

// CrossChainState

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CrossChainState {
    pub bitcoin_state: BitcoinState,
    pub ethereum_state: EthereumState,
}
// BitcoinState 
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]    
pub struct BitcoinState {
    pub stealth_addresses: Vec<StealthAddress>,
    pub bitcoin_wallet: BitcoinWallet,
    pub bitcoin_node: BitcoinNode,
}




// BridgeParameters
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BridgeParameters {
    pub bitcoin_node: BitcoinNode,
    pub ethereum_node: EthereumNode,
    pub ethereum_private_key: SecretKey,
}

// EthereumState
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EthereumState {
    pub bridge_parameters: BridgeParameters,
    pub ethereum_wallet: EthereumWallet,
    pub ethereum_node: EthereumNode,
}

// BitcoinNode
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BitcoinNode {
    pub network: Network,
    pub node_url: String,
    pub rpc_user: String,
    pub rpc_password: String,
}

// EthereumNode
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EthereumNode {
    pub network: Network,
    pub node_url: String,
    pub rpc_user: String,
    pub rpc_password: String,
}

// BitcoinWallet
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct BitcoinWallet {
    pub network: Network,
    pub wallet_name: String,
    pub wallet_password: String,
    pub mnemonic: String,
    pub seed: Vec<u8>,
    pub master_key: ExtendedPrivKey,
}

// EthereumWallet
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EthereumWallet {
    pub network: Network,
    pub wallet_name: String,
    pub wallet_password: String,
    pub mnemonic: String,
    pub seed: Vec<u8>,
    pub master_key: ExtendedPrivKey,
}



pub type Result<T> = std::result::Result<T, BitcoinStateError>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct StealthAddress {
    #[serde(with = "serde_arrays")]
    pub ephemeral_public_key: [u8; 33],
    #[serde(with = "serde_arrays")]
    pub payment_code: [u8; 32],
    pub view_tag: u8,
    #[serde(with = "serde_arrays")]
    pub spend_pubkey: [u8; 33],
    #[serde(with = "serde_arrays")]
    pub scan_pubkey: [u8; 33],
    metadata: HashMap<String, Vec<u8>>,
}

impl StealthAddress {
    pub fn new(
        scan_key: &SecretKey,
        spend_key: &SecretKey,
        secp: &Secp256k1<bitcoin::secp256k1::All>,
    ) -> Result<Self> {
        let ephemeral_key = SecretKey::new(&mut rand::thread_rng());
        let ephemeral_pubkey = PublicKey::from_secret_key(secp, &ephemeral_key);
        
        let shared_secret = Self::compute_shared_secret(
            &ephemeral_key,
            &PublicKey::from_secret_key(secp, scan_key),
            secp,
        )?;

        let spend_pubkey = PublicKey::from_secret_key(secp, spend_key);
        let scan_pubkey = PublicKey::from_secret_key(secp, scan_key);

        Ok(Self {
            ephemeral_public_key: ephemeral_pubkey.serialize(),
            payment_code: shared_secret,
            view_tag: shared_secret[0],
            spend_pubkey: spend_pubkey.serialize(),
            scan_pubkey: scan_pubkey.serialize(),
            metadata: HashMap::new(),
        })
    }

    pub fn scan_outputs(&self, tx: &Transaction, scan_key: &SecretKey) -> Result<Vec<TxOut>> {
        let secp = Secp256k1::new();
        let mut found_outputs = Vec::new();

        for output in &tx.output {
            if self.scan_output(output, scan_key, &secp)? {
                found_outputs.push(output.clone());
            }
        }

        Ok(found_outputs)
    }

    pub fn derive_spending_key(&self, scan_key: &SecretKey) -> Result<SecretKey> {
        let secp = Secp256k1::new();
        let shared_secret = self.compute_shared_secret_from_scan(scan_key, &secp)?;
        
        let mut spending_key = [0u8; 32];
        for i in 0..32 {
            spending_key[i] = shared_secret[i] ^ scan_key[i];
        }

        SecretKey::from_slice(&spending_key)
            .map_err(|e| BitcoinStateError::CryptoError(e.to_string()))
    }

    fn compute_shared_secret(
        ephemeral_key: &SecretKey,
        scan_pubkey: &PublicKey,
        secp: &Secp256k1<bitcoin::secp256k1::All>,
    ) -> Result<[u8; 32]> {
        let shared_point = scan_pubkey.mul_tweak(secp, &(*ephemeral_key).into())
            .map_err(|e| BitcoinStateError::CryptoError(e.to_string()))?;

        let mut hasher = sha256d::Hash::engine();
        hasher.input(&shared_point.serialize());
        Ok(sha256d::Hash::from_engine(hasher).to_byte_array())
    }

    fn compute_shared_secret_from_scan(
        &self,
        scan_key: &SecretKey,
        secp: &Secp256k1<bitcoin::secp256k1::All>,
    ) -> Result<[u8; 32]> {
        let ephemeral_pubkey = PublicKey::from_slice(&self.ephemeral_public_key)
            .map_err(|e| BitcoinStateError::CryptoError(e.to_string()))?;
        
        Self::compute_shared_secret(scan_key, &ephemeral_pubkey, secp)
    }

    fn scan_output(
        &self,
        output: &TxOut,
        scan_key: &SecretKey,
        secp: &Secp256k1<bitcoin::secp256k1::All>,
    ) -> Result<bool> {
        let script_pubkey = &output.script_pubkey;
        if !script_pubkey.is_p2pkh() && !script_pubkey.is_p2pkh() {
            return Ok(false);
        }

        let derived_key = self.derive_spending_key(scan_key)?;
        let derived_pubkey = PublicKey::from_secret_key(secp, &derived_key);
        
        Ok(script_pubkey.as_bytes().iter().any(|&byte| derived_pubkey.serialize().contains(&byte)))
    }}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HTLCParameters {
    pub amount: u64,
    #[serde(with = "serde_arrays")]
    pub receiver: [u8; 20],
    #[serde(with = "serde_arrays")]
    pub hash_lock: [u8; 32],
    pub timeout_height: u32,
    pub stealth_address: Option<StealthAddress>,
    pub rebalancing_proof: Option<Vec<u8>>,
    pub state_merkle_proof: Option<Vec<u8>>,
}

impl Default for HTLCParameters {
    fn default() -> Self {
        Self {
            amount: 0,
            receiver: [0u8; 20],
            hash_lock: [0u8; 32],
            timeout_height: 0,
            stealth_address: None,
            rebalancing_proof: None,
            state_merkle_proof: None,
        }
    }
}

impl HTLCParameters {
    pub fn new(
        amount: u64,
        receiver: [u8; 20],
        hash_lock: [u8; 32],
        timeout_height: u32,
        stealth_address: Option<StealthAddress>,
    ) -> Self {
        Self {
            amount,
            receiver,
            hash_lock,
            timeout_height,
            stealth_address,
            rebalancing_proof: None,
            state_merkle_proof: None,
        }
    }

    pub fn verify_timelock(&self, current_height: u32) -> bool {
        current_height >= self.timeout_height
    }

    pub fn verify_hashlock(&self, preimage: &[u8]) -> Result<bool> {
        let mut hasher = sha256d::Hash::engine();
        hasher.input(preimage);
        let hash = sha256d::Hash::from_engine(hasher).to_byte_array();
        Ok(hash == self.hash_lock)
    }

    pub fn generate_witness_script(&self) -> Box<bitcoin::Script> {
        let mut builder = bitcoin::blockdata::script::Builder::new()
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_HASH256)
            .push_slice(&self.hash_lock)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_EQUALVERIFY)
            .push_slice(&self.receiver)
            .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIG);

        if let Some(stealth) = &self.stealth_address {
            builder = builder
                .push_slice(&stealth.ephemeral_public_key)
                .push_opcode(bitcoin::blockdata::opcodes::all::OP_CHECKSIGVERIFY);
        }
        builder.into_script().into()
    }
    pub fn estimate_witness_size(&self) -> usize {
        let base_size = 107; // Base witness size for HTLC
        match &self.stealth_address {
            Some(_) => base_size + 74, // Additional size for stealth address verification
            None => base_size,
        }
    }

    pub fn verify_rebalancing(&self) -> Result<bool> {
        match (&self.rebalancing_proof, &self.state_merkle_proof) {
            (Some(rebalancing), Some(merkle)) => {
                // Verify rebalancing proof against merkle proof
                self.verify_rebalancing_internal(rebalancing, merkle)
            }
            _ => Ok(false),
        }
    }

    fn verify_rebalancing_internal(&self, rebalancing: &[u8], merkle: &[u8]) -> Result<bool> {
        // Implementation of rebalancing verification logic
        if rebalancing.is_empty() || merkle.is_empty() {
            return Ok(false);
        }

        // Verify merkle proof
        let mut hasher = sha256d::Hash::engine();
        hasher.input(rebalancing);
        hasher.input(merkle);
        let hash = sha256d::Hash::from_engine(hasher);

        // Verify against hash_lock
        Ok(hash.to_byte_array() == self.hash_lock)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpReturnMetadata {
    pub version: u8,
    pub stealth_address: Option<StealthAddress>,
    pub rebalancing_flags: u8,
    #[serde(with = "serde_arrays")]
    pub channel_id: [u8; 32],
    pub metadata: Vec<u8>,
    pub proof_data: Option<Vec<u8>>,
}

impl OpReturnMetadata {
    pub const MAX_SIZE: usize = 80;
    pub const PROTOCOL_IDENTIFIER: [u8; 4] = *b"OVPS";

    pub fn new(
        channel_id: [u8; 32],
        stealth_address: Option<StealthAddress>,
        rebalancing_flags: u8,
    ) -> Self {
        Self {
            version: 1,
            stealth_address,
            rebalancing_flags,
            channel_id,
            metadata: Vec::new(),
            proof_data: None,
        }
    }

    pub fn encode_for_bitcoin(&self) -> Result<Vec<u8>> {
        let mut data = Vec::with_capacity(Self::MAX_SIZE);
        data.extend_from_slice(&Self::PROTOCOL_IDENTIFIER);
        data.push(self.version);
        data.extend_from_slice(&self.channel_id);
        data.push(self.rebalancing_flags);

        if let Some(stealth) = &self.stealth_address {
            data.push(1);
            data.extend_from_slice(&stealth.payment_code);
        } else {
            data.push(0);
        }

        if data.len() > Self::MAX_SIZE {
            return Err(BitcoinStateError::EncodingError(
                "OP_RETURN data exceeds maximum size".into()
            ));
        }

        Ok(data)
    }

    pub fn decode_from_bitcoin(data: &[u8]) -> Result<Self> {
        if data.len() < 38 {
            return Err(BitcoinStateError::EncodingError(
                "Invalid OP_RETURN data length".into()
            ));
        }

        if &data[0..4] != &Self::PROTOCOL_IDENTIFIER {
            return Err(BitcoinStateError::EncodingError(
                "Invalid protocol identifier".into()
            ));
        }

        let version = data[4];
        let mut channel_id = [0u8; 32];
        channel_id.copy_from_slice(&data[5..37]);
        let rebalancing_flags = data[37];

        let stealth_address = if data.len() > 38 && data[38] == 1 {
            if data.len() < 71 {
                return Err(BitcoinStateError::EncodingError(
                    "Invalid stealth address data length".into()
                ));
            }
            let mut payment_code = [0u8; 32];
            payment_code.copy_from_slice(&data[39..71]);
            Some(StealthAddress {
                payment_code,
                ephemeral_public_key: [0u8; 33],
                view_tag: 0,
                spend_pubkey: [0u8; 33],
                scan_pubkey: [0u8; 33],
                metadata: HashMap::new(),
            })
        } else {
            None
        };

        Ok(Self {
            version,
            stealth_address,
            rebalancing_flags,
            channel_id,
            metadata: Vec::new(),
            proof_data: None,
        })
    }

    pub fn verify_rebalancing_proof(&self, proof: &[u8]) -> Result<bool> {
        if proof.is_empty() {
            return Ok(false);
        }

        if self.proof_data.is_none() {
            return Ok(false);
        }

        let proof_data = self.proof_data.as_ref().unwrap();
    
        // Verify proof length
        if proof.len() != 64 {
            return Ok(false);
        }

        // Extract signature components
        let mut r = [0u8; 32];
        let mut s = [0u8; 32];
        r.copy_from_slice(&proof[0..32]);
        s.copy_from_slice(&proof[32..64]);

        // Verify signature matches proof data
        let secp = secp256k1::Secp256k1::new();
        let message = secp256k1::Message::from_slice(proof_data)
            .map_err(|_| BitcoinStateError::EncodingError("Invalid proof data".into()))?;
        let signature = secp256k1::ecdsa::Signature::from_compact(&[&r[..], &s[..]].concat())
            .map_err(|_| BitcoinStateError::EncodingError("Invalid signature".into()))?;
        let public_key = secp256k1::PublicKey::from_slice(&self.channel_id)
        .map_err(|_| BitcoinStateError::EncodingError("Invalid public key".into()))?;
    
        let signature_valid = secp.verify_ecdsa(&message, &signature, &public_key).is_ok();

        // Check rebalancing flags
        let rebalancing_enabled = self.rebalancing_flags & 0x01 == 0x01;

        Ok(signature_valid && rebalancing_enabled)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinLockState {
    pub lock_amount: u64,
    #[serde(with = "serde_arrays")]
    pub lock_script_hash: [u8; 32],
    pub lock_height: u64,
    #[serde(with = "serde_arrays")]
    pub pubkey_hash: [u8; 20],
    pub sequence: u32,
    pub htlc_params: Option<HTLCParameters>,
    pub op_return: Option<OpReturnMetadata>,
    #[serde(skip)]
    state_cache: Arc<RwLock<HashMap<[u8; 32], Vec<u8>>>>,
}
impl BitcoinLockState {
    pub fn new(
        lock_amount: u64,
        lock_script_hash: [u8; 32],
        lock_height: u64,
        pubkey_hash: [u8; 20],
        sequence: u32,
        htlc_params: Option<HTLCParameters>,
        op_return: Option<OpReturnMetadata>,
    ) -> Result<Self> {
        let params = HTLCParameters::default();
        
        if lock_amount < params.amount {
            return Err(BitcoinStateError::InvalidLockAmount(format!(
                "Lock amount must be at least {} satoshis",
                params.amount
            )));
        }

        if lock_height as u32 > params.timeout_height {
            return Err(BitcoinStateError::InvalidLockHeight(format!(
                "Lock height cannot exceed {}", 
                params.timeout_height
            )));
        }
        Ok(Self {
            lock_amount,
            lock_script_hash,
            lock_height,
            pubkey_hash,
            sequence,
            htlc_params,
            op_return,
            state_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn compute_state_hash(&self) -> Result<[u8; 32]> {
        let mut engine = sha256d::Hash::engine();
        
        engine.input(&self.lock_amount.to_le_bytes());
        engine.input(&self.lock_script_hash);
        engine.input(&self.lock_height.to_le_bytes());
        engine.input(&self.pubkey_hash);
        engine.input(&self.sequence.to_le_bytes());

        if let Some(htlc) = &self.htlc_params {
            let htlc_bytes = bincode::serialize(htlc).map_err(|e| BitcoinStateError::VerificationError(e.to_string()))?;
            engine.input(&htlc_bytes);
        }

        if let Some(op_return) = &self.op_return {
            let op_return_bytes = op_return.encode_for_bitcoin()?;
            engine.input(&op_return_bytes);
        }

        Ok(sha256d::Hash::from_engine(engine).to_byte_array())
    }
    pub async fn verify_state_transition(&self, next_state: &BitcoinLockState) -> Result<bool> {
        // Verify basic state transition rules
        if next_state.lock_height <= self.lock_height {
            return Ok(false);
        }

        // Verify HTLC conditions if present
        if let Some(htlc) = &self.htlc_params {
            if !htlc.verify_timelock(next_state.lock_height as u32) {
                return Ok(false);
            }
        }

        // Verify state hash consistency
        let current_hash = self.compute_state_hash().await?;
        let next_hash = next_state.compute_state_hash().await?;
        
        if current_hash == next_hash {
            return Ok(false);
        }

        // Cache the verified state transition
        let mut cache = self.state_cache.write().await;
        cache.insert(next_hash, bincode::serialize(next_state).map_err(|e| BitcoinStateError::VerificationError(e.to_string()))?);

        Ok(true)
    }}
#[cfg(test)]
mod tests {
    use bitcoin::opcodes::all;
use bitcoin::Script;
use bitcoin::absolute::LockTime;
    use super::*;
    use bitcoin::secp256k1::Secp256k1;

    fn setup_test_environment() -> (Secp256k1<bitcoin::secp256k1::All>, SecretKey, SecretKey) {
        let secp = Secp256k1::new();
        let scan_key = SecretKey::new(&mut rand::thread_rng());
        let spend_key = SecretKey::new(&mut rand::thread_rng());
        (secp, scan_key, spend_key)
    }

    #[test]
    fn test_stealth_address_creation_and_scanning() {
        let (secp, scan_key, spend_key) = setup_test_environment();
        
        let stealth_address = StealthAddress::new(&scan_key, &spend_key, &secp).unwrap();
        assert_eq!(stealth_address.view_tag, stealth_address.payment_code[0]);
        
        // Create test transaction
        let mut tx = Transaction {
            version: 2,
            lock_time: LockTime::ZERO,
            input: vec![],
            output: vec![],
        };
        // Create test output script
        
        let ephemeral_key = SecretKey::from_slice(&[0; 32]).unwrap();
        let ephemeral_pubkey = bitcoin::PublicKey::from_private_key(&secp, &bitcoin::PrivateKey::new(ephemeral_key, bitcoin::Network::Bitcoin));
        let script = Script::builder()
            .push_key(&ephemeral_pubkey)
            .push_opcode(all::OP_CHECKSIG)
            .into_script();
        
    
        tx.output.push(TxOut {
            value: 50000,
            script_pubkey: script,
        });

        let found_outputs = stealth_address.scan_outputs(&tx, &scan_key).unwrap();
        assert_eq!(found_outputs.len(), 1);
    }    #[tokio::test]    async fn test_htlc_operations() {        let htlc = HTLCParameters::new(            
            100,
            [0u8; 20],
            [1u8; 32],
            1000,
            None,
        );
        assert!(htlc.verify_timelock(100));
        assert!(!htlc.verify_timelock(101));
        assert!(htlc.verify_timelock(1000));
        assert!(!htlc.verify_timelock(1001));
        assert!(htlc.verify_timelock(20000));
        assert!(!htlc.verify_timelock(20001));
        assert!(htlc.verify_timelock(30000));
        assert!(!htlc.verify_timelock(30001));
        assert!(htlc.verify_timelock(40000));
        assert!(!htlc.verify_timelock(40001));
    }  
   
    #[tokio::test]
    async fn test_htlc_operations_async() {
        let htlc = HTLCParameters::new(
            1_000_000,
            [1u8; 20],
            [2u8; 32],
            750_000,
            None,
        );

        assert!(!htlc.verify_timelock(749_999));
        assert!(htlc.verify_timelock(750_000));
        
        let preimage = b"test_preimage";
        let mut hasher = sha256d::Hash::engine();
        hasher.input(preimage);
        let hash = sha256d::Hash::from_engine(hasher).to_byte_array();
        
        let htlc_with_hash = HTLCParameters::new(
            1_000_000,
            [1u8; 20],
            hash,
            750_000,
            None,
        );        
        assert!(htlc_with_hash.verify_hashlock(preimage).unwrap());
    }
    #[tokio::test]
    async fn test_op_return_metadata() {
        let channel_id = [3u8; 32];
        let metadata = OpReturnMetadata::new(
            channel_id,
            None,
            1,
        );

        let encoded = metadata.encode_for_bitcoin().unwrap();
        assert!(encoded.len() <= OpReturnMetadata::MAX_SIZE);

        let decoded = OpReturnMetadata::decode_from_bitcoin(&encoded).unwrap();
        assert_eq!(decoded.channel_id, channel_id);
        assert_eq!(decoded.rebalancing_flags, 1);
    }

    #[tokio::test]
    async fn test_bitcoin_lock_state_transitions() {
        let initial_state = BitcoinLockState::new(
            1_000_000,
            [0u8; 32],
            750_000,
            [1u8; 20],
            0xFFFFFFFF,
            None,
            None,
        ).unwrap();

        let next_state = BitcoinLockState::new(
            900_000,
            [0u8; 32],
            750_001,
            [1u8; 20],
            0xFFFFFFFF,
            None,
            None,
        ).unwrap();

        assert!(initial_state.verify_state_transition(&next_state).await.unwrap());
    }

    #[test]
    fn test_witness_script_generation() {
        let htlc = HTLCParameters::new(
            1_000_000,
            [1u8; 20],
            [2u8; 32],
            750_000,
            None,
        );

        let script = htlc.generate_witness_script();
        assert!(!script.is_empty());
        assert!(script.is_witness_program());
    }
}