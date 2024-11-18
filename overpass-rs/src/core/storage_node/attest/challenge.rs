use crate::core::storage_node::attest::response::Duration;
use crate::core::error::errors::SystemError;
use crate::core::storage_node::storage_node_contract::StorageNode;
use js_sys::Promise;
use crate::core::types::boc::BOC;
use crate::core::zkps::proof::ZkProof;
use crate::core::types::ovp_ops::*;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::SystemTime;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{console, window};

#[derive(Clone)]
pub struct ChallengeRecord {
    pub node_id: [u8; 32],
    pub data_id: [u8; 32],
    pub timestamp: SystemTime,
}

#[derive(Clone)]
pub struct ChallengeDetails {
    pub node_id: [u8; 32],
    pub data_id: [u8; 32],
    pub start_time: SystemTime,
    pub timeout: u64,
    pub status: ChallengeStatus,
}

#[derive(Clone, PartialEq)]
pub enum ChallengeStatus {
    Pending,
    Completed,
    Failed,
    TimedOut,
}

pub trait ChallengeInterface {
    fn get_challenge_threshold(&self) -> u64;
    fn get_challenges(&self) -> Vec<ChallengeRecord>;
    fn get_peers(&self) -> Vec<[u8; 32]>;
    fn get_active_challenges(&self) -> Vec<[u8; 32]>;
    fn add_active_challenge(&mut self, node_id: [u8; 32]);
    fn create_proof_request(
        &mut self,
        node_id: [u8; 32],
        data_id: [u8; 32],
    ) -> Result<Vec<u8>, SystemError>;
    fn send_proof_request(
        &mut self,
        node_id: [u8; 32],
        request: Vec<u8>,
    ) -> Result<(), SystemError>;
    fn get_challenge_timeout(&self) -> u64;
    fn set_challenge_start(&mut self, node_id: [u8; 32], start_time: SystemTime);
    fn store_challenge_details(&mut self, details: ChallengeDetails) -> Result<(), SystemError>;
}

pub struct ChallengeManager {
    storage_node: Arc<RwLock<StorageNode>>,
    challenge_fee: u64,
    challenge_threshold: u64,
    challenge_interval: u64,
}

trait NewTrait<T> where T: ChallengeInterface + Sized {
    fn new(
        storage_node: Arc<RwLock<T>>,
        challenge_fee: u64,
        challenge_threshold: u64,
        challenge_interval: u64,
    ) -> Self;

    fn start_challenge(&self);

    async fn check_challenge(
        storage_node: &Arc<RwLock<T>>,
    ) -> Result<(), SystemError>;

    async fn challenge_nodes(
        challenges: Vec<ChallengeRecord>,
        storage_node: &Arc<RwLock<T>>,
    ) -> Result<(), SystemError>;
}

impl ChallengeManager {
    pub fn create(
        storage_node: Arc<RwLock<StorageNode>>,
        challenge_fee: u64,
        challenge_threshold: u64,
        challenge_interval: u64,
    ) -> Self {
        Self {
            storage_node,
            challenge_fee,
            challenge_threshold,
            challenge_interval,
        }
    }

    async fn check_challenge_impl(
        storage_node: &Arc<RwLock<StorageNode>>,
    ) -> Result<(), SystemError> {
        let challenges = {
            let node = storage_node.read().map_err(|_| SystemError::LockError)?;
            node.get_challenges()
        };
        Self::challenge_nodes(challenges, storage_node).await
    }
  async fn challenge_nodes_impl(
        challenges: Vec<ChallengeRecord>,
        storage_node: &Arc<RwLock<StorageNode>>,
    ) -> Result<(), SystemError> {
        for challenge in challenges {
            let node_id = challenge.node_id;
            let data_id = challenge.data_id;
            
            let mut node = storage_node.write().map_err(|_| SystemError::LockError)?;
            
            // Create and send proof request
            let request = node.create_proof_request(node_id, data_id)?;
            node.send_proof_request(node_id, request)?;
            
            // Mark challenge as started
            node.add_active_challenge(node_id);
            node.set_challenge_start(node_id, SystemTime::now());
        }
        Ok(())
    }

    pub fn start_challenge(&self) {
        let storage_node = self.storage_node.clone();
        let interval = self.challenge_interval;
        let manager = self.clone();
        spawn_local(async move {
            loop {
                let window = web_sys::window().expect("no global window exists");
                let promise = Promise::new(&mut |resolve, _| {
                    window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            interval as i32,
                        )
                        .unwrap();
                });
    
                JsFuture::from(promise).await.unwrap();
                if let Err(e) = manager.check_challenge(&storage_node).await {
                    console::error_1(&format!("Challenge error: {:?}", e).into());
                }
            }
        });
    }
    fn get_challenges(&self) -> Vec<ChallengeRecord> {
        self.challenges.clone()
    }

    fn get_peers(&self) -> Vec<[u8; 32]> {
        self.peers.clone()
    }

    fn get_active_challenges(&self) -> Vec<[u8; 32]> {
        self.active_challenges.clone()
    }

    fn add_active_challenge(&mut self, node_id: [u8; 32]) {
        self.active_challenges.push(node_id);
    }

    fn create_proof_request(
        &mut self,
        node_id: [u8; 32],
        data_id: [u8; 32],
    ) -> Result<Vec<u8>, SystemError> {
        let proof_request = ProofRequest {
            node_id,
            data_id,
            timestamp: SystemTime::now(),
            nonce: rand::random::<[u8; 32]>(),
        };
        Ok(bincode::serialize(&proof_request).map_err(|_| SystemError::SerializationError)?)
    }

    fn send_proof_request(
        &mut self,
        node_id: [u8; 32],
        request: Vec<u8>,
    ) -> Result<(), SystemError> {
        let peer = self.network.get_peer(node_id)
            .ok_or(SystemError::PeerNotFound)?;
        peer.send_message(MessageType::ProofRequest, request)
            .map_err(|_| SystemError::NetworkError)
    }

    fn get_challenge_timeout(&self) -> u64 {
        self.challenge_timeout
    }

    fn set_challenge_start(&mut self, node_id: [u8; 32], start_time: SystemTime) {
        self.challenge_starts.insert(node_id, start_time);
    }

    fn store_challenge_details(&mut self, details: ChallengeDetails) -> Result<(), SystemError> {
        self.challenge_details.insert(details.node_id, details);
        self.persist_challenge_details()
            .map_err(|_| SystemError::StorageError)
    }
}

impl ChallengeManager {
    fn new(
        storage_node: Arc<RwLock<StorageNode>>,
        challenge_fee: u64,
        challenge_threshold: u64,
        challenge_interval: u64,
    ) -> Self {
        Self {
            storage_node,
            challenge_fee,
            challenge_threshold,
            challenge_interval,
        }
    }
    
    fn start_challenge(&self) {
        let interval = self.challenge_interval;
        let manager = self.storage_node.clone();
    
        let challenge_task = async move {
            loop {
                let window = window().unwrap();
                let promise = Promise::new(&mut |resolve, _| {
                    window
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            interval as i32,
                        )
                        .unwrap();
                });
    
                JsFuture::from(promise).await.unwrap();
                if let Err(e) = Self::check_challenge(&manager).await {
                    console::error_1(&format!("Challenge error: {:?}", e).into());
                }
            }
        };
    
        spawn_local(challenge_task);
    }
    
    async fn check_challenge(
        storage_node: &Arc<RwLock<StorageNode>>,
    ) -> Result<(), SystemError> {
        let challenge_threshold = {
            let node = storage_node.read().unwrap();
            node.get_challenge_threshold()
        };
        let challenges: Vec<ChallengeRecord> = {
            let node = storage_node.read().unwrap();
            node.get_challenges()
        };
        let challenge_count = challenges.len();
    
        if challenge_count >= challenge_threshold as usize {
            Self::challenge_nodes(challenges, storage_node).await?;
        }
    
        Ok(())
    }
    
    async fn challenge_nodes(
        challenges: Vec<ChallengeRecord>,
        storage_node: &Arc<RwLock<StorageNode>>,
    ) -> Result<(), SystemError> {
        for challenge in challenges {
            let node_id = challenge.node_id;
            let node = storage_node.read().unwrap();
    
            let peers = node.get_peers();
            if !peers.contains(&node_id) {
                continue;
            }
    
            let active_challenges = node.get_active_challenges();
            if active_challenges.contains(&node_id) {
                continue;
            }
            drop(node);
    
            let mut node = storage_node.write().unwrap();
            node.add_active_challenge(node_id);
    
            let proof_request = node.create_proof_request(node_id, challenge.data_id)?;
            node.send_proof_request(node_id, proof_request)?;
    
            let challenge_timeout = node.get_challenge_timeout();
            let challenge_start = SystemTime::now();
            node.set_challenge_start(node_id, challenge_start);
    
            let challenge_details = ChallengeDetails {
                node_id,
                data_id: challenge.data_id,
                start_time: challenge_start,
                timeout: challenge_timeout,
                status: ChallengeStatus::Pending,
            };
            node.store_challenge_details(challenge_details)?;
        }
    
        Ok(())
    }
}
