use async_trait::async_trait;
use crate::core::client::channel::channel_contract::Transaction;
use crate::network::traits::NetworkConnection;
use crate::network::api::Api;
use crate::common::error::client_errors::Result as NetworkResult;
use crate::common::types::state_boc::StateInit;

pub struct BitcoinCS {
    url: String,
    port: u16,
    username: String,
    password: String,
    api: Api,
}

impl BitcoinCS {
    pub fn new(url: String, port: u16, username: String, password: String) -> Self {
        let api = Api::new(
            format!("{}:{}", url.clone(), port),
            String::new(), // Empty SSN URL as not needed for Bitcoin CS
        );

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
    async fn get_state_init(&self, address: String) -> NetworkResult<StateInit> {
        self.api.get_state_init(address).await
    }

    async fn get_transaction(&self, transaction_id: String) -> NetworkResult<Transaction> {
        self.api.get_transaction(transaction_id).await
    }

    async fn get_transactions(&self, address: String, limit: Option<u32>) -> NetworkResult<Vec<Transaction>> {
        self.api.get_transactions(address, limit).await
    }

    async fn get_balance(&self, address: String) -> NetworkResult<u64> {
        self.api.get_balance(address).await
    }

    async fn get_blockchain_info(&self) -> NetworkResult<serde_json::Value> {
        self.api.get_blockchain_info().await
    }
    
    fn get_server_address(&self) -> String {
        format!("{}:{}", self.url, self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
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