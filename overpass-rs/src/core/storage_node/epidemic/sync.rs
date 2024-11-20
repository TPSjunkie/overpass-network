use crate::core::types::StorageOpCode::SyncState;
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use web_sys::{console, window};
use wasm_bindgen_futures::spawn_local;
use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::battery::charging::BatteryChargingSystem;
use crate::core::storage_node::epidemic::overlap::StorageOverlapManager;
use crate::core::types::boc::BOC;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStateEnum {
    Idle,
    Syncing,
    Verified,
    Failed,
    Suspended,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetrics {
    pub total_syncs: u64,
    pub successful_syncs: u64,
    pub failed_syncs: u64,
    pub average_sync_time: f64,
    pub active_syncs: u64,
    pub battery_rejections: u64,
    pub suspended_peers: usize,
    pub verified_peers: usize,
}
impl Default for SyncMetrics {
    fn default() -> Self {
        Self {
            total_syncs: 0,
            successful_syncs: 0,
            failed_syncs: 0,
            average_sync_time: 0.0,
            active_syncs: 0,
            battery_rejections: 0,
            suspended_peers: 0,
            verified_peers: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncConfig {
    pub min_battery_for_sync: u64,
    pub max_concurrent_syncs: usize,
    pub sync_interval: u64,
    pub sync_timeout: u64,
    pub max_sync_retries: u32,
    pub min_overlap_for_sync: f64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            min_battery_for_sync: 20,
            max_concurrent_syncs: 3,
            sync_interval: 5000,
            sync_timeout: 30000,
            max_sync_retries: 3,
            min_overlap_for_sync: 0.6,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalSyncState {
    pub hash: [u8; 32],
    pub data: BOC,
    pub timestamp: u64,
}

pub struct SynchronizationManager {
    battery_system: Arc<BatteryChargingSystem>,
    overlap_manager: Arc<StorageOverlapManager>,
    config: SyncConfig,
    peer_states: RwLock<HashMap<[u8; 32], crate::core::types::StorageOpCode>>,
    active_syncs: RwLock<HashSet<[u8; 32]>>,
    sync_attempts: RwLock<HashMap<[u8; 32], u32>>,
    last_sync: RwLock<HashMap<[u8; 32], u64>>,
    metrics: RwLock<SyncMetrics>,
    verified_states: RwLock<HashMap<[u8; 32], HashSet<[u8; 32]>>>,
    is_syncing: RwLock<bool>,
    states: RwLock<HashMap<[u8; 32], crate::core::types::StorageOpCode>>,
}

impl SynchronizationManager {
    pub fn new(
        battery_system: Arc<BatteryChargingSystem>,
        overlap_manager: Arc<StorageOverlapManager>,
        config: SyncConfig,
    ) -> Self {
        Self {
            battery_system,
            overlap_manager,
            config,
            peer_states: RwLock::new(HashMap::new()),
            active_syncs: RwLock::new(HashSet::new()),
            sync_attempts: RwLock::new(HashMap::new()),
            last_sync: RwLock::new(HashMap::new()),
            metrics: RwLock::new(SyncMetrics::default()),
            verified_states: RwLock::new(HashMap::new()),
            is_syncing: RwLock::new(false),
            states: RwLock::new(HashMap::new()),
        }
    }

    pub async fn start_sync(self: Arc<Self>) -> Result<(), SystemError> {
        let mut is_syncing = self.is_syncing.write();
        if *is_syncing {
            return Ok(());
        }
        *is_syncing = true;
        drop(is_syncing);

        let manager = self.clone();
        spawn_local(async move {
            while *manager.is_syncing.read() {
                manager.sync_cycle().await;
                wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
                    window().unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            manager.config.sync_interval as i32,
                        )
                        .unwrap();
                })).await.unwrap();
            }
        });

        Ok(())
    }

    pub fn stop_sync(&self) {
        *self.is_syncing.write() = false;
    }

    async fn sync_cycle(&self) {
        let battery_level = self.battery_system.get_charge_percentage();
        if battery_level < self.config.min_battery_for_sync as f64 {
            self.metrics.write().battery_rejections += 1;
            return;
        }

        let sync_peers = self.overlap_manager.get_synchronized_nodes();
        if sync_peers.is_empty() {
            return;
        }

        let peers_to_sync = self.select_sync_peers(&sync_peers).await;
        for peer in peers_to_sync {
            if let Err(e) = self.sync_with_peer(&peer).await {
                console::warn_1(&format!("Sync failed with peer {:?}: {:?}", peer, e).into());
            }
        }

        self.update_metrics();
    }

    async fn select_sync_peers(&self, available_peers: &HashSet<[u8; 32]>) -> Vec<[u8; 32]> {
        let mut selected_peers = Vec::new();
        let active_syncs = self.active_syncs.read();
        let peer_states = self.peer_states.read();
        let last_sync = self.last_sync.read();
        let now = window().unwrap().performance().unwrap().now() as u64;

        for peer in available_peers {
            if active_syncs.contains(peer) {
                continue;
            }

            if peer_states.get(peer) == Some(&crate::core::types::StorageOpCode::SyncState) {
                continue;
            }

            if let Some(last) = last_sync.get(peer) {
                if now - last < self.config.sync_interval {
                    continue;
                }
            }

            let overlap_score = self.overlap_manager.calculate_overlap_score(*peer, [0; 32]);
            if overlap_score < self.config.min_overlap_for_sync {
                continue;
            }

            selected_peers.push(*peer);
            if selected_peers.len() >= self.config.max_concurrent_syncs {
                break;
            }
        }

        selected_peers
    }

    async fn sync_with_peer(&self, peer: &[u8; 32]) -> Result<(), SystemError> {
        let mut active_syncs = self.active_syncs.write();
        if active_syncs.len() >= self.config.max_concurrent_syncs {
            return Err(SystemError::new(
                SystemErrorType::TooManySyncs,
                "Maximum concurrent syncs reached".to_string(),
            ));
        }
        active_syncs.insert(*peer);
        drop(active_syncs);

        self.peer_states.write().insert(*peer, crate::core::types::StorageOpCode::SyncState);
        let start_time = window().unwrap().performance().unwrap().now();

        let result = self.execute_sync(peer).await;

        match result {
            Ok(()) => {
                self.last_sync.write().insert(*peer, start_time as u64);
                self.peer_states.write().insert(*peer, crate::core::types::StorageOpCode::SyncState);
                Ok(())
            }
            Err(e) => {
                self.peer_states.write().insert(*peer, crate::core::types::StorageOpCode::SyncState);
                Err(e)
            }
        }
    }

    async fn execute_sync(&self, peer: &[u8; 32]) -> Result<(), SystemError> {
        let states_to_sync = self.get_unverified_states(peer).await?;
        if states_to_sync.is_empty() {
            return Ok(());
        }

        for state in states_to_sync {
            if let Err(e) = self.verify_state(&state).await {
                console::warn_1(&format!(
                    "State verification failed for peer {:?}, state {:?}: {:?}",
                    peer, state, e
                ).into());
                continue;
            }

            self.verified_states.write()
                .entry(*peer)
                .or_insert_with(HashSet::new)
                .insert(state);
        }

        Ok(())
    }
    async fn get_unverified_states(&self, peer: &[u8; 32]) -> Result<HashSet<[u8; 32]>, SystemError> {
        let verified = self.verified_states.read()
            .get(peer)
            .cloned()
            .unwrap_or_default();
        
        let mut unverified = HashSet::new();
        for (hash, _) in self.states.read().iter() {
            if !verified.contains(hash) {
                unverified.insert(*hash);
            }
        }
        
        Ok(unverified)
    }

    async fn verify_state(&self, state_hash: &[u8; 32]) -> Result<(), SystemError> {
        let states = self.states.read();
        let state = states.get(state_hash).ok_or_else(|| {
            SystemError::new(
                SystemErrorType::InvalidInput,
                "State not found".to_string(),
            )
        })?;

        // Verify state validity here
        // For now just checking existence
        Ok(())
    }

    fn update_metrics(&self) {
        let mut metrics = self.metrics.write();
        metrics.total_syncs = metrics.successful_syncs + metrics.failed_syncs;
    }

    pub fn get_metrics(&self) -> SyncMetrics {
        self.metrics.read().clone()
    }
    pub fn get_config(&self) -> SyncConfig {
        self.config.clone()
    }
}
    #[cfg(test)]
    mod tests {
        use crate::core::types::StorageOpCode;
        use super::*;
        use wasm_bindgen_test::*;
    
        wasm_bindgen_test_configure!(run_in_browser);
    
        async fn setup_sync_manager() -> SynchronizationManager {
            let battery_system = Arc::new(BatteryChargingSystem::new(Default::default()));
            let overlap_manager = Arc::new(StorageOverlapManager::new(0.8, 3));
            let config = SyncConfig::default();
            SynchronizationManager::new(battery_system, overlap_manager, config)
        }
    
        #[wasm_bindgen_test]
        async fn test_sync_lifecycle() {
            let manager = setup_sync_manager().await;
            let manager = Arc::new(manager);
            assert!(manager.clone().start_sync().await.is_ok());
            assert!(*manager.is_syncing.read());
            manager.stop_sync();
            assert!(!*manager.is_syncing.read());
        }
    
        #[wasm_bindgen_test]
        async fn test_battery_requirements() {
            let mut manager = setup_sync_manager().await;
            Arc::get_mut(&mut manager.battery_system).unwrap().consume_charge(90).await.unwrap();
            manager.sync_cycle().await;
            let metrics = manager.get_metrics();
            assert!(metrics.battery_rejections > 0);
        }
    
        #[wasm_bindgen_test]
        async fn test_peer_sync() {
            let manager = setup_sync_manager().await;
            let peer = [1u8; 32];
            let result = manager.sync_with_peer(&peer).await;
            assert!(result.is_ok());
            assert_eq!(
                *manager.peer_states.read().get(&peer).unwrap(),
                StorageOpCode::SyncState
            );
        }
    
        #[wasm_bindgen_test]
        async fn test_sync_retries() {
            let manager = setup_sync_manager().await;
            let peer = [2u8; 32];
            for _ in 0..manager.config.max_sync_retries {
                manager.sync_with_peer(&peer).await.unwrap_err();
            }
            assert_eq!(
                *manager.peer_states.read().get(&peer).unwrap(),
                StorageOpCode::ValidateSync
            );
            let metrics = manager.get_metrics();
            assert_eq!(metrics.suspended_peers, 1);
        }
    
        #[wasm_bindgen_test]
        async fn test_metrics_tracking() {
            let manager = setup_sync_manager().await;
            let peer = [3u8; 32];
            manager.sync_with_peer(&peer).await.unwrap();
            let metrics = manager.get_metrics();
            assert_eq!(metrics.successful_syncs, 1);
            assert_eq!(metrics.failed_syncs, 0);
            assert!(metrics.average_sync_time > 0.0);
        }
    
        #[wasm_bindgen_test]
        async fn test_concurrent_syncs() {
            let manager = setup_sync_manager().await;
            let mut peers = Vec::new();
            for i in 0..manager.config.max_concurrent_syncs + 1 {
                peers.push([i as u8; 32]);
            }
            for peer in &peers {
                let result = manager.sync_with_peer(peer).await;
                if peers.len() > manager.config.max_concurrent_syncs {
                    assert!(result.is_err());
                } else {
                    assert!(result.is_ok());
                }
            }
        }
    }
