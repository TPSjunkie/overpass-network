use crate::core::boc::{StateBOC, DAGBOC};
use crate::error::StorageError;
use serde::{Serialize, Deserialize};
use web_sys::{IdbDatabase, IdbTransaction, IdbObjectStore};
use wasm_bindgen::JsValue;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct StoredState {
    pub version: u64,
    pub root: [u8; 32],
    pub channels: HashMap<[u8; 32], ChannelState>,
    pub contracts: HashMap<[u8; 32], ContractState>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelState {
    pub state_boc: Vec<u8>,    // Serialized StateBOC
    pub contract_boc: Vec<u8>, // Serialized DAGBOC
    pub nonce: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContractState {
    pub code: Vec<u8>,
    pub state: Vec<u8>,
}

pub struct StateStorage {
    db: IdbDatabase,
}

impl StateStorage {
    pub async fn new() -> Result<Self, StorageError> {
        let window = web_sys::window().unwrap();
        let indexeddb = window.indexed_db().unwrap().unwrap();
        
        let db_open = indexeddb.open_with_u32("ovp_client_storage", 1)?;
        
        // Set up database schema
        db_open.set_onupgradeneeded(Some(&js_sys::Function::new_with_args(
            "event",
            "
            const db = event.target.result;
            
            // Create stores
            db.createObjectStore('states', {keyPath: 'version'});
            db.createObjectStore('channels', {keyPath: 'id'});
            db.createObjectStore('contracts', {keyPath: 'id'});
            "
        )));
        
        let db = wasm_bindgen_futures::JsFuture::from(db_open)
            .await?
            .dyn_into::<IdbDatabase>()?;
            
        Ok(Self { db })
    }

    pub async fn store_state(&mut self, state: StoredState) -> Result<(), StorageError> {
        let transaction = self.db.transaction_with_str_and_mode(
            "states",
            "readwrite"
        )?;
        
        let store = transaction.object_store("states")?;
        
        // Store main state
        let state_value = JsValue::from_serde(&state)?;
        store.put(&state_value)?;
        
        // Store channels
        let channel_store = transaction.object_store("channels")?;
        for (id, channel_state) in state.channels {
            let channel_value = JsValue::from_serde(&channel_state)?;
            channel_store.put_with_key(&channel_value, &JsValue::from_serde(&id)?)?;
        }
        
        // Store contracts
        let contract_store = transaction.object_store("contracts")?;
        for (id, contract_state) in state.contracts {
            let contract_value = JsValue::from_serde(&contract_state)?;
            contract_store.put_with_key(&contract_value, &JsValue::from_serde(&id)?)?;
        }
        
        Ok(())
    }

    pub async fn load_latest_state(&self) -> Result<Option<StoredState>, StorageError> {
        let transaction = self.db.transaction_with_str("states")?;
        let store = transaction.object_store("states")?;
        
        // Get latest state by version
        let request = store
            .get_all()?;
            
        let result = wasm_bindgen_futures::JsFuture::from(request)
            .await?;
            
        if result.is_undefined() {
            return Ok(None);
        }
        
        let states: Vec<StoredState> = result.into_serde()?;
        Ok(states.into_iter().max_by_key(|s| s.version))
    }
}
