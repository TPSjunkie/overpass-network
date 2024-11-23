// ./src/network/client_side.rs

use crate::common::error::client_errors::{Error, SystemError, SystemErrorType};
use bitcoincore_rpc::bitcoin::Network;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub url: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub network: Network,
    pub timeout: u64,
    pub max_retries: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost".to_string(),
            port: 8332,
            username: "user".to_string(),
            password: "pass".to_string(),
            network: Network::Bitcoin,
            timeout: 30,
            max_retries: 3,
        }
    }
}

#[derive(Clone)]
pub struct ClientSideNetworkConnection {
    config: Arc<NetworkConfig>,
    connection_state: Arc<RwLock<ConnectionState>>,
}

#[derive(Debug)]
pub struct ConnectionState {
    is_connected: bool,
    retry_count: u32,
    last_error: Option<String>,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            is_connected: false,
            retry_count: 0,
            last_error: None,
        }
    }
}

impl ClientSideNetworkConnection {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            config: Arc::new(config),
            connection_state: Arc::new(RwLock::new(ConnectionState::new())),
        }
    }

    pub async fn connect(&self) -> Result<(), SystemError> {
        let mut state = self.connection_state.write().await;
        if state.is_connected {
            return Ok(());
        }

        let url = format!("{}:{}", self.config.url, self.config.port);
        if !Self::validate_url(&url) {
            return Err(SystemError::new(
                SystemErrorType::NetworkError,
                "Invalid URL format".to_string(),
            ));
        }

        // Attempt connection with retry logic
        for attempt in 0..self.config.max_retries {
            match self.try_connect().await {
                Ok(_) => {
                    state.is_connected = true;
                    state.retry_count = 0;
                    state.last_error = None;
                    return Ok(());
                }
                Err(e) => {
                    state.retry_count = attempt + 1;
                    state.last_error = Some(e.to_string());
                    
                    if attempt + 1 == self.config.max_retries {
                        return Err(SystemError::new(
                            SystemErrorType::NetworkError,
                            format!("Failed to connect after {} attempts: {}", attempt + 1, e),
                        ));
                    }
                    
                    tokio::time::sleep(tokio::time::Duration::from_secs(1 << attempt)).await;
                }
            }
        }

        Err(SystemError::new(
            SystemErrorType::NetworkError,
            "Connection attempts exhausted".to_string(),
        ))
    }

    pub async fn disconnect(&self) -> Result<(), SystemError> {
        let mut state = self.connection_state.write().await;
        if !state.is_connected {
            return Ok(());
        }

        // Perform cleanup and disconnection
        state.is_connected = false;
        state.retry_count = 0;
        state.last_error = None;

        Ok(())
    }

    pub async fn is_connected(&self) -> bool {
        self.connection_state.read().await.is_connected
    }

    pub async fn get_connection_status(&self) -> ConnectionStatus {
        let state = self.connection_state.read().await;
        ConnectionStatus {
            is_connected: state.is_connected,
            retry_count: state.retry_count,
            last_error: state.last_error.clone(),
            network: self.config.network,
        }
    }

    pub fn get_config(&self) -> Arc<NetworkConfig> {
        self.config.clone()
    }

    // Private helper methods
    async fn try_connect(&self) -> Result<(), SystemError> {
        // Implement actual connection logic here
        // This could include setting up TLS, authentication, etc.
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        Ok(())
    }

    fn validate_url(url: &str) -> bool {
        match Url::parse(url) {
            Ok(parsed_url) => {
                parsed_url.scheme() == "http" || parsed_url.scheme() == "https"
            }
            Err(_) => false,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionStatus {
    pub is_connected: bool,
    pub retry_count: u32,
    pub last_error: Option<String>,
    pub network: Network,
}

// Error types specific to network operations
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Network timeout after {0} seconds")]
    Timeout(u64),
    
    #[error("Invalid network configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_connection_lifecycle() {
        let config = NetworkConfig::default();
        let connection = ClientSideNetworkConnection::new(config);

        // Test initial state
        assert!(!connection.is_connected().await);

        // Test successful connection
        assert!(connection.connect().await.is_ok());
        assert!(connection.is_connected().await);

        // Test disconnection
        assert!(connection.disconnect().await.is_ok());
        assert!(!connection.is_connected().await);
    }

    #[test]
    async fn test_connection_status() {
        let config = NetworkConfig::default();
        let connection = ClientSideNetworkConnection::new(config);

        let status = connection.get_connection_status().await;
        assert!(!status.is_connected);
        assert_eq!(status.retry_count, 0);
        assert!(status.last_error.is_none());
    }

    #[test]
    fn test_url_validation() {
        assert!(ClientSideNetworkConnection::validate_url("http://localhost:8332"));
        assert!(ClientSideNetworkConnection::validate_url("https://example.com:8333"));
        assert!(!ClientSideNetworkConnection::validate_url("invalid-url"));
    }
}