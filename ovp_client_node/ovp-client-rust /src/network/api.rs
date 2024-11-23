pub mod network {
    pub mod client_side {
        #[derive(Clone, Debug)]
        pub struct NetworkConfig {
            pub url: String,
            pub username: String,
            pub password: String,
            pub network: String,
            pub port: u16,
        }

        impl Default for NetworkConfig {
            fn default() -> Self {
                Self {
                    url: "localhost".to_string(),
                    username: "user".to_string(),
                    password: "pass".to_string(),
                    network: "bitcoin".to_string(),
                    port: 8332,
                }
            }
        }

        #[derive(Debug, Clone)]
        pub struct ConnectionStatus {
            pub is_connected: bool,
            pub retry_count: u32,
            pub last_error: Option<String>,
        }

        #[derive(Clone)]
        pub struct ClientSideNetworkConnection {
            config: NetworkConfig,
            status: ConnectionStatus,
        }

        impl ClientSideNetworkConnection {
            pub fn new(config: NetworkConfig) -> Self {
                Self {
                    config,
                    status: ConnectionStatus {
                        is_connected: false,
                        retry_count: 0,
                        last_error: None,
                    },
                }
            }

            pub async fn connect(&self) -> Result<(), crate::common::error::client_errors::SystemError> {
                Ok(())
            }

            pub async fn disconnect(&self) -> Result<(), crate::common::error::client_errors::SystemError> {
                Ok(())
            }

            pub async fn is_connected(&self) -> bool {
                true
            }

            pub async fn get_connection_status(&self) -> ConnectionStatus {
                self.status.clone()
            }

            pub fn get_config(&self) -> NetworkConfig {
                self.config.clone()
            }
        }

        #[derive(Clone)]
        pub struct NetworkApi {
            connection: ClientSideNetworkConnection,
        }

        impl NetworkApi {
            pub fn new(config: NetworkConfig) -> Self {
                Self {
                    connection: ClientSideNetworkConnection::new(config),
                }
            }
        }
    }
}