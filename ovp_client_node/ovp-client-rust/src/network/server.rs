// ./src/network/server.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use std::collections::HashMap;
use crate::common::error::client_errors::{SystemError, SystemErrorType};
use bitcoin::Network;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub network: Network,
    pub max_connections: usize,
    pub timeout_seconds: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8332,
            network: Network::Bitcoin,
            max_connections: 100,
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct PeerConnection {
    peer_id: String,
    address: String,
    connected_at: std::time::Instant,
    last_seen: std::time::Instant,
    bytes_sent: u64,
    bytes_received: u64,
}
pub struct NetworkServer {
    config: Arc<ServerConfig>,
    listener: Option<Arc<TcpListener>>,
    peers: Arc<RwLock<HashMap<String, PeerConnection>>>,
    running: Arc<RwLock<bool>>,
}

impl NetworkServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config: Arc::new(config),
            listener: None,
            peers: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn start(&mut self) -> Result<(), SystemError> {
        let mut running = self.running.write().await;
        if *running {
            return Err(SystemError::new(
                SystemErrorType::InvalidOperation,
                "Server is already running".to_string(),
            ));
        }

        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        match TcpListener::bind(&addr).await {
            Ok(listener) => {
                self.listener = Some(Arc::new(listener));
                *running = true;
                self.accept_connections().await?;
                Ok(())
            }
            Err(e) => Err(SystemError::new(
                SystemErrorType::NetworkError,
                format!("Failed to bind server: {}", e),
            )),
        }
    }

    pub async fn stop(&mut self) -> Result<(), SystemError> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }

        // Close all peer connections
        let mut peers = self.peers.write().await;
        peers.clear();
        
        // Reset listener
        self.listener = None;
        *running = false;

        Ok(())
    }

    pub async fn broadcast_message(&self, _message: &[u8]) -> Result<(), SystemError> {
        let peers = self.peers.read().await;
        if peers.is_empty() {
            return Err(SystemError::new(
                SystemErrorType::NetworkError,
                "No connected peers".to_string(),
            ));
        }

        // In a real implementation, you would iterate through peers and send the message
        // This is a placeholder for the actual broadcast logic
        Ok(())
    }
    pub async fn get_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    pub async fn get_server_stats(&self) -> ServerStats {
        let peers = self.peers.read().await;
        let running = *self.running.read().await;

        ServerStats {
            running,
            peer_count: peers.len(),
            network: self.config.network,
            bind_address: self.config.bind_address.clone(),
            port: self.config.port,
            total_bytes_sent: peers.values().map(|p| p.bytes_sent).sum(),
            total_bytes_received: peers.values().map(|p| p.bytes_received).sum(),
        }
    }

    // Private helper methods
    async fn accept_connections(&self) -> Result<(), SystemError> {
        let listener = match &self.listener {
            Some(l) => l.clone(),
            None => return Err(SystemError::new(
                SystemErrorType::NetworkError,
                "Server listener not initialized".to_string(),
            )),
        };

        loop {
            if !*self.running.read().await {
                break;
            }

            match listener.accept().await {
                Ok((socket, addr)) => {
                    let peer_id = format!("{}", addr);
                    let mut peers = self.peers.write().await;
                    
                    if peers.len() >= self.config.max_connections {
                        continue; // Connection limit reached
                    }

                    peers.insert(peer_id.clone(), PeerConnection {
                        peer_id,
                        address: addr.to_string(),
                        connected_at: std::time::Instant::now(),
                        last_seen: std::time::Instant::now(),
                        bytes_sent: 0,
                        bytes_received: 0,
                    });

                    // Spawn a task to handle this connection
                    self.handle_connection(socket);
                }
                Err(e) => {
                    if *self.running.read().await {
                        eprintln!("Accept error: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_connection(&self, socket: tokio::net::TcpStream) {
        // Spawn a new task for handling the connection
        let peers = self.peers.clone();
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let peer_addr = socket.peer_addr().ok();
            if let Some(addr) = peer_addr {
                let peer_id = format!("{}", addr);
                
                // Connection handling loop
                while *running.read().await {
                    // Handle incoming messages
                    // Update peer stats
                    // Check timeouts
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }

                // Clean up peer connection
                let mut peers = peers.write().await;
                peers.remove(&peer_id);
            }
        });
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerStats {
    pub running: bool,
    pub peer_count: usize,
    pub network: Network,
    pub bind_address: String,
    pub port: u16,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_server_lifecycle() {
        let config = ServerConfig::default();
        let mut server = NetworkServer::new(config);

        // Test server start
        let start_result = server.start().await;
        assert!(start_result.is_ok());

        let stats = server.get_server_stats().await;
        assert!(stats.running);
        assert_eq!(stats.peer_count, 0);

        // Test server stop
        let stop_result = server.stop().await;
        assert!(stop_result.is_ok());

        let stats = server.get_server_stats().await;
        assert!(!stats.running);
    }

    #[test]
    async fn test_peer_management() {
        let config = ServerConfig {
            max_connections: 2,
            ..Default::default()
        };
        let server = NetworkServer::new(config);

        assert_eq!(server.get_peer_count().await, 0);

        // In a real test, you would connect actual peers here
        // For now, we just verify the initial state
        let stats = server.get_server_stats().await;
        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.total_bytes_sent, 0);
        assert_eq!(stats.total_bytes_received, 0);
    }

    #[test]
    async fn test_broadcast_message() {
        let config = ServerConfig::default();
        let server = NetworkServer::new(config);

        // Test broadcast with no peers
        let result = server.broadcast_message(b"test message").await;
        assert!(result.is_err());

        // In a real test, you would:
        // 1. Connect some peers
        // 2. Broadcast a message
        // 3. Verify the message was received by all peers
    }
}