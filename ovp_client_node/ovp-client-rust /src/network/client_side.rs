use std::io::{Read, Write};
use std::net::{TcpStream, Shutdown};
use async_trait::async_trait;
use crate::common::error::client_errors::Result as NetworkResult;
use crate::network::traits::NetworkConnection;
use crate::common::types::state_boc::StateInit;
use crate::core::client::channel::channel_contract::Transaction;
use crate::network::api::Api;

pub struct ClientSideNetworkConnection {
    url: String,
    port: u16,
    username: String,
    password: String,
    api: Api,
}

impl ClientSideNetworkConnection {
    pub fn new(url: String, port: u16, username: String, password: String) -> Self {
        let api = Api::new(
            format!("{}:{}", url.clone(), port),
            String::new(), // Empty SSN URL as not needed for client side
        );
        
        Self {
            url,
            port,
            username,
            password,
            api,
        }
    }

    pub fn connect(&self) -> std::io::Result<()> {
        let address = self.get_server_address();
        let mut stream = TcpStream::connect(address)?;
        
        let auth_data = format!("{}:{}", self.username, self.password);
        stream.write_all(auth_data.as_bytes())?;
        
        Ok(())
    }

    pub fn disconnect(&self) -> std::io::Result<()> {
        let address = self.get_server_address();
        let stream = TcpStream::connect(address)?;
        stream.shutdown(Shutdown::Both)
    }

    pub fn send_data(&self, data: &[u8]) -> std::io::Result<()> {
        let address = self.get_server_address();
        let mut stream = TcpStream::connect(address)?;
        stream.write_all(data)?;
        stream.flush()
    }

    pub fn receive_data(&self) -> std::io::Result<Vec<u8>> {
        let address = self.get_server_address();
        let mut stream = TcpStream::connect(address)?;
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    pub fn update_credentials(&mut self, username: String, password: String) {
        self.username = username;
        self.password = password;
    }
}

#[async_trait]
impl NetworkConnection for ClientSideNetworkConnection {
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