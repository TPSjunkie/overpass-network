use crate::core::error::errors::SystemError;
use crate::core::storage_node::storage_node_contract::StorageNode;
use js_sys::Promise;
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

pub struct ChallengeManager<RootTree, IntermediateTreeManager> {
    storage_node: Arc<RwLock<StorageNode<RootTree, IntermediateTreeManager>>>,
    challenge_fee: u64,
    challenge_threshold: u64,
    challenge_interval: u64,
}

impl<RootTree, IntermediateTreeManager> ChallengeManager<RootTree, IntermediateTreeManager>
where
    StorageNode<RootTree, IntermediateTreeManager>: ChallengeInterface,
{
    pub fn new(
        storage_node: Arc<RwLock<StorageNode<RootTree, IntermediateTreeManager>>>,
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

    pub fn start_challenge(&self) {
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
        storage_node: &Arc<RwLock<StorageNode<RootTree, IntermediateTreeManager>>>,
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
        storage_node: &Arc<RwLock<StorageNode<RootTree, IntermediateTreeManager>>>,
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
