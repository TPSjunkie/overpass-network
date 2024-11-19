use std::sync::Arc;
use parking_lot::RwLock;
use hashbrown::{HashMap, HashSet};
use web_sys::window;
use serde::{Serialize, Deserialize};
use wasm_bindgen_futures::spawn_local;

use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::battery::charging::BatteryChargingSystem;
use crate::core::storage_node::epidemic::overlap::StorageOverlapManager;

/// Represents the various states a node can have regarding a message
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageState {
    Unknown,        // Never seen the message
    Seen,          // Received but not propagated
    Propagating,   // Currently propagating
    Propagated,    // Successfully propagated
    Failed,        // Failed to propagate
}

/// Message priorities for propagation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Critical = 0,   // Must propagate immediately
    High = 1,       // Should propagate soon
    Medium = 2,     // Normal priority
    Low = 3,        // Propagate when convenient
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationMessage {
    pub id: [u8; 32],
    pub data_hash: [u8; 32],
    pub source_node: [u8; 32],
    pub priority: MessagePriority,
    pub timestamp: u64,
    pub ttl: u32,
    pub battery_requirement: u64,  // Minimum battery needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationMetrics {
    pub messages_seen: u64,
    pub messages_propagated: u64,
    pub messages_failed: u64,
    pub active_propagations: u64,
    pub successful_peers: u64,
    pub failed_peers: u64,
    pub average_propagation_time: f64,
    pub battery_rejections: u64,
}

impl Default for PropagationMetrics {
    fn default() -> Self {
        Self {
            messages_seen: 0,
            messages_propagated: 0,
            messages_failed: 0,
            active_propagations: 0,
            successful_peers: 0,
            failed_peers: 0,
            average_propagation_time: 0.0,
            battery_rejections: 0,
        }
    }
}

pub struct EpidemicPropagation {
    battery_system: Arc<BatteryChargingSystem>,
    overlap_manager: Arc<StorageOverlapManager>,
    
    // Message tracking
    message_states: RwLock<HashMap<[u8; 32], MessageState>>,
    active_messages: RwLock<HashMap<[u8; 32], PropagationMessage>>,
    seen_messages: RwLock<HashSet<[u8; 32]>>,
    
    // Peer tracking
    peer_success_rates: RwLock<HashMap<[u8; 32], f64>>,
    peer_last_propagation: RwLock<HashMap<[u8; 32], u64>>,
    
    // Metrics
    metrics: RwLock<PropagationMetrics>,
    
    // Configuration
    min_battery_levels: HashMap<MessagePriority, u64>,
    max_active_propagations: usize,
    propagation_timeout: u64,
    max_retries: u32,
}

impl EpidemicPropagation {
    pub fn new(
        battery_system: Arc<BatteryChargingSystem>,
        overlap_manager: Arc<StorageOverlapManager>,
    ) -> Self {
        let mut min_battery_levels = HashMap::new();
        min_battery_levels.insert(MessagePriority::Critical, 5);  // 5% minimum
        min_battery_levels.insert(MessagePriority::High, 20);     // 20% minimum
        min_battery_levels.insert(MessagePriority::Medium, 40);   // 40% minimum
        min_battery_levels.insert(MessagePriority::Low, 60);      // 60% minimum

        Self {
            battery_system,
            overlap_manager,
            message_states: RwLock::new(HashMap::new()),
            active_messages: RwLock::new(HashMap::new()),
            seen_messages: RwLock::new(HashSet::new()),
            peer_success_rates: RwLock::new(HashMap::new()),
            peer_last_propagation: RwLock::new(HashMap::new()),
            metrics: RwLock::new(PropagationMetrics::default()),
            min_battery_levels,
            max_active_propagations: 10,
            propagation_timeout: 30000,  // 30 seconds
            max_retries: 3,
        }
    }

    // Handle incoming message
    pub async fn handle_message(&self, message: PropagationMessage) -> Result<(), SystemError> {
        // Check if we've seen this message before
        if !self.seen_messages.write().insert(message.id) {
            return Ok(());
        }

        // Update metrics
        self.metrics.write().messages_seen += 1;

        // Check battery level requirements
        let battery_level = self.battery_system.get_charge_percentage();
        let min_battery = self.min_battery_levels.get(&message.priority)
            .copied()
            .unwrap_or(100);

        if battery_level < min_battery as f64 {
            self.metrics.write().battery_rejections += 1;
            return Err(SystemError::new(
                SystemErrorType::InsufficientBattery,
                "Insufficient battery for propagation"
            ));
        }

        // Check if we can handle more propagations
        if self.active_messages.read().len() >= self.max_active_propagations {
            return Err(SystemError::new(
                SystemErrorType::TooManyPropagations,
                "Maximum active propagations reached"
            ));
        }

        // Start propagation
        self.start_propagation(message).await
    }

    // Start propagating a message
    async fn start_propagation(&self, message: PropagationMessage) -> Result<(), SystemError> {
        // Update message state
        self.message_states.write().insert(message.id, MessageState::Propagating);
        self.active_messages.write().insert(message.id, message.clone());
        self.metrics.write().active_propagations += 1;

        // Get synchronized peers
        let sync_peers = self.overlap_manager.get_synchronized_nodes();
        let selected_peers = self.select_propagation_targets(&sync_peers).await?;

        // Spawn propagation task
        let propagation = self.clone();
        spawn_local(async move {
            let start_time = window().unwrap().performance().unwrap().now();
            let mut successful_peers = 0;
            let mut failed_peers = 0;

            for peer in selected_peers {
                match propagation.propagate_to_peer(&message, &peer).await {
                    Ok(_) => {
                        successful_peers += 1;
                        propagation.update_peer_success(&peer, true).await;
                    }
                    Err(_) => {
                        failed_peers += 1;
                        propagation.update_peer_success(&peer, false).await;
                    }
                }
            }

            // Update metrics
            let mut metrics = propagation.metrics.write();
            let elapsed = window().unwrap().performance().unwrap().now() - start_time;
            metrics.successful_peers += successful_peers;
            metrics.failed_peers += failed_peers;
            
            // Update average propagation time
            let total_propagations = metrics.messages_propagated;
            metrics.average_propagation_time = 
                (metrics.average_propagation_time * total_propagations as f64 + elapsed) /
                (total_propagations + 1) as f64;

            // Update final state
            if successful_peers > 0 {
                propagation.message_states.write().insert(message.id, MessageState::Propagated);
                metrics.messages_propagated += 1;
            } else {
                propagation.message_states.write().insert(message.id, MessageState::Failed);
                metrics.messages_failed += 1;
            }

            metrics.active_propagations -= 1;
            propagation.active_messages.write().remove(&message.id);
        });

        Ok(())
    }

    // Select peers for propagation based on sync scores and success rates
    async fn select_propagation_targets(&self, peers: &HashSet<[u8; 32]>) -> Result<Vec<[u8; 32]>, SystemError> {
        let mut selected_peers = Vec::new();
        let success_rates = self.peer_success_rates.read();
        
        // Sort peers by success rate and sync score
        let mut scored_peers: Vec<_> = peers.iter().map(|peer| {
            let success_rate = success_rates.get(peer).copied().unwrap_or(1.0);
            let sync_score = self.overlap_manager.calculate_sync_boost(peer);
            (*peer, success_rate * sync_score as f64)
        }).collect();

        scored_peers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Select top peers
        for (peer, _) in scored_peers.iter().take(3) {
            selected_peers.push(*peer);
        }

        Ok(selected_peers)
    }

    // Propagate message to specific peer
    async fn propagate_to_peer(&self, message: &PropagationMessage, peer: &[u8; 32]) -> Result<(), SystemError> {
        // Here we would implement actual peer communication
        // For now, simulate success/failure
        if self.peer_success_rates.read().get(peer).copied().unwrap_or(1.0) > 0.8 {
            Ok(())
        } else {
            Err(SystemError::new(
                SystemErrorType::PropagationFailed,
                "Peer propagation failed"
            ))
        }
    }

    // Update peer success rate
    async fn update_peer_success(&self, peer: &[u8; 32], success: bool) {
        let mut success_rates = self.peer_success_rates.write();
        let current_rate = success_rates.get(peer).copied().unwrap_or(1.0);
        let new_rate = if success {
            (current_rate * 0.9) + 0.1  // Slowly increase
        } else {
            current_rate * 0.9  // Quickly decrease
        };
        success_rates.insert(*peer, new_rate);
        
        // Update last propagation time
        self.peer_last_propagation.write().insert(*peer, 
            window().unwrap().performance().unwrap().now() as u64);
    }

    // Get current metrics
    pub fn get_metrics(&self) -> PropagationMetrics {
        self.metrics.read().clone()
    }

    // Check message state
    pub fn get_message_state(&self, message_id: &[u8; 32]) -> MessageState {
        self.message_states.read()
            .get(message_id)
            .copied()
            .unwrap_or(MessageState::Unknown)
    }
}

impl Clone for EpidemicPropagation {
    fn clone(&self) -> Self {
        Self {
            battery_system: Arc::clone(&self.battery_system),
            overlap_manager: Arc::clone(&self.overlap_manager),
            message_states: RwLock::new(HashMap::new()),
            active_messages: RwLock::new(HashMap::new()),
            seen_messages: RwLock::new(HashSet::new()),
            peer_success_rates: RwLock::new(HashMap::new()),
            peer_last_propagation: RwLock::new(HashMap::new()),
            metrics: RwLock::new(PropagationMetrics::default()),
            min_battery_levels: self.min_battery_levels.clone(),
            max_active_propagations: self.max_active_propagations,
            propagation_timeout: self.propagation_timeout,
            max_retries: self.max_retries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn create_test_message(priority: MessagePriority) -> PropagationMessage {
        PropagationMessage {
            id: [0u8; 32],
            data_hash: [0u8; 32],
            source_node: [0u8; 32],
            priority,
            timestamp: 0,
            ttl: 10,
            battery_requirement: 0,
        }
    }

    async fn setup_propagation() -> EpidemicPropagation {
        let battery_system = Arc::new(BatteryChargingSystem::new(Default::default()));
        let overlap_manager = Arc::new(StorageOverlapManager::new(0.8, 3));
        EpidemicPropagation::new(battery_system, overlap_manager)
    }

    #[wasm_bindgen_test]
    async fn test_message_handling() {
        let propagation = setup_propagation().await;
        let message = create_test_message(MessagePriority::High);
        
        let result = propagation.handle_message(message.clone()).await;
        assert!(result.is_ok());
        
        // Check message state
        assert_eq!(
            propagation.get_message_state(&message.id),
            MessageState::Propagating
        );
    }

    #[wasm_bindgen_test]
    async fn test_battery_requirements() {
        let propagation = setup_propagation().await;
        
        // Drain battery
        propagation.battery_system.consume_charge(90).await.unwrap();
        
        // Try propagating high priority message
        let message = create_test_message(MessagePriority::High);
        let result = propagation.handle_message(message).await;
        
        assert!(result.is_err());
        let metrics = propagation.get_metrics();
        assert_eq!(metrics.battery_rejections, 1);
    }

    #[wasm_bindgen_test]
    async fn test_duplicate_messages() {
        let propagation = setup_propagation().await;
        let message = create_test_message(MessagePriority::Medium);
        
        // First attempt should succeed
        propagation.handle_message(message.clone()).await.unwrap();
        
        // Second attempt should be ignored
        propagation.handle_message(message.clone()).await.unwrap();
        
        let metrics = propagation.get_metrics();
        assert_eq!(metrics.messages_seen, 1);
    }

    #[wasm_bindgen_test]
    async fn test_peer_success_tracking() {
        let propagation = setup_propagation().await;
        let peer = [1u8; 32];
        
        // Update success rate
        propagation.update_peer_success(&peer, true).await;
        
        let success_rate = propagation.peer_success_rates.read().get(&peer).copied().unwrap();
        assert!(success_rate > 0.0);
    }
}