use async_trait::async_trait;
use crate::core::client::channel::channel_contract::Transaction;
use crate::network::traits::NetworkConnection;
use crate::common::error::client_errors::Result as NetworkResult;
use crate::common::types::state_boc::StateInit;

use super::api::network::client_side::{NetworkApi, NetworkConfig};
pub struct BitcoinCS {
    url: String,
    port: u16,
    username: String,
    password: String,
    api: NetworkApi,
}

impl BitcoinCS {
    pub fn new(url: String, port: u16, username: String, password: String) -> Self {
        let config = NetworkConfig {
            url: format!("{}:{}", url.clone(), port),
            username: username.clone(),
            password: password.clone(),
            network: "bitcoin".to_string(),
            port,
        };
        let api = NetworkApi::new(config);

        Self {
            url,
            port,
            username,
            password,
            api,
        }
    }
}

#[async_trait]
impl NetworkConnection for BitcoinCS {
    async fn get_state_init(&self, _address: String) -> NetworkResult<StateInit> {
        // Implement the method or delegate to a proper implementation
        unimplemented!()
    }

    async fn get_transaction(&self, _transaction_id: String) -> NetworkResult<Transaction> {
        // Implement the method or delegate to a proper implementation
        unimplemented!()
    }

    async fn get_transactions(&self, _address: String, _limit: Option<u32>) -> NetworkResult<Vec<Transaction>> {
        // Implement the method or delegate to a proper implementation
        unimplemented!()
    }

    async fn get_balance(&self, _address: String) -> NetworkResult<u64> {
        // Implement the method or delegate to a proper implementation
        unimplemented!()
    }

    async fn get_blockchain_info(&self) -> NetworkResult<serde_json::Value> {
        // Implement the method or delegate to a proper implementation
        unimplemented!()
    }
    
    fn get_server_address(&self) -> String {
        format!("{}:{}", self.url, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg_attr(test, tokio::test)]
    async fn test_bitcoin_cs_connection() {
        let cs = BitcoinCS::new(
            "localhost".to_string(),
            8332,
            "user".to_string(),
            "pass".to_string(),
        );

        assert_eq!(cs.get_server_address(), "localhost:8332");
    }
}