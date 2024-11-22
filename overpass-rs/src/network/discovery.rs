// src/network/discovery.rs improvements

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock; // Replace std Mutex with tokio RwLock

#[derive(Debug, Clone)] // Make NodeInfo Clone
pub struct NodeInfo {
    active: bool,
    last_seen: Instant,
    reputation: i32,
}

pub struct NodeDiscovery {
    // Use RwLock instead of Mutex for better concurrency
    known_nodes: Arc<RwLock<HashMap<SocketAddr, NodeInfo>>>,
    cleanup_interval: Duration,
    inactive_threshold: Duration,
}

impl NodeDiscovery {
    pub fn new() -> Self {
        NodeDiscovery {
            known_nodes: Arc::new(RwLock::new(HashMap::new())),
            cleanup_interval: Duration::from_secs(300),
            inactive_threshold: Duration::from_secs(3600),
        }
    }

    // Modified methods to use RwLock properly
    pub async fn add_node(&self, addr: SocketAddr) {
        let mut nodes = self.known_nodes.write().await;
        nodes.insert(
            addr,
            NodeInfo {
                active: true,
                last_seen: Instant::now(),
                reputation: 0,
            },
        );
    }

    pub async fn update_node(&self, addr: SocketAddr) {
        let mut nodes = self.known_nodes.write().await;
        if let Some(node) = nodes.get_mut(&addr) {
            node.last_seen = Instant::now();
            node.active = true;
        }
    }

    // Add batch operations to reduce lock contention
    pub async fn update_nodes_batch(&self, addrs: &[SocketAddr]) {
        let mut nodes = self.known_nodes.write().await;
        for addr in addrs {
            if let Some(node) = nodes.get_mut(addr) {
                node.last_seen = Instant::now();
                node.active = true;
            }
        }
    }

    // Use read lock for queries that don't modify data
    pub async fn get_node_info(&self, addr: &SocketAddr) -> Option<NodeInfo> {
        let nodes = self.known_nodes.read().await;
        nodes.get(addr).cloned()
    }
}
