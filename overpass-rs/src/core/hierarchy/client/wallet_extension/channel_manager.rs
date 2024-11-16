use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use wasm_bindgen::prelude::*;

use crate::core::hierarchy::client::channel::channel_contract::ChannelContract;

// Type alias for ChannelStore
type ChannelStore = Arc<RwLock<HashMap<[u8; 32], Arc<RwLock<ChannelContract>>>>>;

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub timeout: u64,
    pub min_balance: u64,
    pub max_balance: u64,
}

#[wasm_bindgen]
pub struct ChannelManager {
    channels: ChannelStore,
    proof_system: Arc<Plonky2SystemHandle>,
    wallet_id: [u8; 32],
    spending_limit: u64,
}

#[wasm_bindgen]
impl ChannelManager {
    #[wasm_bindgen(constructor)]
    pub fn new() -> ChannelManager {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            proof_system: Arc::new(Plonky2SystemHandle::new()),
            wallet_id: [0; 32],
            spending_limit: 0,
        }
    }

    #[wasm_bindgen]
    pub fn new_with_wallet(wallet_id: &[u8], spending_limit: u64) -> ChannelManager {
        let mut id = [0u8; 32];
        id.copy_from_slice(&wallet_id[0..32]);
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            proof_system: Arc::new(Plonky2SystemHandle::new()),
            wallet_id: id,
            spending_limit,
        }
    }

    #[wasm_bindgen]
    pub async fn dispatch(&self, op_code: u8, params: &[u8]) -> Result<Box<[u8]>, JsValue> {
        let op_code =
            ChannelOpCode::from_u8(op_code).ok_or_else(|| JsValue::from_str("Invalid op_code"))?;
        match op_code {
            ChannelOpCode::GetChannel => {
                let channel_id = decode_channel_id(params)?;
                let channel = self.get_channel(&channel_id)?;
                Ok(serialize_channel(&channel).into_boxed_slice())
            }
            ChannelOpCode::InitChannel => {
                let (sender, recipient, initial_balance, config) = decode_create_params(params)?;
                let channel_id =
                    self.create_channel(sender, recipient, initial_balance, &config)?;
                Ok(channel_id.to_vec().into_boxed_slice())
            }
            ChannelOpCode::UpdateState => {
                let (channel_id, new_state) = decode_update_params(params)?;
                self.update_channel_state(&channel_id, new_state)?;
                Ok(vec![].into_boxed_slice())
            }
            ChannelOpCode::VerifyProof => {
                let (channel_id, proof, old_balance, new_balance) = decode_verify_params(params)?;
                let channel = self.get_channel(&channel_id)?;
                let channel = channel
                    .read()
                    .map_err(|_| JsValue::from_str("InvalidTransaction"))?;
                let is_valid = self.verify_proof(&channel, &proof, old_balance, new_balance)?;
                Ok(vec![is_valid as u8].into_boxed_slice())
            }
        }
    }

    fn get_channel(&self, channel_id: &[u8; 32]) -> Result<Arc<RwLock<ChannelContract>>, JsValue> {
        self.channels
            .read()
            .map_err(|_| JsValue::from_str("InvalidTransaction"))?
            .get(channel_id)
            .cloned()
            .ok_or_else(|| JsValue::from_str("Channel not found"))
    }

    fn create_channel(
        &self,
        sender: [u8; 32],
        recipient: [u8; 32],
        initial_balance: u64,
        _config: &ChannelConfig,
    ) -> Result<[u8; 32], JsValue> {
        let mut hasher = Sha256::new();
        hasher.update(sender);
        hasher.update(recipient);
        hasher.update(initial_balance.to_le_bytes());
        let hash = hasher.finalize();
        let mut channel_id = [0u8; 32];
        channel_id.copy_from_slice(&hash);

        let mut channels = self
            .channels
            .write()
            .map_err(|_| JsValue::from_str("InvalidTransaction"))?;
        if channels.contains_key(&channel_id) {
            return Err(JsValue::from_str("InvalidChannel"));
        }

        let channel = ChannelContract::new(&hex::encode(channel_id));
        let channel = Arc::new(RwLock::new(channel));

        {
            let mut channel_lock = channel
                .write()
                .map_err(|_| JsValue::from_str("InvalidTransaction"))?;
            channel_lock
                .update_balance(initial_balance)
                .map_err(|_| JsValue::from_str("InvalidAmount"))?;
        }

        channels.insert(channel_id, channel);
        Ok(channel_id)
    }    fn update_channel_state(
        &self,
        channel_id: &[u8; 32],
        new_state: Vec<u8>,
    ) -> Result<(), JsValue> {
        let channel = self.get_channel(channel_id)?;
        let mut channel = channel
            .write()
            .map_err(|_| JsValue::from_str("InvalidTransaction"))?;

        if !self.validate_channel(&channel)? {
            return Err(JsValue::from_str("InvalidChannel"));
        }

        if new_state.len() < 8 {
            return Err(JsValue::from_str("InvalidTransaction"));
        }

        let mut balance_bytes = [0u8; 8];
        balance_bytes.copy_from_slice(&new_state[0..8]);
        let balance = u64::from_le_bytes(balance_bytes);

        channel
            .update_balance(balance)
            .map_err(|_| JsValue::from_str("InvalidAmount"))?;

        Ok(())
    }

    fn validate_channel(&self, channel: &ChannelContract) -> Result<bool, JsValue> {
        if channel.balance() > self.spending_limit {
            return Err(JsValue::from_str("Spending limit exceeded"));
        }
        Ok(true)
    }

    fn verify_proof(
        &self,
        channel: &ChannelContract,
        _proof: &ZkProof,
        old_balance: u64,
        new_balance: u64,
    ) -> Result<bool, JsValue> {
        self.validate_channel(channel)?;

        let mut proof_data = Vec::new();
        proof_data.extend_from_slice(&old_balance.to_le_bytes());
        proof_data.extend_from_slice(&channel.nonce().to_le_bytes());
        proof_data.extend_from_slice(&new_balance.to_le_bytes());
        proof_data.extend_from_slice(&(channel.nonce() + 1).to_le_bytes());
        proof_data.extend_from_slice(&new_balance.saturating_sub(old_balance).to_le_bytes());

        self.proof_system
            .verify_proof_js(&proof_data)
            .map_err(|_| JsValue::from_str("InvalidProof"))
    }
}

fn decode_channel_id(params: &[u8]) -> Result<[u8; 32], JsValue> {
    if params.len() != 32 {
        return Err(JsValue::from_str("InvalidTransaction"));
    }
    let mut channel_id = [0u8; 32];
    channel_id.copy_from_slice(params);
    Ok(channel_id)
}

fn decode_create_params(
    params: &[u8],
) -> Result<([u8; 32], [u8; 32], u64, ChannelConfig), JsValue> {
    if params.len() < 80 {
        return Err(JsValue::from_str("InvalidTransaction"));
    }

    let mut sender = [0u8; 32];
    sender.copy_from_slice(&params[0..32]);

    let mut recipient = [0u8; 32];
    recipient.copy_from_slice(&params[32..64]);

    let mut balance_bytes = [0u8; 8];
    balance_bytes.copy_from_slice(&params[64..72]);
    let initial_balance = u64::from_le_bytes(balance_bytes);

    let mut timeout_bytes = [0u8; 8];
    timeout_bytes.copy_from_slice(&params[72..80]);
    let timeout = u64::from_le_bytes(timeout_bytes);

    let config = ChannelConfig {
        timeout,
        min_balance: 0,
        max_balance: u64::MAX,
    };

    Ok((sender, recipient, initial_balance, config))
}

fn decode_update_params(params: &[u8]) -> Result<([u8; 32], Vec<u8>), JsValue> {
    if params.len() <= 32 {
        return Err(JsValue::from_str("InvalidTransaction"));
    }

    let mut channel_id = [0u8; 32];
    channel_id.copy_from_slice(&params[0..32]);
    let new_state = params[32..].to_vec();

    Ok((channel_id, new_state))
}

fn decode_verify_params(params: &[u8]) -> Result<([u8; 32], ZkProof, u64, u64), JsValue> {
    if params.len() < 32 + 64 + 16 {
        return Err(JsValue::from_str("InvalidProofDataLength"));
    }

    let mut channel_id = [0u8; 32];
    channel_id.copy_from_slice(&params[0..32]);

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| JsValue::from_str("InvalidTimestamp"))?
        .as_secs();

    let public_inputs = Vec::new();
    let merkle_root = vec![0u8; 32];

    let proof = ZkProof::new(
        params[32..96].to_vec(),
        public_inputs,
        merkle_root,
        timestamp,
    );

    let mut old_balance_bytes = [0u8; 8];
    old_balance_bytes.copy_from_slice(&params[96..104]);
    let old_balance = u64::from_le_bytes(old_balance_bytes);

    let mut new_balance_bytes = [0u8; 8];
    new_balance_bytes.copy_from_slice(&params[104..112]);
    let new_balance = u64::from_le_bytes(new_balance_bytes);

    Ok((channel_id, proof, old_balance, new_balance))
}

fn serialize_channel(channel: &Arc<RwLock<ChannelContract>>) -> Vec<u8> {
    let channel = channel.read().unwrap();
    let mut serialized = Vec::with_capacity(32 + 24);

    if let Ok(id_bytes) = hex::decode(&channel.id()) {
        serialized.extend_from_slice(&id_bytes);
    }

    serialized.extend_from_slice(&channel.balance().to_le_bytes());
    serialized.extend_from_slice(&channel.nonce().to_le_bytes());
    serialized.extend_from_slice(&channel.seqno().to_le_bytes());

    serialized
}


#[wasm_bindgen]
#[derive(Debug)]
pub enum ChannelOpCode {
    GetChannel = 0,
    InitChannel = 1,
    UpdateState = 2,
    VerifyProof = 3,
}

impl ChannelOpCode {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ChannelOpCode::GetChannel),
            1 => Some(ChannelOpCode::InitChannel),
            2 => Some(ChannelOpCode::UpdateState),
            3 => Some(ChannelOpCode::VerifyProof),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Plonky2SystemHandle;

impl Plonky2SystemHandle {
    pub fn new() -> Self {
        Plonky2SystemHandle
    }

    pub fn verify_proof_js(&self, _proof_data: &[u8]) -> Result<bool, JsValue> {
        Ok(true)
    }
}

#[derive(Debug)]
pub struct ZkProof {
    proof: Vec<u8>,
    public_inputs: Vec<u8>,
    merkle_root: Vec<u8>,
    timestamp: u64,
}

impl ZkProof {
    pub fn new(
        proof: Vec<u8>,
        public_inputs: Vec<u8>,
        merkle_root: Vec<u8>,
        timestamp: u64,
    ) -> Self {
        ZkProof {
            proof,
            public_inputs,
            merkle_root,
            timestamp,
        }
    }
}
