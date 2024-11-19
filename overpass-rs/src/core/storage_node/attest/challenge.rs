    use crate::core::storage_node::storage_node_contract::NetworkConfig;
use crate::core::storage_node::storage_node_contract::EpidemicProtocolConfig;
use crate::core::storage_node::epidemic::sync::SyncConfig;
use crate::core::storage_node::storage_node_contract::SyncConfig;
use crate::core::storage_node::storage_node_contract::EpidemicProtocolConfig;
use frame_support::{Deserialize, Serialize};
    use std::sync::{Arc, RwLock};
    use std::time::SystemTime;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use wasm_bindgen::prelude::*;
    use web_sys::{console, window};
    use wasm_bindgen_futures::JsFuture;
    use js_sys::Promise;

    use crate::core::error::{SystemError, SystemErrorType};
    use crate::core::storage_node::storage_node_contract::{StorageNode, StorageNodeConfig};
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

    // Interface for challenges
    pub trait ChallengeInterface {
        fn get_challenge_threshold(&self) -> u64;
        fn get_challenges(&self) -> Result<Vec<ChallengeRecord>, SystemError>;
        fn get_peers(&self) -> Result<Vec<[u8; 32]>, SystemError>;
        fn get_active_challenges(&self) -> Result<Vec<[u8; 32]>, SystemError>;
        fn add_active_challenge(&mut self, node_id: [u8; 32]) -> Result<(), SystemError>;
        fn create_proof_request(&mut self, node_id: [u8; 32], data_id: [u8; 32]) 
            -> Result<Vec<u8>, SystemError>;
        fn send_proof_request(&mut self, node_id: [u8; 32], request: Vec<u8>) 
            -> Result<(), SystemError>;
        fn get_challenge_timeout(&self) -> u64;
        fn set_challenge_start(&mut self, node_id: [u8; 32], start_time: SystemTime) 
            -> Result<(), SystemError>;
        fn store_challenge_details(&mut self, details: ChallengeDetails) -> Result<(), SystemError>;
    }

    // Main ChallengeManager implementation
    pub struct ChallengeManager {
        storage_node: Arc<StorageNode>,
        challenge_fee: u64,
        challenge_threshold: u64,
        challenge_interval: u64,
        active_challenges: Arc<RwLock<Vec<[u8; 32]>>>,
        challenge_details: Arc<RwLock<HashMap<[u8; 32], ChallengeDetails>>>,
    }
    impl ChallengeManager {
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
            let challenges = ChallengeInterface::get_challenges(self.storage_node.as_ref())
                .map_err(ChallengeManagerError::from)?;

            let challenge_count = challenges.len();

            if challenge_count >= self.challenge_threshold as usize {
                self.challenge_nodes(challenges).await?;
            }

            Ok(())
        }
        async fn challenge_nodes(
            &self,
            challenges: Vec<ChallengeRecord>
        ) -> Result<(), ChallengeManagerError> {
            for challenge in challenges {
                let node_id = challenge.node_id;

                // Get current peers and active challenges
                let peers = self.storage_node.as_ref().get_peers()
                    .map_err(ChallengeManagerError::from)?;

                if !peers.contains(&node_id) {
                    continue;
                }

                let active_challenges = {
                    let guard = self.active_challenges.read()
                        .map_err(|_| ChallengeManagerError::LockError("Failed to read active challenges".into()))?;
                    guard.clone()
                };

                if active_challenges.contains(&node_id) {
                    continue;
                }

                // Add to active challenges
                {
                    let mut guard = self.active_challenges.write()
                        .map_err(|_| ChallengeManagerError::LockError("Failed to write active challenges".into()))?;
                    guard.push(node_id);
                }

                // Create and send proof request
                let request = ChallengeInterface::create_proof_request(
                    &mut self.storage_node.as_ref().clone(),
                    node_id,
                    *self.challenge_fee
                )?;

                ChallengeInterface::send_proof_request(
                    &mut self.storage_node.as_ref(),
                    node_id,
                    request
                )?;

                // Store challenge details
                let challenge_details = ChallengeDetails {
                    node_id,
                    data_id: challenge.data_id,
                    start_time: SystemTime::now(),
                    timeout: self.challenge_interval,
                    status: ChallengeStatus::Pending,
                };

                self.store_challenge_details(challenge_details)?;
            }
            Ok(())
        }

        
     
        pub fn store_challenge_details(
            &self,
            details: ChallengeDetails
        ) -> Result<(), ChallengeManagerError> {
            let mut guard = self.challenge_details.write()
                .map_err(|_| ChallengeManagerError::LockError("Failed to write challenge details".into()))?;
            
            guard.insert(details.node_id, details);
            Ok(())
        }

        pub fn get_active_challenges(&self) -> Result<Vec<[u8; 32]>, ChallengeManagerError> {
            let guard = self.active_challenges.read()
                .map_err(|_| ChallengeManagerError::LockError("Failed to read active challenges".into()))?;
            Ok(guard.clone())
        }

        pub fn get_challenge_details(
            &self,
            node_id: &[u8; 32]
        ) -> Result<Option<ChallengeDetails>, ChallengeManagerError> {
            let guard = self.challenge_details.read()
                .map_err(|_| ChallengeManagerError::LockError("Failed to read challenge details".into()))?;
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

    // WASM bindings
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
            let node_id = nod.try_into().unwrap()e_id.try_into()
                .map_err(|_| JsValue::from_str("Invalid node ID length"))?;
            
            let storage_node = Arc::new(StorageNode::new(
                node_id,
                challenge_fee,
                StorageNodeConfig::new(
                    (BatteryConfig::defau).into()lt(),
                    SyncConfig::default(),
                    EpidemicProtocolConfig::default(),
                    NetworkConfig::default(),
                    node_id,
                    challenge_fee as i32,
                    HashSet::new()
                ),
                HashSet::new()
            ).await.map_err(|e| JsValue::from_str(&format!("{:?}", e)))?);
            
            let manager = ChallengeManager::create(
                storage_node,
                challenge_fee,
                challenge_threshold,
                challenge_interval,
            ).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            
            Ok(ChallengeManagerWrapper(manager))
        }
        pub async fn start_challenge(&self) -> Result<(), JsValue> {
            self.0.start_challenge()
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
                node_id.to_vec().to_vec(),
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