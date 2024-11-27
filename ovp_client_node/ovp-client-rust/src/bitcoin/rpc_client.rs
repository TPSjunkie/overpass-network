use bitcoincore_rpc::RpcApi;
use std::fmt;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use bitcoin::{Transaction, Txid, Amount, Address, Network};

#[derive(Error, Debug)]
pub enum RpcError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("JSON-RPC error: {0}")]
    JsonRpcError(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[cfg(not(target_arch = "wasm32"))]
    #[error("Bitcoin Core RPC error: {0}")]
    BitcoinCoreError(#[from] bitcoincore_rpc::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinRpcConfig {
    pub url: String,
    pub user: String,
    pub password: String,
    pub network: Network,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_timeout() -> u64 {
    30
}

impl Default for BitcoinRpcConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8332".to_string(),
            user: "".to_string(),
            password: "".to_string(),
            network: Network::Bitcoin,
            timeout_seconds: default_timeout(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    jsonrpc: String,
    id: u64,
    result: Option<T>,
    error: Option<RpcResponseError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcResponseError {
    code: i32,
    message: String,
}

#[cfg(target_arch = "wasm32")]
pub struct BitcoinRpcClient {
    config: BitcoinRpcConfig,
    request_id: std::sync::atomic::AtomicU64,
}

#[cfg(not(target_arch = "wasm32"))]
pub struct BitcoinRpcClient {
    inner: bitcoincore_rpc::Client,
    config: BitcoinRpcConfig,
}

impl BitcoinRpcClient {
    pub fn new(config: BitcoinRpcConfig) -> Result<Self, RpcError> {
        #[cfg(target_arch = "wasm32")] {
            Ok(Self {
                config,
                request_id: std::sync::atomic::AtomicU64::new(1),
            })
        }

        #[cfg(not(target_arch = "wasm32"))] {
            let client = bitcoincore_rpc::Client::new(
                &config.url,
                bitcoincore_rpc::Auth::UserPass(config.user.clone(), config.password.clone()),
            ).map_err(RpcError::BitcoinCoreError)?;
            Ok(Self { inner: client, config })
        }
    }

    #[cfg(target_arch = "wasm32")]
    async fn make_request<T: for<'de> Deserialize<'de>>(&self, method: &str, params: Vec<serde_json::Value>) -> Result<T, RpcError> {
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, RequestMode, Response};

        let request = RpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.request_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            method: method.to_string(),
            params,
        };

        let mut opts = RequestInit::new();
        opts.method("POST")
            .mode(RequestMode::Cors)
            .body(Some(&JsValue::from_str(&serde_json::to_string(&request).map_err(|e| RpcError::InvalidRequest(e.to_string()))?)));

        let request = Request::new_with_str_and_init(&self.config.url, &opts)
            .map_err(|e| RpcError::InvalidRequest(format!("Failed to create request: {:?}", e)))?;

        request.headers().set("Content-Type", "application/json")
            .map_err(|e| RpcError::InvalidRequest(format!("Failed to set headers: {:?}", e)))?;

        let window = web_sys::window().ok_or_else(|| RpcError::NetworkError("No window object available".to_string()))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| RpcError::NetworkError(format!("Fetch failed: {:?}", e)))?;

        let resp: Response = resp_value.dyn_into()
            .map_err(|e| RpcError::InvalidResponse(format!("Invalid response type: {:?}", e)))?;

        let json = JsFuture::from(resp.json()
            .map_err(|e| RpcError::InvalidResponse(format!("Failed to parse JSON: {:?}", e)))?)
            .await
            .map_err(|e| RpcError::InvalidResponse(format!("JSON parsing failed: {:?}", e)))?;

        let rpc_response: RpcResponse<T> = serde_wasm_bindgen::from_value(json)
            .map_err(|e| RpcError::InvalidResponse(format!("Failed to deserialize response: {:?}", e)))?;

        match (rpc_response.result, rpc_response.error) {
            (Some(result), None) => Ok(result),
            (None, Some(error)) => Err(RpcError::JsonRpcError(error.message)),
            _ => Err(RpcError::InvalidResponse("Invalid response structure".to_string())),
        }
    }

    // Common interface methods that work in both environments
    pub async fn get_block_count(&self) -> Result<u64, RpcError> {
        #[cfg(target_arch = "wasm32")] {
            self.make_request("getblockcount", vec![]).await
        }

        #[cfg(not(target_arch = "wasm32"))] {
            Ok(self.inner.get_block_count().map_err(RpcError::BitcoinCoreError)?)
        }
    }

    pub async fn get_raw_transaction(&self, txid: &Txid) -> Result<Transaction, RpcError> {
        #[cfg(target_arch = "wasm32")] {
            self.make_request("getrawtransaction", vec![
                serde_json::Value::String(txid.to_string()),
                serde_json::Value::Bool(true),
            ]).await
        }

        #[cfg(not(target_get_raw_transaction_hex))] {
            self.inner.get_raw_transaction(txid, None)
                .map_err(RpcError::BitcoinCoreError)
        }
    }

    pub async fn get_balance(&self) -> Result<Amount, RpcError> {
        #[cfg(target_arch = "wasm32")] {
            let btc: f64 = self.make_request("getbalance", vec![]).await?;
            Amount::from_btc(btc).map_err(|e| RpcError::InvalidResponse(e.to_string()))
        }

        #[cfg(not(target_arch = "wasm32"))] {
            self.inner.get_balance(None, None)
                .map_err(RpcError::BitcoinCoreError)
        }
    }

    pub async fn send_raw_transaction(&self, tx: &Transaction) -> Result<Txid, RpcError> {
        #[cfg(target_arch = "wasm32")] {
            let tx_hex = bitcoin::consensus::encode::serialize_hex(tx);
            self.make_request("sendrawtransaction", vec![serde_json::Value::String(tx_hex)]).await
        }

        #[cfg(not(target_arch = "wasm32"))] {
            self.inner.send_raw_transaction(tx)
                .map_err(RpcError::BitcoinCoreError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rpc_client_creation() {
        let config = BitcoinRpcConfig {
            url: "http://127.0.0.1:8332".to_string(),
            user: "test".to_string(),
            password: "test".to_string(),
            network: Network::Regtest,
            timeout_seconds: 30,
        };

        let client = BitcoinRpcClient::new(config);
        assert!(client.is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[tokio::test]
    async fn test_native_rpc_calls() {
        let config = BitcoinRpcConfig {
            url: "http://127.0.0.1:18443".to_string(),
            user: "test".to_string(),
            password: "test".to_string(),
            network: Network::Regtest,
            timeout_seconds: 30,
        };

        let client = BitcoinRpcClient::new(config).unwrap();
        let block_count = client.get_block_count().await;
        assert!(block_count.is_ok());
    }
}