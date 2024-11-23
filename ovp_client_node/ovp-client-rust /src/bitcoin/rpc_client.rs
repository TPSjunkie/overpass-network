use bitcoin::Address;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct BitcoinRpcConfig {
    pub url: String,
    pub user: String,
    pub password: String,
}

#[cfg(target_arch = "wasm32")]
pub struct BitcoinRpcClient {
    config: BitcoinRpcConfig,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct BitcoinRpcClient {
    inner: bitcoincore_rpc::Client,
    config: BitcoinRpcConfig,
}

impl BitcoinRpcClient {
    pub fn new(config: BitcoinRpcConfig) -> Result<Self, Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        {
            Ok(Self { config })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let client = bitcoincore_rpc::Client::new(
                &config.url,
                bitcoincore_rpc::Auth::UserPass(config.user.clone(), config.password.clone()),
            )?;
            Ok(Self { inner: client, config })
        }
    }

    pub async fn get_balance(&self, address: &Address) -> Result<u64, Box<dyn std::error::Error>> {
        #[cfg(target_arch = "wasm32")]
        {
            // Use web APIs to make RPC call
            let response = reqwest::Client::new()
                .post(&self.config.url)
                .basic_auth(&self.config.user, Some(&self.config.password))
                .json(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "1",
                    "method": "getbalance",
                    "params": [address.to_string()]
                }))
                .send()
                .await?;
            
            let result: serde_json::Value = response.json().await?;
            Ok(result["result"].as_u64().unwrap_or(0))
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            Ok(self.inner.get_balance()?.to_sat())
        }
    }

    // Add other RPC methods similarly...
}