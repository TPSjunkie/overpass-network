use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::battery::charging::BatteryChargingSystem;
use crate::core::storage_node::battery::monitoring::{
    MessagePriority, MessageState, PropagationMessage, PropagationMetrics,
};
use crate::core::storage_node::epidemic::overlap::StorageOverlapManager;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

#[async_trait(?Send)]
pub trait NetworkSystem {
    async fn send_message(&self, peer: &[u8; 32], payload: &[u8]) -> Result<(), SystemError>;
    async fn wait_for_ack(&self, peer: &[u8; 32], message_id: [u8; 32]) -> Result<(), SystemError>;
}

pub struct EpidemicPropagation {
    battery_system: Arc<BatteryChargingSystem>,
    overlap_manager: Arc<StorageOverlapManager>,
    network: Arc<RwLock<dyn NetworkSystem>>,
    message_states: RwLock<HashMap<[u8; 32], MessageState>>,
    active_messages: RwLock<HashMap<[u8; 32], PropagationMessage>>,
    seen_messages: RwLock<HashSet<[u8; 32]>>,
    peer_success_rates: RwLock<HashMap<[u8; 32], f64>>,
    peer_last_propagation: RwLock<HashMap<[u8; 32], u64>>,
    metrics: RwLock<PropagationMetrics>,
    min_battery_levels: HashMap<MessagePriority, u64>,
    max_active_propagations: usize,
    propagation_timeout: u64,
    max_retries: u32,
}

impl EpidemicPropagation {
    pub fn new(
        battery_system: Arc<BatteryChargingSystem>,
        overlap_manager: Arc<StorageOverlapManager>,
        network: Arc<RwLock<dyn NetworkSystem>>,
    ) -> Self {
        let mut min_battery_levels = HashMap::new();
        min_battery_levels.insert(MessagePriority::Critical, 5);
        min_battery_levels.insert(MessagePriority::High, 20);
        min_battery_levels.insert(MessagePriority::Medium, 40);
        min_battery_levels.insert(MessagePriority::Low, 60);

        Self {
            battery_system,
            overlap_manager,
            network,
            message_states: RwLock::new(HashMap::new()),
            active_messages: RwLock::new(HashMap::new()),
            seen_messages: RwLock::new(HashSet::new()),
            peer_success_rates: RwLock::new(HashMap::new()),
            peer_last_propagation: RwLock::new(HashMap::new()),
            metrics: RwLock::new(PropagationMetrics::default()),
            min_battery_levels,
            max_active_propagations: 10,
            propagation_timeout: 30000,
            max_retries: 3,
        }
    }

    pub async fn propagate_message(&self, message: PropagationMessage) -> Result<(), SystemError> {
        if !self.seen_messages.write().insert(message.id) {
            return Ok(());
        }

        self.metrics.write().messages_seen += 1;

        let battery_level = self.battery_system.get_charge_percentage();
        let min_battery = self
            .min_battery_levels
            .get(&message.priority)
            .ok_or_else(|| {
                SystemError::new(
                    SystemErrorType::InvalidInput,
                    "Invalid message priority".to_string(),
                )
            })?;

        if battery_level < *min_battery as f64 {
            self.metrics.write().battery_rejections += 1;
            return Err(SystemError::new(
                SystemErrorType::LowBattery,
                "Insufficient battery for propagation".to_string(),
            ));
        }

        if self.active_messages.read().len() >= self.max_active_propagations {
            return Err(SystemError::new(
                SystemErrorType::ResourceLimitReached,
                "Maximum active propagations reached".to_string(),
            ));
        }

        self.start_propagation(message).await
    }

    async fn start_propagation(&self, message: PropagationMessage) -> Result<(), SystemError> {
        self.message_states
            .write()
            .insert(message.id, MessageState::Propagating);
        self.active_messages
            .write()
            .insert(message.id, message.clone());
        self.metrics.write().active_propagations += 1;

        let sync_peers = self.overlap_manager.get_synchronized_nodes();
        let selected_peers = self.select_propagation_targets(&sync_peers).await?;

        let epidemic = Arc::new(self.clone());
        let message_id = message.id;

        spawn_local(async move {
            let start_time = window().unwrap().performance().unwrap().now();
            let mut successful_peers = 0;
            let mut failed_peers = 0;

            for peer in &selected_peers {
                match epidemic.propagate_to_peer(&message, peer).await {
                    Ok(_) => {
                        successful_peers += 1;
                        epidemic.update_peer_success(peer, true).await;
                    }
                    Err(_) => {
                        failed_peers += 1;
                        epidemic.update_peer_success(peer, false).await;
                    }
                }
            }

            let mut metrics = epidemic.metrics.write();
            let elapsed = window().unwrap().performance().unwrap().now() - start_time;
            metrics.successful_peers += successful_peers;
            metrics.failed_peers += failed_peers;

            let total_propagations = metrics.messages_propagated;
            metrics.average_propagation_time = if total_propagations > 0 {
                (metrics.average_propagation_time * total_propagations as f64 + elapsed)
                    / (total_propagations + 1) as f64
            } else {
                elapsed
            };

            let state = if successful_peers > 0 {
                MessageState::Propagated
            } else {
                MessageState::Failed
            };

            epidemic.message_states.write().insert(message_id, state);
            metrics.active_propagations = metrics.active_propagations.saturating_sub(1);
            epidemic.active_messages.write().remove(&message_id);
        });

        Ok(())
    }

    async fn propagate_to_peer(
        &self,
        message: &PropagationMessage,
        peer: &[u8; 32],
    ) -> Result<(), SystemError> {
        let message = message.clone();

        let payload = bincode::serialize(&message).map_err(|e| {
            SystemError::new(
                SystemErrorType::SerializationError,
                format!("Failed to serialize message: {}", e),
            )
        })?;

        let network = self.network.read();

        network.send_message(peer, &payload).await?;

        match network.wait_for_ack(peer, message.id).await {
            Ok(_) => {
                self.update_peer_success(peer, true).await;
                self.metrics.write().messages_propagated += 1;
                Ok(())
            }
            Err(e) => {
                self.update_peer_success(peer, false).await;
                Err(SystemError::new(
                    SystemErrorType::NetworkError,
                    format!("Failed to receive acknowledgment: {}", e),
                ))
            }
        }
    }

    async fn select_propagation_targets(
        &self,
        peers: &HashSet<[u8; 32]>,
    ) -> Result<Vec<[u8; 32]>, SystemError> {
        let mut selected_peers = Vec::new();
        let success_rates = self.peer_success_rates.read();

        let mut scored_peers: Vec<_> = peers
            .iter()
            .map(|peer| {
                let success_rate = success_rates.get(peer).copied().unwrap_or(1.0);
                let sync_score = self.overlap_manager.calculate_sync_boost(peer);
                (*peer, success_rate * sync_score as f64)
            })
            .collect();

        scored_peers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for (peer, _) in scored_peers.iter().take(3) {
            selected_peers.push(*peer);
        }

        Ok(selected_peers)
    }

    async fn update_peer_success(&self, peer: &[u8; 32], success: bool) {
        let mut success_rates = self.peer_success_rates.write();
        let current_rate = success_rates.get(peer).copied().unwrap_or(1.0);
        let new_rate = if success {
            (current_rate * 0.9) + 0.1
        } else {
            current_rate * 0.9
        };
        success_rates.insert(*peer, new_rate);

        self.peer_last_propagation
            .write()
            .insert(*peer, window().unwrap().performance().unwrap().now() as u64);
    }

    pub fn get_metrics(&self) -> PropagationMetrics {
        self.metrics.read().clone()
    }

    pub fn get_message_state(&self, message_id: &[u8; 32]) -> MessageState {
        self.message_states
            .read()
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
            network: Arc::clone(&self.network),
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

    struct MockNetworkSystem {
        send_success: bool,
        ack_success: bool,
    }

    #[async_trait(?Send)]
    impl NetworkSystem for MockNetworkSystem {
        async fn send_message(&self, _peer: &[u8; 32], _payload: &[u8]) -> Result<(), SystemError> {
            if self.send_success {
                Ok(())
            } else {
                Err(SystemError::new(
                    SystemErrorType::NetworkError,
                    "Mock send failed".into(),
                ))
            }
        }

        async fn wait_for_ack(
            &self,
            _peer: &[u8; 32],
            _message_id: [u8; 32],
        ) -> Result<(), SystemError> {
            if self.ack_success {
                Ok(())
            } else {
                Err(SystemError::new(
                    SystemErrorType::NetworkError,
                    "Mock ack failed".into(),
                ))
            }
        }
    }

    async fn setup_test_propagation() -> EpidemicPropagation {
        let battery_system = Arc::new(BatteryChargingSystem::new(Default::default()));
        let overlap_manager = Arc::new(StorageOverlapManager::new(0.8, 3));
        let network = Arc::new(RwLock::new(MockNetworkSystem {
            send_success: true,
            ack_success: true,
        }));
        EpidemicPropagation::new(battery_system, overlap_manager, network)
    }

    fn create_test_message(priority: MessagePriority) -> PropagationMessage {
        PropagationMessage {
            id: [0; 32],
            data_hash: [0; 32],
            source_node: [0; 32],
            priority,
            timestamp: 0,
            ttl: 10,
            battery_requirement: 0,
        }
    }

    #[wasm_bindgen_test]
    async fn test_message_handling() {
        let propagation = setup_test_propagation().await;
        let message = create_test_message(MessagePriority::High);

        let result = propagation.propagate_message(message.clone()).await;
        assert!(result.is_ok());

        assert_eq!(
            propagation.get_message_state(&message.id),
            MessageState::Propagating
        );
    }

    #[wasm_bindgen_test]
    async fn test_metrics_tracking() {
        let propagation = setup_test_propagation().await;
        let message = create_test_message(MessagePriority::Medium);

        let _ = propagation.propagate_message(message.clone()).await;

        let metrics = propagation.get_metrics();
        assert_eq!(metrics.messages_seen, 1);
    }

    #[wasm_bindgen_test]
    async fn test_battery_requirements() {
        let propagation = setup_test_propagation().await;
        let mut message = create_test_message(MessagePriority::Low);
        message.battery_requirement = 100;

        let result = propagation.propagate_message(message).await;
        assert!(result.is_err());

        let metrics = propagation.get_metrics();
        assert_eq!(metrics.battery_rejections, 1);
    }

    #[wasm_bindgen_test]
    async fn test_peer_selection() {
        let propagation = setup_test_propagation().await;
        let mut peers = HashSet::new();
        peers.insert([1u8; 32]);
        peers.insert([2u8; 32]);
        peers.insert([3u8; 32]);
        peers.insert([4u8; 32]);

        let selected = propagation
            .select_propagation_targets(&peers)
            .await
            .unwrap();
        assert_eq!(selected.len(), 3);
    }

    #[wasm_bindgen_test]
    async fn test_peer_success_rates() {
        let propagation = setup_test_propagation().await;
        let peer = [1u8; 32];

        propagation.update_peer_success(&peer, true).await;
        let rates = propagation.peer_success_rates.read();
        assert!(rates.contains_key(&peer));
        assert!(*rates.get(&peer).unwrap() > 0.0);
    }
}
