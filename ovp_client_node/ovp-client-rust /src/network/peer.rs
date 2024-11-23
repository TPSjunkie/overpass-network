use crate::network::client_side::NetworkError;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::common::error::client_errors::{SystemErrorType::NetworkError as OtherNetworkError, Result as NetworkResult};
use crate::common::types::state_boc::StateInit;
use crate::core::client::channel::channel_contract::Transaction;
use crate::network::traits::NetworkConnection;


#[derive(Serialize, Deserialize, Debug)]
pub struct GetStateInitParams {
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetStateInitResult {
    pub state_init: StateInit,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransactionParams {
    pub transaction_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransactionResult {
    pub transaction: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransactionsParams {
    pub address: String,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransactionsResult {
    pub transactions: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBalanceParams {
    pub address: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBalanceResult {
    pub balance: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetChannelParams {
    pub channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetChannelResult {
    pub channel: serde_json::Value,
}

pub struct PeerApi {
    client: Client,
    url: String,
}

impl PeerApi {
    pub fn new(url: String) -> Self {
        Self {
            client: Client::new(),
            url,
        }
    }

    pub async fn get_state_init(&self, address: String) -> NetworkResult<StateInit> {
        let params = GetStateInitParams { address };
        let response = self
            .client
            .post(format!("{}/state_init", self.url))
            .json(params)   
            .send()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        let result: GetStateInitResult = response
            .json()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        Ok(result.state_init)
    }

    pub async fn get_transaction(&self, transaction_id: String) -> NetworkResult<Transaction> {
        let params = GetTransactionParams { transaction_id };
        let response = self
            .client
            .post(format!("{}/transaction", self.url))
            .json(¶ms)
            .send()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        let result: GetTransactionResult = response
            .json()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        Ok(result.transaction)
    }

    pub async fn get_transactions(
        &self,
        address: String,
        limit: Option<u32>,
    ) -> NetworkResult<Vec<Transaction>> {
        let params = GetTransactionsParams { address, limit };
        let response = self
            .client
            .post(format!("{}/transactions", self.url))
            .json(¶ms)
            .send()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        let result: GetTransactionsResult = response
            .json()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        Ok(result.transactions)
    }

    pub async fn get_balance(&self, address: String) -> NetworkResult<u64> {
        let params = GetBalanceParams { address };
        let response = self
            .client
            .post(format!("{}/balance", self.url))
            .json(¶ms)
            .send()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        let result: GetBalanceResult = response
            .json()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        Ok(result.balance)
    }

    pub async fn get_channel(&self, channel_id: String) -> NetworkResult<serde_json::Value> {
        let params = GetChannelParams { channel_id };
        let response = self
            .client
            .post(format!("{}/channel", self.url))
            .json(¶ms)
            .send()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        let result: GetChannelResult = response
            .json()
            .await
            .map_err(|e| NetworkError::Other(e.to_string()))?;
        Ok(result.channel)
    }
}
pub struct Peer {
    api: PeerApi,
    url: String,
}

impl Peer {
    pub fn new(url: String) -> Self {
        Self {
            api: PeerApi::new(url.clone()),
            url,
        }
    }
}

#[async_trait]
impl NetworkConnection for Peer {
    async fn get_state_init(&self, address: String) -> NetworkResult<StateInit> {
        self.api.get_state_init(address).await
    }

    async fn get_transaction(&self, transaction_id: String) -> NetworkResult<Transaction> {
        self.api.get_transaction(transaction_id).await
    }

    async fn get_transactions(
        &self,
        address: String,
        limit: Option<u32>,
    ) -> NetworkResult<Vec<Transaction>> {
        self.api.get_transactions(address, limit).await
    }

    async fn get_balance(&self, address: String) -> NetworkResult<u64> {
        self.api.get_balance(address).await
    }

    async fn get_blockchain_info(&self) -> NetworkResult<serde_json::Value> {
        // For peer connections, we'll use the channel info as blockchain info
        let channel_id = "default".to_string(); // Can be parameterized if needed
        self.api.get_channel(channel_id).await
    }

    fn get_server_address(&self) -> String {
        self.url.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;

    #[tokio::test]
    async fn test_get_balance() {
        let mock = mock("POST", "/balance")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"balance": 100}"#)
            .create();

        let peer = Peer::new(mockito::server_url());
        let balance = peer.get_balance("test_address".to_string()).await.unwrap();

        assert_eq!(balance, 100);
        mock.assert();
    }

    #[tokio::test]
    async fn test_get_channel() {
        let mock = mock("POST", "/channel")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"channel": {"id": "test_channel"}}"#)
            .create();

        let peer_api = PeerApi::new(mockito::server_url());
        let channel = peer_api.get_channel("test_channel".to_string()).await.unwrap();

        assert_eq!(channel["id"], "test_channel");
        mock.assert();
    }
}