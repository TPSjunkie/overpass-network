// ./src/network/client_side.rs

/// Represents a client-side network connection
pub struct ClientSideNetworkConnection {
    /// The URL of the server
    pub url: String,
    /// The port of the server
    pub port: u16,
    /// The username of the server
    pub username: String,
    /// The password of the server
    pub password: String,
}
