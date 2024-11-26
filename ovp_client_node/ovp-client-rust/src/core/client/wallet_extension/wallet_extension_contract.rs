use sha2::Digest;
use std::error::Error;
use crate::core::client::wallet_extension::channel_manager::ChannelConfig;
use crate::core::client::wallet_extension::user::ChannelState;
use crate::core::client::channel::channel_contract::ChannelContract;
use crate::core::state::sparse_merkle_tree_wasm::SparseMerkleTreeWasm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;
use crate::core::client::wallet_extension::channel_manager::Plonky2SystemHandle;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
interface ByteArray32 {
    toArray(): Uint8Array;
    toString(): string;
    fromString(val: string): ByteArray32;
}
"#;

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct ByteArray32Local(#[wasm_bindgen(skip)] pub [u8; 32]);

#[wasm_bindgen]
impl ByteArray32Local {
    #[wasm_bindgen(constructor)]
    pub fn new_from_slice(array: &[u8]) -> Result<ByteArray32Local, JsValue> {
        if array.len() != 32 {
            return Err(JsValue::from_str("Array must be 32 bytes long"));
        }
        let mut result = [0u8; 32];
        result.copy_from_slice(array);
        Ok(ByteArray32Local(result))
    }

    pub fn to_string(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_string(val: &str) -> Result<ByteArray32Local, JsValue> {
        let array = hex::decode(val)
            .map_err(|e| JsValue::from_str(&format!("Invalid hex string: {}", e)))?;
        Self::new_from_slice(&array)
    }

    pub fn to_array(&self) -> js_sys::Uint8Array {
        unsafe { js_sys::Uint8Array::view(&self.0) }
    }

    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}



pub struct WalletExtension {
    pub wallet_id: ByteArray32Local,
    pub proof_system: Arc<Plonky2SystemHandle>,
    pub state_tree: Arc<RwLock<SparseMerkleTreeWasm>>,
    pub root_hash: ByteArray32Local,
    pub balance: u64,
    pub encrypted_states: HashMap<ByteArray32Local, Vec<u8>>,
    rebalance_config: RebalanceConfig,
    total_locked_balance: i32,
    channel_id: HashMap<ByteArray32Local, ChannelContract>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RebalanceConfig {
    pub min_balance: u64,
    pub max_balance: u64,
    pub target_balance: u64,
}

impl WalletExtension {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let proof_system = Arc::new(Plonky2SystemHandle::new());
        let state_tree = Arc::new(RwLock::new(SparseMerkleTreeWasm::new()));
        
        Ok(WalletExtension {
            wallet_id: ByteArray32Local([0u8; 32]),
            channels: HashMap::new(),
            total_locked_balance: 0,
            rebalance_config: RebalanceConfig::default(),
            proof_system,
            state_tree,
            root_hash: ByteArray32Local([0u8; 32]),
            balance: 0,
            encrypted_states: HashMap::new(),
        })
    }

    pub async fn create_channel(
        &mut self,
        sender: ByteArray32Local,
        recipient: ByteArray32Local,
        deposit: u64,
        config: &ChannelConfig,
    ) -> Result<ByteArray32Local, Box<dyn Error>> {
        // Validate deposit
        if deposit > self.balance {
            return Err("Insufficient balance for deposit".into());
        }

        // Generate channel ID
        let channel_id = self.generate_channel_id(&sender, &recipient)?;

        // Check if channel already exists
        if self.channels.contains_key(&channel_id) {
            return Err("Channel already exists".into());
        }

        // Create channel contract
        let channel = ChannelContract::new(
            &sender.to_string(),
        );

        // Update state tree
        {
            let mut tree = self.state_tree.write()
                .map_err(|_| "Failed to acquire state tree lock")?;
    
            let final_state = channel.get_final_state();
            tree.update(&channel_id.0, &final_state)
                .map_err(|_| "Failed to update state tree")?;
    
            self.root_hash = ByteArray32Local(tree.root());
        }

        // Update wallet state
        self.channels.insert(channel_id, Arc::new(RwLock::new(channel)));
        self.total_locked_balance += deposit as i32;
        self.balance -= deposit;

        Ok(channel_id)
    }

    pub async fn update_channel_state(
        &mut self,
        channel_id: ByteArray32Local,
        new_state: ChannelState,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        // Get channel
        let channel = self.channels.get(&channel_id)
            .ok_or("Channel not found")?;

        // Update channel state
        let mut channel = channel.write()
            .map_err(|_| "Failed to acquire channel lock")?;

        let state_update = channel.update_balance(new_state.balance)?;

        // Update state tree
        let mut tree = self.state_tree.write()
            .map_err(|_| "Failed to acquire state tree lock")?;
        
        tree.update(&channel_id.0, &state_update)
            .map_err(|_| "Failed to update state tree")?;
        
        self.root_hash = ByteArray32Local(tree.root());

        Ok(state_update)
    }

    pub fn get_channel(&self, channel_id: &ByteArray32Local) -> Option<Arc<RwLock<ChannelContract>>> {
        self.channels.get(channel_id).cloned()
    }

    pub fn get_root_hash(&self) -> ByteArray32Local {
        self.root_hash
    }

    fn generate_channel_id(
        &self,
        sender: &ByteArray32Local,
        recipient: &ByteArray32Local,
    ) -> Result<ByteArray32Local, Box<dyn Error>> {
        use sha2::Sha256;
        let mut hasher = Sha256::new();
        hasher.update(&sender.0);
        hasher.update(&recipient.0);
        let result = hasher.finalize();
        
        let mut channel_id = [0u8; 32];
        channel_id.copy_from_slice(&result);
        Ok(ByteArray32Local(channel_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_array32_conversion() {
        let bytes = [1u8; 32];
        let array = ByteArray32Local::new_from_slice(&bytes).unwrap();
        assert_eq!(array.0, bytes);
        
        let hex_string = array.to_string();
        let from_hex = ByteArray32Local::from_string(&hex_string).unwrap();
        assert_eq!(array, from_hex);
    }

    #[tokio::test]
    async fn test_wallet_extension() {
        let mut wallet = WalletExtension::new().unwrap();
        wallet.balance = 1000000;

        let sender = ByteArray32Local([1u8; 32]);
        let recipient = ByteArray32Local([2u8; 32]);
        let config = ChannelConfig::new();

        let channel_id = wallet.create_channel(
            sender,
            recipient,
            1000,
            &config
        ).await.unwrap();

        assert!(wallet.channels.contains_key(&channel_id));
        assert_eq!(wallet.total_locked_balance, 1000);
        assert_eq!(wallet.balance, 999000);
    }
}