// src/network/discovery.rs

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

/// Manages discovery and tracking of network nodes.
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct NodeInfo {
    active: bool,
    last_seen: Instant,
    reputation: i32,
}

pub struct NodeDiscovery {
    known_nodes: Arc<Mutex<HashMap<SocketAddr, NodeInfo>>>,
    cleanup_interval: Duration,
    inactive_threshold: Duration,
}

impl NodeDiscovery {
    /// Creates a new `NodeDiscovery` instance.
    pub fn new() -> Self {
        NodeDiscovery {
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            inactive_threshold: Duration::from_secs(3600), // 1 hour
        }
    }

    /// Creates a new `NodeDiscovery` instance with custom intervals.
    pub fn with_intervals(cleanup_interval: Duration, inactive_threshold: Duration) -> Self {
        NodeDiscovery {
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
            cleanup_interval,
            inactive_threshold,
        }
    }

    /// Adds a node to the known nodes list.
    pub async fn add_node(&self, addr: SocketAddr) {
        let mut nodes = self.known_nodes.lock().expect("Failed to lock mutex");

        nodes.insert(
            addr,
            NodeInfo {
                active: true,
                last_seen: Instant::now(),
                reputation: 0,
            },
        );
    }

    /// Updates the last seen time for a node.
    pub async fn update_node(&self, addr: SocketAddr) {
        let mut nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        if let Some(node) = nodes.get_mut(&addr) {
            node.last_seen = Instant::now();
            node.active = true;
        }
    }

    /// Marks a node as inactive.
    pub async fn mark_inactive(&self, addr: SocketAddr) {
        let mut nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        if let Some(node) = nodes.get_mut(&addr) {
            node.active = false;
        }
    }

    /// Updates the reputation of a node.
    pub async fn update_reputation(&self, addr: SocketAddr, delta: i32) {
        let mut nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        if let Some(node) = nodes.get_mut(&addr) {
            node.reputation += delta;
        }
    }

    /// Checks if a node is known.
    pub async fn is_known(&self, addr: &SocketAddr) -> bool {
        let nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes.contains_key(addr)
    }

    /// Checks if a node is active.
    pub async fn is_active(&self, addr: &SocketAddr) -> bool {
        let nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes
            .get(addr)
            .map(|node| node.active && node.last_seen.elapsed() < self.inactive_threshold)
            .unwrap_or(false)
    }

    /// Retrieves a list of all known nodes.
    pub async fn get_known_nodes(&self) -> Vec<SocketAddr> {
        let nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes.keys().cloned().collect()
    }

    /// Retrieves a list of active nodes.
    pub async fn get_active_nodes(&self) -> Vec<SocketAddr> {
        let nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes
            .iter()
            .filter(|(_, info)| info.active && info.last_seen.elapsed() < self.inactive_threshold)
            .map(|(addr, _)| *addr)
            .collect()
    }

    /// Gets the reputation of a node.
    pub async fn get_reputation(&self, addr: &SocketAddr) -> Option<i32> {
        let nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes.get(addr).map(|node| node.reputation)
    }

    /// Removes inactive nodes that haven't been seen for longer than the inactive threshold.
    pub async fn cleanup(&self) {
        let mut nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes.retain(|_, info| info.last_seen.elapsed() < self.inactive_threshold);
    }

    /// Removes a specific node from the known nodes list.
    pub async fn remove_node(&self, addr: &SocketAddr) {
        let mut nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes.remove(addr);
    }

    /// Gets the number of known nodes.
    pub async fn node_count(&self) -> usize {
        let nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes.len()
    }

    /// Gets the number of active nodes.
    pub async fn active_node_count(&self) -> usize {
        let nodes = self.known_nodes.lock().expect("Failed to lock mutex");
        nodes
            .values()
            .filter(|info| info.active && info.last_seen.elapsed() < self.inactive_threshold)
            .count()
    }
}
