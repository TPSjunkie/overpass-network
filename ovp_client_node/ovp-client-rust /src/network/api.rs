use crate::common::types::state_boc::StateInit;
use crate::core::client::channel::channel_contract::Transaction;
use crate::common::error::client_errors::{Result as NetworkResult, Error as NetworkError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
#[derive(Serialize, Deserialize)]
pub struct GetTransactionResult {
    #[serde(with = "channel_contract_transaction_serde")]
    pub transaction: Transaction,
}

impl std::fmt::Debug for GetTransactionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetTransactionResult")
            .field("transaction", &"Transaction")
            .finish()
    }
}
mod channel_contract_transaction_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(_transaction: &Transaction, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Implement custom serialization for Transaction
        // This is a placeholder implementation. You need to replace it with actual serialization logic.
        unimplemented!("Implement custom serialization for Transaction")
    }

    pub fn deserialize<'de, D>(_deserializer: D) -> Result<Transaction, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Implement custom deserialization for Transaction
        // This is a placeholder implementation. You need to replace it with actual deserialization logic.
        unimplemented!("Implement custom deserialization for Transaction")
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransactionsParams {
    pub address: String,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransactionsResult {
    #[serde(with = "channel_contract_transactions_serde")]
    pub transactions: Vec<Transaction>,
}

mod channel_contract_transactions_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(transactions: &Vec<Transaction>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Implement custom serialization for Vec<Transaction>
        // This is a placeholder implementation. You need to replace it with actual serialization logic.
        unimplemented!("Implement custom serialization for Vec<Transaction>")
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Transaction>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Implement custom deserialization for Vec<Transaction>
        // This is a placeholder implementation. You need to replace it with actual deserialization logic.
        unimplemented!("Implement custom deserialization for Vec<Transaction>")
    }
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
pub struct GetBlockchainInfoParams {}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBlockchainInfoResult {
    pub blockchain_info: serde_json::Value,
}

pub struct Api {
    client: Client,
    cs_url: String,
    ssn_url: String,
}

impl Api {
    pub fn new(cs_url: String, ssn_url: String) -> Self {
        Api {
            client: Client::new(),
            cs_url,
            ssn_url,
        }
    }

    pub async fn get_state_init(&self, address: String) -> NetworkResult<StateInit> {
        let params = GetStateInitParams { address };
        let response = self
            .client
            .post(self.cs_url.clone())
            .json(¶ms)
            .send()
            .await
            .map_err(|e| NetworkError::RequestError(e.to_string()))?;
        let result: GetStateInitResult = response.json().await
            .map_err(|e| NetworkError::DeserializationError(e.to_string()))?;
        Ok(result.state_init)
    }

    pub async fn get_transaction(&self, transaction_id: String) -> NetworkResult<Transaction> {
        let params = GetTransactionParams { transaction_id };
        let response = self
            .client
            .post(self.cs_url.clone())
            .json(¶ms)
            .send()
            .await
            .map_err(|e| NetworkError::RequestError(e.to_string()))?;
        let result: GetTransactionResult = response.json().await
            .map_err(|e| NetworkError::DeserializationError(e.to_string()))?;
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
            .post(self.cs_url.clone())
            .json(¶ms)
            .send()
            .await
            .map_err(|e| NetworkError::RequestError(e.to_string()))?;
        let result: GetTransactionsResult = response.json().await
            .map_err(|e| NetworkError::DeserializationError(e.to_string()))?;
        Ok(result.transactions)
    }

    pub async fn get_balance(&self, address: String) -> NetworkResult<u64> {
        let params = GetBalanceParams { address };
        let response = self
            .client
            .post(self.cs_url.clone())
            .json(¶ms)
            .send()
            .await
            .map_err(|e| NetworkError::RequestError(e.to_string()))?;
        let result: GetBalanceResult = response.json().await
            .map_err(|e| NetworkError::DeserializationError(e.to_string()))?;
        Ok(result.balance)
    }

    pub async fn get_blockchain_info(&self) -> NetworkResult<serde_json::Value> {
        let response = self
            .client
            .post(self.cs_url.clone())
            .json(&json!({}))
            .send()
            .await
            .map_err(|e| NetworkError::RequestError(e.to_string()))?;
        let result: GetBlockchainInfoResult = response.json().await
            .map_err(|e| NetworkError::DeserializationError(e.to_string()))?;
        Ok(result.blockchain_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::MockServer;
    use wiremock::{Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_get_balance() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"balance": 100})))
            .mount(&mock_server)
            .await;

        let api = Api::new(mock_server.uri(), String::new());
        let balance = api.get_balance("some_address".to_string()).await.unwrap();

        assert_eq!(balance, 100);
    }
}