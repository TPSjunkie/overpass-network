// ./src/network/bitcoin_cs.rs

/// Represents a Bitcoin Cash network connection
/// Implements the ClientSideNetworkConnection trait
pub struct BitcoinCS {
    /// The URL of the server
    pub url: String,
    /// The port of the server
    pub port: u16,
    /// The username of the server
    pub username: String,
    /// The password of the server
    pub password: String,
}
impl BitcoinCS {
    /// Create a new Bitcoin Cash network connection
    pub fn new(url: String, port: u16, username: String, password: String) -> Self {
        BitcoinCS {
            url,
            port,
            username,
            password,
        }
    }
}
