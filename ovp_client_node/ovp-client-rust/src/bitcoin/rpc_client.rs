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
    #[allow(dead_code)]
    inner: bitcoincore_rpc::Client,
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
            Ok(Self { inner: client })
        }
    }
}