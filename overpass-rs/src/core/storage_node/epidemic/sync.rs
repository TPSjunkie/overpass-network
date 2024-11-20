use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use web_sys::{console, window};
use wasm_bindgen_futures::spawn_local;

use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::battery::charging::BatteryChargingSystem;
use crate::core::storage_node::epidemic::overlap::StorageOverlapManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncState {
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
    pub min_battery_for_sync: u64,      // Minimum battery level to start sync
    pub max_concurrent_syncs: usize,     // Maximum concurrent sync operations
    pub sync_interval: u64,             // Milliseconds between sync attempts
    pub sync_timeout: u64,              // Milliseconds before sync timeout
    pub max_sync_retries: u32,          // Maximum retry attempts
    pub min_overlap_for_sync: f64,      // Minimum overlap score to attempt sync
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            min_battery_for_sync: 20,    // 20% minimum battery
            max_concurrent_syncs: 3,      // Max 3 concurrent syncs
            sync_interval: 5000,          // 5 seconds between syncs
            sync_timeout: 30000,          // 30 second timeout
            max_sync_retries: 3,          // 3 retry attempts
            min_overlap_for_sync: 0.6,    // 60% minimum overlap
        }
    }
}

pub struct SynchronizationManager {
    battery_system: Arc<BatteryChargingSystem>,
    overlap_manager: Arc<StorageOverlapManager>,
    config: SyncConfig,
    
    // Sync state tracking
    peer_states: RwLock<HashMap<[u8; 32], SyncState>>,
    active_syncs: RwLock<HashSet<[u8; 32]>>,
    sync_attempts: RwLock<HashMap<[u8; 32], u32>>,
    last_sync: RwLock<HashMap<[u8; 32], u64>>,
    
    // Metrics
    metrics: RwLock<SyncMetrics>,
    
    // Verification tracking
    verified_states: RwLock<HashMap<[u8; 32], HashSet<[u8; 32]>>>, // peer -> verified states
    is_syncing: RwLock<bool>,
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
        }
    }

    // Start synchronization process
    pub async fn start_sync(&self) -> Result<(), SystemError> {
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
                
                // Wait for next sync interval
                wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
                    window().unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            manager.config.sync_interval as i32,
                        )
                        .unwrap();
                }))
                .await
                .unwrap();
            }
        });

        Ok(())
    }

    // Stop synchronization
    pub fn stop_sync(&self) {
        *self.is_syncing.write() = false;
    }

    // Main sync cycle
    async fn sync_cycle(&self) {
        // Check battery level
        let battery_level = self.battery_system.get_charge_percentage();
        if battery_level < self.config.min_battery_for_sync as f64 {
            self.metrics.write().battery_rejections += 1;
            return;
        }

        // Get potential sync peers
        let sync_peers = self.overlap_manager.get_synchronized_nodes();
        if sync_peers.is_empty() {
            return;
        }

        // Filter and prioritize peers
        let peers_to_sync = self.select_sync_peers(&sync_peers).await;
        for peer in peers_to_sync {
            if let Err(e) = self.sync_with_peer(&peer).await {
                console::warn_1(&format!("Sync failed with peer {:?}: {:?}", peer, e).into());
            }
        }

        self.update_metrics().await;
    }

    // Select peers for synchronization
    async fn select_sync_peers(&self, available_peers: &HashSet<[u8; 32]>) -> Vec<[u8; 32]> {
        let mut selected_peers = Vec::new();
        let active_syncs = self.active_syncs.read();
        let peer_states = self.peer_states.read();
        let last_sync = self.last_sync.read();
        let now = window().unwrap().performance().unwrap().now() as u64;

        for peer in available_peers {
            // Skip if already syncing
            if active_syncs.contains(peer) {
                continue;
            }

            // Skip suspended peers
            if peer_states.get(peer) == Some(&SyncState::Suspended) {
                continue;
            }

            // Check sync interval
            if let Some(last) = last_sync.get(peer) {
                if now - last < self.config.sync_interval {
                    continue;
                }
            }

            // Check overlap score
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

    // Sync with specific peer
    async fn sync_with_peer(&self, peer: &[u8; 32]) -> Result<(), SystemError> {
        // Update state and tracking
        {
            let mut active_syncs = self.active_syncs.write();
            if active_syncs.len() >= self.config.max_concurrent_syncs {
                return Err(SystemError::new(
                    SystemErrorType::TooManySyncs,
                    "Maximum concurrent syncs reached".to_owned()
                ));
            }
            active_syncs.insert(*peer);
        }
        
        self.peer_states.write().insert(*peer, SyncState::Syncing);
        let start_time = window().unwrap().performance().unwrap().now();

        // Attempt sync
        match self.execute_sync(peer).await {
            Ok(()) => {
                self.handle_successful_sync(peer, start_time).await;
                Ok(())
            }
            Err(e) => {
                self.handle_failed_sync(peer).await;
                Err(e)
            }
        }
    }

    // Execute sync operation
    async fn execute_sync(&self, peer: &[u8; 32]) -> Result<(), SystemError> {
        // Get states to sync
        let states_to_sync = self.get_unverified_states(peer).await?;
        if states_to_sync.is_empty() {
            return Ok(());
        }

        // Verify states
        for state in states_to_sync {
            if let Err(e) = self.verify_state(peer, state).await {
                console::warn_1(&format!(
                    "State verification failed for peer {:?}, state {:?}: {:?}",
                    peer, state, e
                ).into());
                continue;
            }
        }

        Ok(())
    }

    // Handle successful sync
    async fn handle_successful_sync(&self, peer: &[u8; 32], start_time: f64) {
        let mut metrics = self.metrics.write();
        metrics.successful_syncs += 1;
        
        let elapsed = window().unwrap().performance().unwrap().now() - start_time;
        metrics.average_sync_time = 
            (metrics.average_sync_time * metrics.successful_syncs as f64 + elapsed) /
            (metrics.successful_syncs + 1) as f64;

        // Update states
        self.peer_states.write().insert(*peer, SyncState::Verified);
        self.active_syncs.write().remove(peer);
        self.sync_attempts.write().remove(peer);
        self.last_sync.write().insert(*peer, window().unwrap().performance().unwrap().now() as u64);
    }

    // Handle failed sync
    async fn handle_failed_sync(&self, peer: &[u8; 32]) {
        let mut metrics = self.metrics.write();
        metrics.failed_syncs += 1;

        // Update retry count
        let mut attempts = self.sync_attempts.write();
        let attempt_count = attempts.entry(*peer).or_insert(0);
        *attempt_count += 1;

        // Check if we should suspend
        if *attempt_count >= self.config.max_sync_retries {
            self.peer_states.write().insert(*peer, SyncState::Suspended);
            metrics.suspended_peers += 1;
        } else {
            self.peer_states.write().insert(*peer, SyncState::Failed);
        }

        self.active_syncs.write().remove(peer);
    }

    // Get states that need verification
    async fn get_unverified_states(&self, peer: &[u8; 32]) -> Result<HashSet<[u8; 32]>, SystemError> {
        let verified = self.verified_states.read().get(peer)
            .cloned()
            .unwrap_or_default();

        let all_states = self.overlap_manager.as_ref().get_shared_states(peer).await?;
        Ok(all_states.difference(&verified).copied().collect())
    }
    // Verify specific state
    async fn verify_state(&self, peer: &[u8; 32], state: [u8; 32]) -> Result<(), SystemError> {
        // Here we would implement actual state verification
        // For now, simulate success
        self.verified_states.write()
            .entry(*peer)
            .or_insert_with(HashSet::new)
            .insert(state);
        
        Ok(())
    }

    // Update metrics
    async fn update_metrics(&self) {
        let mut metrics = self.metrics.write();
        metrics.active_syncs = self.active_syncs.read().len() as u64;
        metrics.verified_peers = self.peer_states.read()
            .values()
            .filter(|&state| *state == SyncState::Verified)
            .count();
    }

    // Get current metrics
    pub fn get_metrics(&self) -> SyncMetrics {
        self.metrics.read().clone()
    }
}

impl Clone for SynchronizationManager {
    fn clone(&self) -> Self {
        Self {
            battery_system: Arc::clone(&self.battery_system),
            overlap_manager: Arc::clone(&self.overlap_manager),
            config: self.config.clone(),
            peer_states: RwLock::new(HashMap::new()),
            active_syncs: RwLock::new(HashSet::new()),
            sync_attempts: RwLock::new(HashMap::new()),
            last_sync: RwLock::new(HashMap::new()),
            metrics: RwLock::new(SyncMetrics::default()),
            verified_states: RwLock::new(HashMap::new()),
            is_syncing: RwLock::new(false),
        }
    }
}

#[cfg(test)]
mod tests {
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
        
        // Start sync
        assert!(manager.start_sync().await.is_ok());
        assert!(*manager.is_syncing.read());
        
        // Stop sync
        manager.stop_sync();
        assert!(!*manager.is_syncing.read());
    }

    #[wasm_bindgen_test]
    async fn test_battery_requirements() {
        let manager = setup_sync_manager().await;
        
        // Drain battery
        Arc::get_mut(&mut manager.battery_system).unwrap().consume_charge(90).await.unwrap();
        
        // Try sync cycle
        manager.sync_cycle().await;
        
        // Check metrics
        let metrics = manager.get_metrics();
        assert!(metrics.battery_rejections > 0);
    }

    #[wasm_bindgen_test]
    async fn test_peer_sync() {
        let manager = setup_sync_manager().await;
        let peer = [1u8; 32];
        
        // Attempt sync
        let result = manager.sync_with_peer(&peer).await;
        assert!(result.is_ok());
        
        // Verify state updated
        assert_eq!(
            *manager.peer_states.read().get(&peer).unwrap(),
            SyncState::Verified
        );
    }

    #[wasm_bindgen_test]
    async fn test_sync_retries() {
        let manager = setup_sync_manager().await;
        let peer = [2u8; 32];
        
        // Force multiple failures
        for _ in 0..manager.config.max_sync_retries {
            manager.handle_failed_sync(&peer).await;
        }
        
        // Check if peer was suspended
        assert_eq!(
            *manager.peer_states.read().get(&peer).unwrap(),
            SyncState::Suspended
        );
        
        // Verify metrics
        let metrics = manager.get_metrics();
        assert_eq!(metrics.suspended_peers, 1);
    }
 
    #[wasm_bindgen_test]
    async fn test_metrics_tracking() {
        let manager = setup_sync_manager().await;
        let peer = [3u8; 32];
        
        // Successful sync
        manager.sync_with_peer(&peer).await.unwrap();
        
        let metrics = manager.get_metrics();
        assert_eq!(metrics.successful_syncs, 1);
        assert_eq!(metrics.failed_syncs, 0);
        assert!(metrics.average_sync_time > 0.0);
    }
 
    #[wasm_bindgen_test]
    async fn test_concurrent_syncs() {
        let manager = setup_sync_manager().await;
        
        // Try to sync with more peers than allowed
        let mut peers = Vec::new();
        for i in 0..manager.config.max_concurrent_syncs + 1 {
            peers.push([i as u8; 32]);
        }
 
        for peer in &peers {
            if peers.len() > manager.config.max_concurrent_syncs {
                assert!(manager.sync_with_peer(peer).await.is_err());
            } else {
                assert!(manager.sync_with_peer(peer).await.is_ok());
            }
        }
    }
 }