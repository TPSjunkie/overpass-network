use std::error::Error as StdError;
use std::fmt;
use crate::network::client_side::NetworkError;

#[derive(Debug)]
pub enum Error {
    NetworkError(String),
    DeserializationError(String),
    InvalidStateError(String),
    SystemError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Error::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            Error::InvalidStateError(msg) => write!(f, "Invalid state: {}", msg),
            Error::SystemError(msg) => write!(f, "System error: {}", msg),
        }
    }
}

impl StdError for Error {}

impl From<NetworkError> for Error {
    fn from(err: NetworkError) -> Self {
        match err {
            NetworkError::ConnectionFailed(msg) => Error::NetworkError(msg),
            NetworkError::DeserializationError(msg) => Error::DeserializationError(msg),
            NetworkError::InvalidResponse(msg) => Error::NetworkError(msg),
            NetworkError::InvalidRequest(msg) => Error::NetworkError(msg),
            NetworkError::AuthenticationFailed => Error::NetworkError("Authentication failed".to_string()),
            NetworkError::Timeout(_) => Error::NetworkError("Network timeout".to_string()),
            NetworkError::InvalidConfiguration(_) => Error::NetworkError("Invalid configuration".to_string()),
            _ => Error::NetworkError("Unknown network error".to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;