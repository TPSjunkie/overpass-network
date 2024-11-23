// ./src/network/api.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::common::error::client_errors::{SystemError, SystemErrorType};
use crate::network::client_side::{ClientSideNetworkConnection, NetworkConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequest {
    pub method: String,
    pub params: Vec<serde_json::Value>,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub result: Option<serde_json::Value>,
    pub error: Option<ApiError>,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub struct NetworkApi {
    connection: Arc<ClientSideNetworkConnection>,
    timeout: Duration,
    request_id: Arc<RwLock<u64>>,
}

impl NetworkApi {
    pub fn new(config: NetworkConfig) -> Self {
        Self {
            connection: Arc::new(ClientSideNetworkConnection::new(config)),
            timeout: Duration::from_secs(30),
            request_id: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn connect(&self) -> Result<(), SystemError> {
        self.connection.connect().await
    }

    pub async fn disconnect(&self) -> Result<(), SystemError> {
        self.connection.disconnect().await
    }

    pub async fn send_request(&self, method: &str, params: Vec<serde_json::Value>) -> Result<ApiResponse, SystemError> {
        if !self.connection.is_connected().await {
            return Err(SystemError::new(
                SystemErrorType::NetworkError,
                "Not connected to network".to_string(),
            ));
        }

        let request_id = {
            let mut id = self.request_id.write().await;
            *id += 1;
            *id
        };

        let request = ApiRequest {
            method: method.to_string(),
            params,
            id: request_id,
        };

        self.send_and_receive(request).await
    }

    pub async fn send_batch_requests(&self, requests: Vec<(String, Vec<serde_json::Value>)>) -> Result<Vec<ApiResponse>, SystemError> {
        if !self.connection.is_connected().await {
            return Err(SystemError::new(
                SystemErrorType::NetworkError,
                "Not connected to network".to_string(),
            ));
        }

        let mut api_requests = Vec::with_capacity(requests.len());
        let mut request_id = self.request_id.write().await;

        for (method, params) in requests {
            *request_id += 1;
            api_requests.push(ApiRequest {
                method,
                params,
                id: *request_id,
            });
        }

        self.send_and_receive_batch(api_requests).await
    }

    pub async fn get_network_status(&self) -> Result<NetworkStatus, SystemError> {
        let connection_status = self.connection.get_connection_status().await;
        let config = self.connection.get_config();

        Ok(NetworkStatus {
            connected: connection_status.is_connected,
            network: config.network,
            url: config.url.clone(),
            port: config.port,
            retry_count: connection_status.retry_count,
            last_error: connection_status.last_error,
        })
    }

    // Private helper methods
    async fn send_and_receive(&self, request: ApiRequest) -> Result<ApiResponse, SystemError> {
        // Implement actual API request/response handling
        // This is a placeholder implementation
        Ok(ApiResponse {
            result: Some(serde_json::json!({"status": "success"})),
            error: None,
            id: request.id,
        })
    }

    async fn send_and_receive_batch(&self, requests: Vec<ApiRequest>) -> Result<Vec<ApiResponse>, SystemError> {
        // Implement batch request handling
        let mut responses = Vec::with_capacity(requests.len());
        
        for request in requests {
            match self.send_and_receive(request).await {
                Ok(response) => responses.push(response),
                Err(e) => return Err(e),
            }
        }

        Ok(responses)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkStatus {
    pub connected: bool,
    pub network: bitcoin::Network,
    pub url: String,
    pub port: u16,
    pub retry_count: u32,
    pub last_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn test_api_connection() {
        let config = NetworkConfig::default();
        let api = NetworkApi::new(config);

        assert!(api.connect().await.is_ok());
        
        let status = api.get_network_status().await.unwrap();
        assert!(status.connected);
        
        assert!(api.disconnect().await.is_ok());
    }

    #[test]
    async fn test_api_request() {
        let config = NetworkConfig::default();
        let api = NetworkApi::new(config);

        api.connect().await.unwrap();

        let response = api.send_request(
            "test_method",
            vec![serde_json::json!({"test": "value"})],
        ).await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    async fn test_batch_requests() {
        let config = NetworkConfig::default();
        let api = NetworkApi::new(config);

        api.connect().await.unwrap();

        let requests = vec![
            ("method1".to_string(), vec![serde_json::json!({"param1": "value1"})]),
            ("method2".to_string(), vec![serde_json::json!({"param2": "value2"})]),
        ];

        let responses = api.send_batch_requests(requests).await;
        assert!(responses.is_ok());
        assert_eq!(responses.unwrap().len(), 2);
    }
}