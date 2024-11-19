use crate::core::storage_node::battery::charging::BatteryConfig;
use crate::core::storage_node::storage_node_contract::BatteryConfig;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::collections::HashSet;
use frame_support::traits::IsType;
use plonky2::plonk::proof::Proof;
use wasm_bindgen::prelude::*;
use web_sys::console;
use std::time::Duration as StdDuration;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

// Core imports
use crate::core::{error::{SystemError, SystemErrorType}, storage_node::storage_node_contract::EpidemicProtocolConfig};
use crate::core::storage_node::storage_node_contract::{StorageNode, StorageNodeConfig};
use crate::core::zkps::proof::ZkProof;

// Constants
const MAX_VERIFICATION_ATTEMPTS: u32 = 3;
const DEFAULT_BACKOFF_MS: u64 = 1000;
const MAX_RESPONSE_SIZE: usize = 1024 * 1024; // 1MB
const MIN_RESPONSE_THRESHOLD: u64 = 1;
const MIN_RESPONSE_INTERVAL: u64 = 1000; // 1 second

// Error types
#[derive(Debug)]
pub enum ResponseManagerError {
    InvalidThreshold(String),
    InvalidInterval(String),
    VerificationInProgress,
    StorageError(SystemError),
    InvalidResponse(String),
    ProofVerificationFailed,
}

impl From<SystemError> for ResponseManagerError {
    fn from(error: SystemError) -> Self {
        ResponseManagerError::StorageError(error)
    }
}

// Duration trait implementation
pub trait Duration {
    fn as_millis(&self) -> u64;
}

impl Duration for StdDuration {
    fn as_millis(&self) -> u64 {
        self.as_millis() as u64
    }
}

// Metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationMetrics {
    pub total_verifications: u64,
    pub successful_verifications: u64,
    pub failed_verifications: u64,
    pub average_verification_time_ms: f64,
    pub last_verification_time: Option<f64>,
    pub response_threshold: u64,
    pub response_interval: u64,
}

impl Default for VerificationMetrics {
    fn default() -> Self {
        Self {
            total_verifications: 0,
            successful_verifications: 0,
            failed_verifications: 0,
            average_verification_time_ms: 0.0,
            last_verification_time: None,
            response_threshold: MIN_RESPONSE_THRESHOLD,
            response_interval: MIN_RESPONSE_INTERVAL,
        }
    }
}

// Main ResponseManager implementation
pub struct ResponseManager {
    storage_node: Arc<StorageNode>,
    response_threshold: u64,
    response_interval: u64,
    is_verifying: Arc<AtomicBool>,
    metrics: Arc<RwLock<VerificationMetrics>>,
    last_verification_time: Arc<RwLock<Option<f64>>>,
}<D>

impl ResponseManager {
    pub fn create(
        node_id: [u8; 32],
        response_threshold: u64,
        response_interval: u64
    ) -> Result<Self, ResponseManagerError> {
        // Validate configuration
        if response_threshold < MIN_RESPONSE_THRESHOLD {
            return Err(ResponseManagerError::InvalidThreshold(
                format!("Response threshold must be at least {}", MIN_RESPONSE_THRESHOLD)
            ));
        }

        if response_interval < MIN_RESPONSE_INTERVAL {
            return Err(ResponseManagerError::InvalidInterval(
                format!("Response interval must be at least {}ms", MIN_RESPONSE_INTERVAL)
            ));
        }
        let storage_node = StorageNode::new(
            node_id,
            0,
            StorageNodeConfig {
                battery_config: BatteryConfig::default(),
                sync_config: SyncConfig::default(),
                epidemic_protocol_config: EpidemicProtocolConfig::default(),
                network_config: NetworkConfig::default(),
                node_id,
                fee: 0,
                whitelist: HashSet::new(),
            }
        );        // Create and return the ResponseManager instance
        Ok(ResponseManager {
            storage_node: Arc::new(storage_node),
            response_threshold,            response_interval,
            is_verifying: Arc::new(AtomicBool::new(false)),
            metrics: Arc::new(RwLock::new(VerificationMetrics {
                response_threshold,
                response_interval,
                ..VerificationMetrics::default()
            })),
            last_verification_time: Arc::new(RwLock::new(None)),
        })
    }

    pub fn start_verification(&self) -> Result<(), ResponseManagerError> {        if self.is_verifying.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst,
        ).is_err() {
            return Err(ResponseManagerError::VerificationInProgress);
        }

        let manager = self.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let window = web_sys::window().expect("no global window exists");
            let performance = window
                .performance()
                .expect("performance should be available");

            while manager.is_verifying.load(Ordering::SeqCst) {
                let start_time = performance.now();
                
                match manager.check_response_verification().await {
                    Ok(_) => {
                        let mut metrics = manager.metrics.write();
                        metrics.total_verifications += 1;
                        metrics.successful_verifications += 1;
                        
                        let verification_time = performance.now() - start_time;
                        metrics.average_verification_time_ms = 
                            (metrics.average_verification_time_ms * (metrics.total_verifications - 1) as f64 
                            + verification_time) / metrics.total_verifications as f64;
                    },
                    Err(e) => {
                        console::error_1(&format!("Response verification error: {:?}", e).into());
                        let mut metrics = manager.metrics.write();
                        metrics.total_verifications += 1;
                        metrics.failed_verifications += 1;
                    }
                }

                *manager.last_verification_time.write() = Some(performance.now());

                let elapsed = performance.now() - start_time;
                if elapsed < manager.response_interval as f64 {
                    let delay = manager.response_interval as f64 - elapsed;
                    
                    if manager.delay_with_cancellation(delay).await.is_err() {
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    async fn delay_with_cancellation(&self, delay_ms: f64) -> Result<(), ()> {
        let window = web_sys::window().expect("no global window exists");
        
        let delay_future = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
            &mut |resolve, _| {
                window
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &resolve,
                        delay_ms as i32,
                    )
                    .expect("failed to set timeout");
            },
        ));

        if !self.is_verifying.load(Ordering::SeqCst) {
            return Err(());
        }

        delay_future.await.map_err(|_| ())?;
        Ok(())
    }

    pub fn stop_verification(&self) {
        self.is_verifying.store(false, Ordering::SeqCst);
    }

    async fn check_response_verification(&self) -> Result<(), SystemError> {
        let boc = self.storage_node.retrieve_boc(&[0u8; 32]).await?;

        if boc.cells.len() > MAX_RESPONSE_SIZE {
            return Err(SystemError::new(
                SystemErrorType::InvalidInput,
                "Response size exceeds maximum allowed".into()
            ));
        }
    
        let response_count = boc.cells.len();

        if response_count >= self.response_threshold as usize {
            let responses: Vec<[u8; 32]> = boc
                .cells
                .into_iter()
                .filter_map(|item| {
                    if item.data.len() == 32 {
                        let mut array = [0u8; 32];
                        array.copy_from_slice(&item.data);
                        Some(array)
                    } else {
                        None
                    }
                })
                .collect();
    
            self.verify_responses(responses).await?;
        }

        Ok(())
    }

    async fn verify_responses(&self, responses: Vec<[u8; 32]>) -> Result<(), SystemError> {
        for proof in responses {
            let mut attempts = 0;
            while attempts < MAX_VERIFICATION_ATTEMPTS {
                match self.verify_single_response(&proof).await {
                    Ok(_) => break,
                    Err(e) => {
                        attempts += 1;
                        if attempts == MAX_VERIFICATION_ATTEMPTS {
                            console::error_1(&format!(
                                "Failed to verify proof after {} attempts: {:?}", 
                                MAX_VERIFICATION_ATTEMPTS, 
                                e
                            ).into());
                            return Err(e);
                        }
                        
                        let delay = DEFAULT_BACKOFF_MS * 2u64.pow(attempts);
                        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                            &mut |resolve, _| {
                                web_sys::window()
                                    .unwrap()
                                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                                        &resolve,
                                        delay as i32,
                                    )
                                    .expect("failed to set timeout");
                            },
                        ))
                        .await
                        .expect("timeout should complete");
                    }
                }
            }
        }

        Ok(())
    }
    async fn verify_single_response(&self, proof: &[u8; 32]) -> Result<(), SystemError> {
        let zk_proof = self.storage_node.retrieve_proof(proof).await?;
        let mut proof_bytes = zk_proof.proof_data{ . }to_vec();
        proof_bytes.extend_from_slice(&zk_proof.proof_data);
        let proof = Proof::<ark_bn254::Bn254, C, D>::read(&proof_bytes[..]).map_err(|_| SystemError::new(
            SystemErrorType::InvalidProof,            "Invalid proof".into()
        ))?;
        match proof.verify() {  
            Ok(true) => Ok(()),
            Ok(false) => Err(SystemError::new(
                SystemErrorType::InvalidProof,
                "ZK proof verification failed".into()
            )),
            Err(e) => Err(SystemError::new(
                SystemErrorType::VerificationError,
                format!("Error during proof verification: {:?}", e)
            )),
        }
    }    pub fn get_metrics(&self) -> VerificationMetrics {
        (*self.metrics.read()).clone()
    }
    
    pub fn is_currently_verifying(&self) -> bool {
        self.is_verifying.load(Ordering::SeqCst)
    }
}

impl Clone for ResponseManager {
    fn clone(&self) -> Self {
        Self {
            storage_node: Arc::clone(&self.storage_node),
            response_threshold: self.response_threshold,
            response_interval: self.response_interval,
            is_verifying: Arc::clone(&self.is_verifying),
            metrics: Arc::clone(&self.metrics),
            last_verification_time: Arc::clone(&self.last_verification_time),
        }
    }
}

// WASM bindings
#[wasm_bindgen]
pub struct ResponseManagerWrapper(ResponseManager);

#[wasm_bindgen]
impl ResponseManagerWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(node_id: Vec<u8>, response_threshold: u64, response_interval: u64) -> Result<ResponseManagerWrapper, JsValue> {
        let node_id: [u8; 32] = node_id.try_into().map_err(|_| JsValue::from_str("Invalid node_id length"))?;
        ResponseManager::create(node_id, response_threshold, response_interval)
            .map(ResponseManagerWrapper)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    pub async fn start_verification(&self) -> Result<(), JsValue> {
        self.0.start_verification()
            
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    pub fn stop_verification(&self) {
        self.0.stop_verification();
    }

    pub fn get_verification_metrics(&self) -> Result<JsValue, JsValue> {
        let metrics = self.0.get_metrics();
        serde_wasm_bindgen::to_value(&metrics)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize metrics: {:?}", e)))
    }

    pub fn is_verifying(&self) -> bool {
        self.0.is_currently_verifying()
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
    async fn test_response_manager_creation() {
        let node_id = create_test_node_id();
        let response_threshold = 1000;
        let response_interval = 5000;

        let wrapper = ResponseManagerWrapper::new(node_id.to_vec(), response_threshold, response_interval);
        assert!(wrapper.is_ok());
    }

    #[wasm_bindgen_test]
    async fn test_verification_lifecycle() {
        let node_id = create_test_node_id();
        let wrapper = ResponseManagerWrapper::new(node_id.to_vec(), 1000, 5000).unwrap();
        
        assert!(!wrapper.is_verifying());
        wrapper.start_verification().await.unwrap();
        assert!(wrapper.is_verifying());
        wrapper.stop_verification();
        assert!(!wrapper.is_verifying());
    }

    #[wasm_bindgen_test]
    async fn test_metrics_tracking() {
        let node_id = create_test_node_id();
        let wrapper = ResponseManagerWrapper::new(node_id.to_vec(), 1000, 5000).unwrap();
        
        wrapper.start_verification().await.unwrap();
        
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    &resolve,
                    1000,
                )
                .unwrap();
        }))
        .await
        .unwrap();

        let metrics = wrapper.get_verification_metrics().unwrap();
        assert!(!metrics.is_null());
    }
}