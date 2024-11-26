use async_trait::async_trait;
use crate::core::client::channel::channel_contract::Transaction;
use crate::common::error::client_errors::Result as NetworkResult;
use crate::common::types::state_boc::StateInit;
use serde_json::Value;

#[async_trait]
pub trait NetworkConnection {
    async fn get_state_init(&self, address: String) -> NetworkResult<StateInit>;
    async fn get_transaction(&self, transaction_id: String) -> NetworkResult<Transaction>;
    async fn get_transactions(&self, address: String, limit: Option<u32>) -> NetworkResult<Vec<Transaction>>;
    async fn get_balance(&self, address: String) -> NetworkResult<u64>;
    async fn get_blockchain_info(&self) -> NetworkResult<Value>;
    
    fn get_server_address(&self) -> String;
}