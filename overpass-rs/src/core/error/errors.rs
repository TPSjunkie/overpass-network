use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    InvalidProof,
    UnknownContract,
    InvalidTransaction,
    InvalidSignature,
    InvalidPublicKey,
    InvalidAddress,
    InvalidAmount,
    InvalidChannel,
    InvalidNonce,
    InvalidSequence,
    InvalidTimestamp,
    BatteryError,
    WalletError(String),
    InvalidProofData,
    InvalidProofDataLength,
    InvalidProofDataFormat,
    InvalidProofDataSignature,
    InvalidProofDataPublicKey,
    InvalidProofDataHash,
    StorageError(String),
    StakeError(String),
    NetworkError(String),
    ChargingTooFrequent,
    MaxChargingAttemptsExceeded,
    CellError(CellError),
    ZkProofError(ZkProofError),
    BocError(BocError),
    SystemError(SystemError),
    CustomError(String),
    SerializationError(String),
    DeserializationError(String),
    LockError(String),
    ChannelNotFound(String),
    StateNotFound(String),
    InvalidBOC(String),
    ArithmeticError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemErrorType {
    InvalidTransaction,
    InvalidSignature,
    InvalidPublicKey,
    InvalidAddress,
    InvalidHash,
    InvalidNonce,
    InvalidSequence,
    InvalidState,
    NoProof,
    InvalidReference,
    InvalidAmount,
    InvalidProof,
    InsufficientBalance,
    SpendingLimitExceeded,
    BatteryError,
    NoRootCell,
    InvalidOperation,
    NotFound,
}

impl fmt::Display for SystemErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => write!(f, "Invalid transaction"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::InvalidReference => write!(f, "Invalid reference"),
            Self::InvalidAddress => write!(f, "Invalid address"),
            Self::InvalidHash => write!(f, "Invalid hash"),
            Self::InvalidNonce => write!(f, "Invalid nonce"),
            Self::InvalidSequence => write!(f, "Invalid sequence"),
            Self::InvalidState => write!(f, "Invalid state"),
            Self::NoProof => write!(f, "No proof"),
            Self::InvalidAmount => write!(f, "Invalid amount"),
            Self::InvalidProof => write!(f, "Invalid proof"),
            Self::InsufficientBalance => write!(f, "Insufficient balance"),
            Self::SpendingLimitExceeded => write!(f, "Spending limit exceeded"),
            Self::BatteryError => write!(f, "Battery error"),
            Self::NoRootCell => write!(f, "No root cell"),
            Self::InvalidOperation => write!(f, "Invalid operation"),
            Self::NotFound => write!(f, "Not found"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SystemError {
    pub error_type: SystemErrorType,
    pub message: String,
}

impl SystemError {
    pub fn new(error_type: SystemErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
        }
    }

    pub fn error_type(&self) -> SystemErrorType {
        self.error_type
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}

impl std::error::Error for SystemError {}

impl From<SystemError> for Error {
    fn from(err: SystemError) -> Self {
        Error::SystemError(err)
    }
}

#[derive(Debug)]
pub enum CellError {
    DataTooLarge,
    TooManyReferences,
    InvalidData,
    IoError(io::Error),
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellError::DataTooLarge => write!(f, "Cell data is too large"),
            CellError::TooManyReferences => write!(f, "Too many references in cell"),
            CellError::InvalidData => write!(f, "Invalid cell data"),
            CellError::IoError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl From<io::Error> for CellError {
    fn from(err: io::Error) -> Self {
        CellError::IoError(err)
    }
}

// BatteryError is a custom error type for battery-related errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryError {
    InsufficientBattery,
    SpendingLimitExceeded,
}

#[derive(Debug)]
pub enum ZkProofError {
    InvalidProof,
    InvalidProofData,
    InvalidProofDataLength,
    InvalidProofDataFormat,
    InvalidProofDataSignature,
    InvalidProofDataPublicKey,
    InvalidProofDataHash,
}

impl fmt::Display for ZkProofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZkProofError::InvalidProof => write!(f, "Invalid proof"),
            ZkProofError::InvalidProofData => write!(f, "Invalid proof data"),
            ZkProofError::InvalidProofDataLength => write!(f, "Invalid proof data length"),
            ZkProofError::InvalidProofDataFormat => write!(f, "Invalid proof data format"),
            ZkProofError::InvalidProofDataSignature => write!(f, "Invalid proof data signature"),
            ZkProofError::InvalidProofDataPublicKey => write!(f, "Invalid proof data public key"),
            ZkProofError::InvalidProofDataHash => write!(f, "Invalid proof data hash"),
        }
    }
}

#[derive(Debug)]
pub enum BocError {
    TooManyCells,
    NoRoots,
    TotalSizeTooLarge,
    CellDataTooLarge,
    TooManyReferences,
    InvalidReference { from: usize, to: usize },
    InvalidRoot(usize),
    InvalidMerkleProof,
    InvalidPrunedBranch,
    SerializationError(String),
    DeserializationError(String),
    CycleDetected,
    MaxDepthExceeded,
}

impl fmt::Display for BocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BocError::TooManyCells => write!(f, "Too many cells"),
            BocError::NoRoots => write!(f, "No roots"),
            BocError::TotalSizeTooLarge => write!(f, "Total size too large"),
            BocError::CellDataTooLarge => write!(f, "Cell data too large"),
            BocError::TooManyReferences => write!(f, "Too many references"),
            BocError::InvalidReference { from, to } => {
                write!(f, "Invalid reference from {} to {}", from, to)
            }
            BocError::InvalidRoot(index) => write!(f, "Invalid root at index {}", index),
            BocError::InvalidMerkleProof => write!(f, "Invalid Merkle proof"),
            BocError::InvalidPrunedBranch => write!(f, "Invalid pruned branch"),
            BocError::CycleDetected => write!(f, "Cycle detected"),
            BocError::MaxDepthExceeded => write!(f, "Max depth exceeded"),
            BocError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            BocError::DeserializationError(err) => write!(f, "Deserialization error: {}", err),
        }
    }
}

impl std::error::Error for CellError {}
impl std::error::Error for ZkProofError {}
impl std::error::Error for BocError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(err) => write!(f, "IO error: {}", err),
            Error::InvalidProof => write!(f, "Invalid proof"),
            Error::UnknownContract => write!(f, "Unknown contract"),
            Error::InvalidTransaction => write!(f, "Invalid transaction"),
            Error::InvalidSignature => write!(f, "Invalid signature"),
            Error::InvalidPublicKey => write!(f, "Invalid public key"),
            Error::InvalidAddress => write!(f, "Invalid address"),
            Error::InvalidAmount => write!(f, "Invalid amount"),
            Error::InvalidChannel => write!(f, "Invalid channel"),
            Error::InvalidNonce => write!(f, "Invalid nonce"),
            Error::InvalidSequence => write!(f, "Invalid sequence"),
            Error::InvalidTimestamp => write!(f, "Invalid timestamp"),
            Error::BatteryError => write!(f, "Battery error"),
            Error::WalletError(err) => write!(f, "Wallet error: {}", err),
            Error::InvalidProofData => write!(f, "Invalid proof data"),
            Error::InvalidProofDataLength => write!(f, "Invalid proof data length"),
            Error::InvalidProofDataFormat => write!(f, "Invalid proof data format"),
            Error::InvalidProofDataSignature => write!(f, "Invalid proof data signature"),
            Error::InvalidProofDataPublicKey => write!(f, "Invalid proof data public key"),
            Error::InvalidProofDataHash => write!(f, "Invalid proof data hash"),
            Error::CustomError(msg) => write!(f, "Custom Error: {}", msg),
            Error::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            Error::DeserializationError(msg) => write!(f, "Deserialization Error: {}", msg),
            Error::LockError(msg) => write!(f, "Lock Error: {}", msg),
            Error::ChannelNotFound(msg) => write!(f, "Channel Not Found: {}", msg),
            Error::StateNotFound(msg) => write!(f, "State Not Found: {}", msg),
            Error::InvalidBOC(msg) => write!(f, "Invalid BOC: {}", msg),
            Error::ArithmeticError(msg) => write!(f, "Arithmetic Error: {}", msg),
            Error::StorageError(err) => write!(f, "Storage error: {}", err),
            Error::StakeError(err) => write!(f, "Stake error: {}", err),
            Error::NetworkError(err) => write!(f, "Network error: {}", err),
            Error::ChargingTooFrequent => write!(f, "Charging too frequent"),
            Error::MaxChargingAttemptsExceeded => write!(f, "Max charging attempts exceeded"),
            Error::CellError(err) => write!(f, "Cell error: {}", err),
            Error::ZkProofError(err) => write!(f, "ZK proof error: {}", err),
            Error::BocError(err) => write!(f, "BOC error: {}", err),
            Error::SystemError(err) => write!(f, "System error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

// From implementations
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
}

impl From<CellError> for Error {
    fn from(err: CellError) -> Self {
        Error::CellError(err)
    }
}

impl From<ZkProofError> for Error {
    fn from(err: ZkProofError) -> Self {
        Error::ZkProofError(err)
    }
}

impl From<BocError> for Error {
    fn from(err: BocError) -> Self {
        Error::BocError(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_error_display() {
        let error = SystemError::new(
            SystemErrorType::InvalidTransaction,
            "Transaction validation failed".to_string(),
        );
        assert_eq!(
            error.to_string(),
            "Invalid transaction: Transaction validation failed"
        );
    }

    #[test]
    fn test_error_conversions() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error: Error = io_err.into();
        assert!(matches!(error, Error::IoError(_)));

        let system_err = SystemError::new(
            SystemErrorType::InvalidNonce,
            "Invalid nonce value".to_string(),
        );
        let error: Error = system_err.into();
        assert!(matches!(error, Error::SystemError(_)));
    }

    #[test]
    fn test_error_display() {
        let error = Error::InvalidProof;
        assert_eq!(error.to_string(), "Invalid proof");

        let error = Error::WalletError("Balance too low".to_string());
        assert_eq!(error.to_string(), "Wallet error: Balance too low");
    }
}
