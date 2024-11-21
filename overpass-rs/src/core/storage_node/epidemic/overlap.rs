use parking_lot::RwLock;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use crate::core::error::errors::{SystemError, SystemErrorType};

// Overlap score thresholds
const MIN_OVERLAP_SCORE: f64 = 0.8; // Minimum overlap required for synchronization
const TARGET_REDUNDANCY: usize = 3; // Target number of redundant copies
const MAX_REDUNDANCY: usize = 5; // Maximum allowed redundancy

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlapMetrics {
    pub total_peers: usize,
    pub synchronized_peers: usize,
    pub average_overlap_score: f64,
    pub highest_overlap_score: f64,
    pub lowest_overlap_score: f64,
    pub redundancy_factor: f64,
    pub rebalancing_needed: bool,
}

impl Default for OverlapMetrics {
    fn default() -> Self {
        Self {
            total_peers: 0,
            synchronized_peers: 0,
            average_overlap_score: 0.0,
            highest_overlap_score: 0.0,
            lowest_overlap_score: 0.0,
            redundancy_factor: 0.0,
            rebalancing_needed: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeOverlap {
    pub node_id: [u8; 32],
    pub shared_states: HashSet<[u8; 32]>, // States both nodes have
    pub overlap_score: f64,
    pub last_sync: u64,
    pub sync_success_rate: f64,
}

pub struct StorageOverlapManager {
    // Core state tracking
    node_responsibilities: RwLock<HashMap<[u8; 32], HashSet<[u8; 32]>>>, // node -> states
    state_assignments: RwLock<HashMap<[u8; 32], HashSet<[u8; 32]>>>,     // state -> nodes

    // Overlap tracking
    overlap_scores: RwLock<HashMap<([u8; 32], [u8; 32]), f64>>, // (node1, node2) -> score
    node_overlaps: RwLock<HashMap<[u8; 32], Vec<NodeOverlap>>>, // node -> overlapping nodes

    // Metrics
    metrics: RwLock<OverlapMetrics>,

    // Configuration
    min_overlap_threshold: f64,
    target_redundancy: usize,
}
impl StorageOverlapManager {
    pub fn new(min_overlap_threshold: f64, target_redundancy: usize) -> Self {
        Self {
            node_responsibilities: RwLock::new(HashMap::new()),
            state_assignments: RwLock::new(HashMap::new()),
            overlap_scores: RwLock::new(HashMap::new()),
            node_overlaps: RwLock::new(HashMap::new()),
            metrics: RwLock::new(OverlapMetrics::default()),
            min_overlap_threshold,
            target_redundancy,
        }
    }

    // Assign state to node
    pub fn assign_state(&self, state_id: [u8; 32], node_id: [u8; 32]) -> Result<(), SystemError> {
        // Update node responsibilities
        {
            let mut responsibilities = self.node_responsibilities.write();
            responsibilities
                .entry(node_id)
                .or_insert_with(HashSet::new)
                .insert(state_id);
        }

        // Update state assignments
        {
            let mut assignments = self.state_assignments.write();
            assignments
                .entry(state_id)
                .or_insert_with(HashSet::new)
                .insert(node_id);
        }

        // Update overlap scores
        self.update_overlap_scores(node_id)?;
        self.update_metrics();

        Ok(())
    }

    // Calculate overlap score between two nodes
    pub fn calculate_overlap_score(&self, node1: [u8; 32], node2: [u8; 32]) -> f64 {
        let responsibilities = self.node_responsibilities.read();

        if let (Some(states1), Some(states2)) =
            (responsibilities.get(&node1), responsibilities.get(&node2))
        {
            let intersection: HashSet<_> = states1.intersection(states2).collect();
            let union: HashSet<_> = states1.union(states2).collect();

            if !union.is_empty() {
                intersection.len() as f64 / union.len() as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    // Update overlap scores for a node
    fn update_overlap_scores(&self, node_id: [u8; 32]) -> Result<(), SystemError> {
        let responsibilities = self.node_responsibilities.read();
        let mut overlap_scores = self.overlap_scores.write();
        let mut node_overlaps = self.node_overlaps.write();

        let nodes: Vec<[u8; 32]> = responsibilities.keys().copied().collect();
        let mut node_overlap_list = Vec::new();

        for other_node in nodes {
            if other_node != node_id {
                let score = self.calculate_overlap_score(node_id, other_node);

                // Update overlap scores
                overlap_scores.insert((node_id, other_node), score);

                // Update node overlaps
                if score >= self.min_overlap_threshold {
                    let shared_states = {
                        if let (Some(states1), Some(states2)) = (
                            responsibilities.get(&node_id),
                            responsibilities.get(&other_node),
                        ) {
                            states1.intersection(states2).copied().collect()
                        } else {
                            HashSet::new()
                        }
                    };

                    node_overlap_list.push(NodeOverlap {
                        node_id: other_node,
                        shared_states,
                        overlap_score: score,
                        last_sync: 0, // Will be updated during sync
                        sync_success_rate: 1.0,
                    });
                }
            }
        }

        if !node_overlap_list.is_empty() {
            node_overlaps.insert(node_id, node_overlap_list);
        }

        Ok(())
    }

    // Get synchronized nodes (nodes with sufficient overlap)
    pub fn get_synchronized_nodes(&self) -> HashSet<[u8; 32]> {
        let overlap_scores = self.overlap_scores.read();
        let mut synchronized = HashSet::new();

        for ((node1, node2), score) in overlap_scores.iter() {
            if score >= &self.min_overlap_threshold {
                synchronized.insert(*node1);
                synchronized.insert(*node2);
            }
        }

        synchronized
    }

    // Check if rebalancing is needed
    pub fn needs_rebalancing(&self) -> bool {
        let state_assignments = self.state_assignments.read();
        let mut needs_rebalance = false;

        // Check redundancy
        for nodes in state_assignments.values() {
            if nodes.len() < self.target_redundancy {
                needs_rebalance = true;
                break;
            }
        }

        // Check overlap scores
        if !needs_rebalance {
            let overlap_scores = self.overlap_scores.read();
            for score in overlap_scores.values() {
                if score < &self.min_overlap_threshold {
                    needs_rebalance = true;
                    break;
                }
            }
        }

        needs_rebalance
    }

    // Calculate sync boost for a node based on overlap
    pub fn calculate_sync_boost(&self, node_id: &[u8; 32]) -> u64 {
        let overlap_scores = self.overlap_scores.read();
        let mut total_score = 0.0;
        let mut count = 0;

        for ((node1, node2), score) in overlap_scores.iter() {
            if node1 == node_id || node2 == node_id {
                total_score += score;
                count += 1;
            }
        }

        if count == 0 {
            return 1;
        }

        let average_score = total_score / count as f64;
        (average_score * 100.0).min(100.0).max(1.0) as u64
    }

    // Update metrics
    fn update_metrics(&self) {
        let mut metrics = self.metrics.write();
        let overlap_scores = self.overlap_scores.read();
        let state_assignments = self.state_assignments.read();

        if overlap_scores.is_empty() {
            return;
        }

        // Calculate overlap statistics
        let scores: Vec<f64> = overlap_scores.values().copied().collect();
        metrics.average_overlap_score = scores.iter().sum::<f64>() / scores.len() as f64;
        metrics.highest_overlap_score = scores.iter().copied().fold(0.0, f64::max);
        metrics.lowest_overlap_score = scores.iter().copied().fold(f64::MAX, f64::min);

        // Calculate redundancy
        let redundancy: Vec<usize> = state_assignments
            .values()
            .map(|nodes| nodes.len())
            .collect();
        metrics.redundancy_factor =
            redundancy.iter().sum::<usize>() as f64 / redundancy.len() as f64;

        // Update other metrics
        metrics.total_peers = self.node_responsibilities.read().len();
        metrics.synchronized_peers = self.get_synchronized_nodes().len();
        metrics.rebalancing_needed = self.needs_rebalancing();
    }

    // Get current metrics
    pub fn get_metrics(&self) -> OverlapMetrics {
        self.metrics.read().clone()
    }

    // Get overlap score for a specific node
    pub fn get_overlap_score(&self) -> f64 {
        self.metrics.read().average_overlap_score
    }

    // Get sync score (based on redundancy and overlap)
    pub fn get_sync_score(&self) -> f64 {
        let metrics = self.metrics.read();
        (metrics.redundancy_factor / self.target_redundancy as f64).min(1.0)
            * metrics.average_overlap_score
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn create_test_manager() -> StorageOverlapManager {
        StorageOverlapManager::new(MIN_OVERLAP_SCORE, TARGET_REDUNDANCY)
    }

    fn generate_node_id(index: u8) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = index;
        id
    }

    fn generate_state_id(index: u8) -> [u8; 32] {
        let mut id = [0u8; 32];
        id[0] = index;
        id
    }

    #[wasm_bindgen_test]
    fn test_state_assignment() {
        let manager = create_test_manager();
        let node_id = generate_node_id(1);
        let state_id = generate_state_id(1);

        assert!(manager.assign_state(state_id, node_id).is_ok());

        let responsibilities = manager.node_responsibilities.read();
        assert!(responsibilities.get(&node_id).unwrap().contains(&state_id));
    }

    #[wasm_bindgen_test]
    fn test_overlap_calculation() {
        let manager = create_test_manager();
        let node1 = generate_node_id(1);
        let node2 = generate_node_id(2);
        let state1 = generate_state_id(1);
        let state2 = generate_state_id(2);

        // Assign same state to both nodes
        manager.assign_state(state1, node1).unwrap();
        manager.assign_state(state1, node2).unwrap();

        // Assign different state to node2
        manager.assign_state(state2, node2).unwrap();

        let score = manager.calculate_overlap_score(node1, node2);
        assert!(score > 0.0 && score < 1.0);
    }

    #[wasm_bindgen_test]
    fn test_sync_boost() {
        let manager = create_test_manager();
        let node1 = generate_node_id(1);
        let node2 = generate_node_id(2);
        let state1 = generate_state_id(1);

        // Create perfect overlap
        manager.assign_state(state1, node1).unwrap();
        manager.assign_state(state1, node2).unwrap();

        let boost = manager.calculate_sync_boost(&node1);
        assert!(boost > 0);
    }

    #[wasm_bindgen_test]
    fn test_metrics() {
        let manager = create_test_manager();
        let node1 = generate_node_id(1);
        let node2 = generate_node_id(2);
        let state1 = generate_state_id(1);

        manager.assign_state(state1, node1).unwrap();
        manager.assign_state(state1, node2).unwrap();

        let metrics = manager.get_metrics();
        assert!(metrics.total_peers > 0);
        assert!(metrics.redundancy_factor > 0.0);
    }
}
