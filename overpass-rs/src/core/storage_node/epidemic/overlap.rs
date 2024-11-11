// ./src/core/storage_node/epidemic/overlap.rs

use std::collections::{HashMap, HashSet};

pub struct StorageOverlapManager {
    node_responsibilities: HashMap<[u8; 32], HashSet<[u8; 32]>>, // node_id -> set of wallet_ids
    wallet_assignments: HashMap<[u8; 32], HashSet<[u8; 32]>>,    // wallet_id -> set of node_ids
    overlap_scores: HashMap<([u8; 32], [u8; 32]), f64>, // (node_id, node_id) -> overlap score
    min_overlap_threshold: f64,
    target_redundancy: usize,
}

impl StorageOverlapManager {
    pub fn new(min_overlap_threshold: f64, target_redundancy: usize) -> Self {
        Self {
            node_responsibilities: HashMap::new(),
            wallet_assignments: HashMap::new(),
            overlap_scores: HashMap::new(),
            min_overlap_threshold,
            target_redundancy,
        }
    }

    pub fn assign_wallet(&mut self, wallet_id: [u8; 32], nodes: Vec<[u8; 32]>) {
        let node_set: HashSet<[u8; 32]> = nodes.into_iter().collect();

        // Update wallet assignments
        self.wallet_assignments.insert(wallet_id, node_set.clone());

        // Update node responsibilities
        for node_id in node_set {
            self.node_responsibilities
                .entry(node_id)
                .or_insert_with(HashSet::new)
                .insert(wallet_id);
        }

        self.update_overlap_scores();
    }

    pub fn calculate_sync_boost(&self, node_id: &[u8; 32]) -> u64 {
        let mut total_overlap = 0.0;
        let mut count = 0;

        for (pair, score) in &self.overlap_scores {
            if pair.0 == *node_id || pair.1 == *node_id {
                total_overlap += score;
                count += 1;
            }
        }

        if count == 0 {
            return 1;
        }

        let avg_overlap = total_overlap / count as f64;
        let boost = (avg_overlap * 100.0).min(100.0).max(1.0);
        boost as u64
    }

    pub fn get_synchronized_nodes(&self, node_id: &[u8; 32]) -> HashSet<[u8; 32]> {
        let mut synchronized = HashSet::new();

        for (pair, score) in &self.overlap_scores {
            if pair.0 == *node_id && score >= &self.min_overlap_threshold {
                synchronized.insert(pair.1);
            } else if pair.1 == *node_id && score >= &self.min_overlap_threshold {
                synchronized.insert(pair.0);
            }
        }

        synchronized
    }

    fn update_overlap_scores(&mut self) {
        self.overlap_scores.clear();

        let nodes: Vec<[u8; 32]> = self.node_responsibilities.keys().cloned().collect();

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let node1 = &nodes[i];
                let node2 = &nodes[j];

                if let (Some(resp1), Some(resp2)) = (
                    self.node_responsibilities.get(node1),
                    self.node_responsibilities.get(node2),
                ) {
                    let intersection = resp1.intersection(resp2).count();
                    let union = resp1.union(resp2).count();

                    if union > 0 {
                        let overlap = intersection as f64 / union as f64;
                        self.overlap_scores.insert((*node1, *node2), overlap);
                    }
                }
            }
        }
    }

    pub fn needs_rebalancing(&self) -> bool {
        let mut has_low_overlap = false;
        let mut has_insufficient_redundancy = false;

        // Check overlap scores
        for score in self.overlap_scores.values() {
            if score < &self.min_overlap_threshold {
                has_low_overlap = true;
                break;
            }
        }

        // Check redundancy
        for assignments in self.wallet_assignments.values() {
            if assignments.len() < self.target_redundancy {
                has_insufficient_redundancy = true;
                break;
            }
        }

        has_low_overlap || has_insufficient_redundancy
    }

    pub fn get_redundancy_factor(&self, wallet_id: &[u8; 32]) -> usize {
        self.wallet_assignments
            .get(wallet_id)
            .map(|nodes| nodes.len())
            .unwrap_or(0)
    }
}
