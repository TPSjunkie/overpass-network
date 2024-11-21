use std::sync::Arc;
use std::time::SystemTime;
use std::collections::{HashMap, HashSet};
use parking_lot::RwLock;
use wasm_bindgen::prelude::*;
use web_sys::{console, window};
use wasm_bindgen_futures::JsFuture;
use js_sys::Promise;
use serde::{Serialize, Deserialize};

use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::battery::charging::BatteryConfig;
use crate::core::storage_node::epidemic::sync::SyncConfig;
use crate::core::storage_node::storage_node_contract::*;

// Constants
const MIN_CHALLENGE_THRESHOLD: u64 = 1;
const MIN_CHALLENGE_INTERVAL: u64 = 1000; // 1 second

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeRecord {
    pub node_id: [u8; 32],
    pub data_id: [u8; 32],
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeDetails {
    pub node_id: [u8; 32],
    pub data_id: [u8; 32],
    pub start_time: SystemTime,
    pub timeout: u64,
    pub status: ChallengeStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChallengeStatus {
    Pending,
    Completed,
    Failed, 
    TimedOut,
}

#[derive(Debug)]
pub enum ChallengeManagerError {
    InvalidThreshold(String),
    InvalidInterval(String),
    StorageError(SystemError),
    NetworkError(String),
    LockError(String),
}

impl From<SystemError> for ChallengeManagerError {
    fn from(error: SystemError) -> Self {
        match error.error_type {
            SystemErrorType::NetworkError => 
                ChallengeManagerError::NetworkError(error.message),
            _ => ChallengeManagerError::StorageError(error),
        }
    }
}

pub struct ChallengeManager<StorageNode> {
    storage_node: Arc<StorageNode>,
    challenge_fee: u64,
    challenge_threshold: u64,
    challenge_interval: u64,
    active_challenges: Arc<RwLock<Vec<[u8; 32]>>>,
    challenge_details: Arc<RwLock<HashMap<[u8; 32], ChallengeDetails>>>,
}impl ChallengeManager {
    pub fn create(
        storage_node: Arc<StorageNode>,
        challenge_fee: u64,
        challenge_threshold: u64,
        challenge_interval: u64,
    ) -> Result<Self, ChallengeManagerError> {
        if challenge_threshold < MIN_CHALLENGE_THRESHOLD {
            return Err(ChallengeManagerError::InvalidThreshold(
                format!("Challenge threshold must be at least {}", MIN_CHALLENGE_THRESHOLD)
            ));
        }

        if challenge_interval < MIN_CHALLENGE_INTERVAL {
            return Err(ChallengeManagerError::InvalidInterval(
                format!("Challenge interval must be at least {}ms", MIN_CHALLENGE_INTERVAL)
            ));
        }

        Ok(Self {
            storage_node,
            challenge_fee,
            challenge_threshold,
            challenge_interval,
            active_challenges: Arc::new(RwLock::new(Vec::new())),
            challenge_details: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start_challenge(&self) -> Result<(), ChallengeManagerError> {
        let manager = self.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let window = window().expect("no global window exists");
            let interval = manager.challenge_interval;

            loop {
                let promise = Promise::new(&mut |resolve, _| {
                    window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            interval as i32,
                        )
                        .expect("failed to set timeout");
                });

                JsFuture::from(promise).await.expect("timeout failed");

                if let Err(e) = manager.check_challenge().await {
                    console::error_1(&format!("Challenge error: {:?}", e).into());
                }
            }
        });

        Ok(())
    }

    async fn check_challenge(&self) -> Result<(), ChallengeManagerError> {
        let current_time = SystemTime::now();
        let mut challenges_to_process = Vec::new();

        // Get challenges that need processing
        {
            let details = self.challenge_details.read();
            for (node_id, challenge) in details.iter() {
                if challenge.status == ChallengeStatus::Pending {
                    if let Ok(elapsed) = current_time.duration_since(challenge.start_time) {
                        if elapsed.as_millis() as u64 >= challenge.timeout {
                            challenges_to_process.push((*node_id, challenge.data_id));
                        }
                    }
                }
            }
        }

        // Process challenges
        for (node_id, data_id) in challenges_to_process {
            self.process_challenge(node_id, data_id).await?;
        }

        Ok(())
    }

    async fn process_challenge(
        &self,
        node_id: [u8; 32],
        data_id: [u8; 32]
    ) -> Result<(), ChallengeManagerError> {
        // Update challenge status
        {
            let mut details = self.challenge_details.write();
            if let Some(challenge) = details.get_mut(&node_id) {
                challenge.status = ChallengeStatus::Completed;
            }
        }

        // Remove from active challenges
        {
            let mut active = self.active_challenges.write();
            if let Some(pos) = active.iter().position(|&id| id == node_id) {
                active.remove(pos);
            }
        }

        Ok(())
    }

    pub fn store_challenge_details(
        &self,
        details: ChallengeDetails
    ) -> Result<(), ChallengeManagerError> {
        let mut guard = self.challenge_details.write();
        guard.insert(details.node_id, details);
        Ok(())
    }

    pub fn get_active_challenges(&self) -> Result<Vec<[u8; 32]>, ChallengeManagerError> {
        let guard = self.active_challenges.read();
        Ok(guard.clone())
    }

    pub fn get_challenge_details(
        &self,
        node_id: &[u8; 32]
    ) -> Result<Option<ChallengeDetails>, ChallengeManagerError> {
        let guard = self.challenge_details.read();
        Ok(guard.get(node_id).cloned())
    }
}

impl Clone for ChallengeManager {
    fn clone(&self) -> Self {
        Self {
            storage_node: Arc::clone(&self.storage_node),
            challenge_fee: self.challenge_fee,
            challenge_threshold: self.challenge_threshold,
            challenge_interval: self.challenge_interval,
            active_challenges: Arc::clone(&self.active_challenges),
            challenge_details: Arc::clone(&self.challenge_details),
        }
    }
}

#[wasm_bindgen]
pub struct ChallengeManagerWrapper(ChallengeManager);

#[wasm_bindgen]
impl ChallengeManagerWrapper {
    #[wasm_bindgen(constructor)]
    pub async fn new(
        node_id: Vec<u8>, 
        challenge_fee: u64,
        challenge_threshold: u64,
        challenge_interval: u64,
    ) -> Result<ChallengeManagerWrapper, JsValue> {
        let node_id: [u8; 32] = node_id.try_into()
            .map_err(|_| JsValue::from_str("Invalid node ID length"))?;

        let storage_node = Arc::new(crate::core::storage_node::StorageNode::new(
            node_id,
            challenge_fee.try_into().unwrap(),
            crate::core::storage_node::StorageNodeConfig::new(
                crate::core::storage_node::BatteryConfig::default(),
                crate::core::storage_node::SyncConfig::default(),
                crate::core::storage_node::EpidemicProtocolConfig::default(), 
                crate::core::storage_node::NetworkConfig::default(),
                node_id,
                challenge_fee as i64,
                HashSet::new()
            )
        ).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?);

        let manager = ChallengeManager::create(
            storage_node,
            challenge_fee,
            challenge_threshold,
            challenge_interval,
        ).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        Ok(ChallengeManagerWrapper(manager))
    }

    pub async fn start_challenge(&self) -> Result<(), JsValue> {        self.0.start_challenge()
            .await
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    pub fn get_active_challenges(&self) -> Result<JsValue, JsValue> {
        let challenges = self.0.get_active_challenges()
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        serde_wasm_bindgen::to_value(&challenges)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {:?}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn create_test_node_id() -> [u8; 32] {
        [0u8; 32]
    }

    #[wasm_bindgen_test]
    async fn test_challenge_manager_creation() {
        let node_id = create_test_node_id();
        let challenge_fee = 100;
        let challenge_threshold = 1000;
        let challenge_interval = 5000;

        let wrapper = ChallengeManagerWrapper::new(
            node_id.to_vec(),
            challenge_fee,
            challenge_threshold,
            challenge_interval
        );
        assert!(wrapper.await.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_challenge_lifecycle() {
        let node_id = create_test_node_id();
        let wrapper = ChallengeManagerWrapper::new(
            node_id.to_vec(),
            100,
            1000,
            5000
        ).await.unwrap();
        
        let result = wrapper.start_challenge().await;
        assert!(result.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_active_challenges() {
        let node_id = create_test_node_id();
        let wrapper = ChallengeManagerWrapper::new(
            node_id.to_vec(),
            100,
            1000,
            5000
        ).await.unwrap();

        let challenges = wrapper.get_active_challenges();
        assert!(challenges.is_ok());
    }
}