use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::io;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelErrorType {
    InvalidTransaction,
    InvalidNonce,
    InvalidSequence,
    InvalidAmount,
    InsufficientBalance,
    SpendingLimitExceeded,
    NoRootCell,
    InvalidOperation,
    InvalidArgument,
    NotFound,
    InvalidProof,
}

impl std::fmt::Display for ChannelErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ChannelErrorType::*;
        match self {
            InvalidTransaction => write!(f, "Invalid transaction"),
            InvalidNonce => write!(f, "Invalid nonce"),
            InvalidSequence => write!(f, "Invalid sequence"),
            InvalidAmount => write!(f, "Invalid amount"),
            InsufficientBalance => write!(f, "Insufficient balance"),
            SpendingLimitExceeded => write!(f, "Spending limit exceeded"),
            NoRootCell => write!(f, "No root cell"),
            InvalidOperation => write!(f, "Invalid operation"),
            InvalidArgument => write!(f, "Invalid argument"),
            NotFound => write!(f, "Not found"),
            InvalidProof => write!(f, "Invalid proof"),
        }
    }
}

#[derive(Debug)]
pub struct ChannelError {
    pub error_type: ChannelErrorType,
    pub message: String,
}

impl ChannelError {
    pub fn new(error_type: ChannelErrorType, message: String) -> Self {
        Self {
            error_type,
            message,
        }
    }
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.error_type, self.message)
    }
}
impl std::error::Error for ChannelError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
    IoError(String),
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
    CellError(Box<Error>),
    ZkProofError(ZkProofError),
    StateError(Box<Error>),
    SystemError(SystemError),
    CustomError(String),
    SerializationError(String),
    DeserializationError(String),
    LockError(String),
    ChannelNotFound(String),
    StateNotFound(String),
    InvalidBOC(String),
    ArithmeticError(String),
    StateBocError(StateBocError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemErrorType {
    InvalidTransaction,
    InvalidSignature,
    InvalidPublicKey,
    DecryptionError,
    InvalidAddress,
    ProofGenerationError,
    InvalidHash,
    InvalidNonce,
    InvalidSequence,
    StateUpdateError,
    InsufficientCharge,
    VerificationError,
    DataConversionError,
    InvalidInput,
    InvalidState,
    NoProof,
    ResourceUnavailable,
    NetworkError,
    ResourceLimitReached,
    NoRoots,
    CircuitError,
    ProofError,
    OperationDisabled,
    StorageError,
    LockAcquisitionError,
    InvalidReference,
    InvalidData,
    InvalidAmount,
    StateDataMismatch,
    SerializationError,
    InvalidProof,
    PeerUpdateError,
    InsufficientBalance,
    SpendingLimitExceeded,
    NoRootCell,
    InvalidOperation,
    NotFound,
}

impl fmt::Display for SystemErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTransaction => write!(f, "Invalid transaction"),
            Self::CircuitError => write!(f, "Circuit error"),
            Self::InvalidSignature => write!(f, "Invalid signature"),
            Self::InvalidPublicKey => write!(f, "Invalid public key"),
            Self::InvalidReference => write!(f, "Invalid reference"),
            Self::InvalidAddress => write!(f, "Invalid address"),
            Self::SerializationError => write!(f, "Serialization error"),
            Self::InvalidHash => write!(f, "Invalid hash"),
            Self::DecryptionError => write!(f, "Decryption error"),
            Self::ResourceLimitReached => write!(f, "Resource limit reached"),
            Self::PeerUpdateError => write!(f, "Peer update error"),
            Self::InvalidNonce => write!(f, "Invalid nonce"),
            Self::InvalidSequence => write!(f, "Invalid sequence"),
            Self::InvalidAmount => write!(f, "Invalid amount"),
            Self::InvalidData => write!(f, "Invalid data"),
            Self::InvalidState => write!(f, "Invalid state"),
            Self::InvalidProof => write!(f, "Invalid proof"),
            Self::NoProof => write!(f, "No proof"),
            Self::DataConversionError => write!(f, "Data conversion error"),
            Self::InvalidInput => write!(f, "Invalid input"),
            Self::ProofError => write!(f, "Proof error"),
            Self::StateDataMismatch => write!(f, "State data mismatch"),
            Self::OperationDisabled => write!(f, "Operation disabled"),
            Self::ResourceUnavailable => write!(f, "Resource unavailable"),
            Self::VerificationError => write!(f, "Verification error"),
            Self::StateUpdateError => write!(f, "State update error"),
            Self::ProofGenerationError => write!(f, "Proof generation error"),
            Self::LockAcquisitionError => write!(f, "Lock acquisition error"),
            Self::NetworkError => write!(f, "Network error"),
            Self::NoRoots => write!(f, "No roots"),
            Self::InsufficientBalance => write!(f, "Insufficient balance"),
            Self::SpendingLimitExceeded => write!(f, "Spending limit exceeded"),
            Self::NoRootCell => write!(f, "No root cell"),
            Self::InvalidOperation => write!(f, "Invalid operation"),
            Self::NotFound => write!(f, "Not found"),
            Self::InsufficientCharge => write!(f, "Insufficient charge"),
            Self::StorageError => write!(f, "Storage error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellError {
    DataTooLarge,
    TooManyReferences,
    InvalidData,
    IoError(String),
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
        CellError::IoError(err.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateBocError {
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

impl fmt::Display for StateBocError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateBocError::TooManyCells => write!(f, "Too many cells"),
            StateBocError::NoRoots => write!(f, "No roots"),
            StateBocError::TotalSizeTooLarge => write!(f, "Total size too large"),
            StateBocError::CellDataTooLarge => write!(f, "Cell data too large"),
            StateBocError::TooManyReferences => write!(f, "Too many references"),
            StateBocError::InvalidReference { from, to } => {
                write!(f, "Invalid reference from {} to {}", from, to)
            }
            StateBocError::InvalidRoot(index) => write!(f, "Invalid root at index {}", index),
            StateBocError::InvalidMerkleProof => write!(f, "Invalid Merkle proof"),
            StateBocError::InvalidPrunedBranch => write!(f, "Invalid pruned branch"),
            StateBocError::CycleDetected => write!(f, "Cycle detected"),
            StateBocError::MaxDepthExceeded => write!(f, "Max depth exceeded"),
            StateBocError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            StateBocError::DeserializationError(err) => write!(f, "Deserialization error: {}", err),
        }
    }
}

impl std::error::Error for CellError {}
impl std::error::Error for ZkProofError {}
impl std::error::Error for StateBocError {}

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
            Error::NetworkError(err) => write!(f, "Network error: {}", err),
            Error::CellError(err) => write!(f, "Cell error: {}", err),
            Error::ZkProofError(err) => write!(f, "ZK proof error: {}", err),
            Error::StateBocError(err) => write!(f, "BOC error: {}", err),
            Error::SystemError(err) => write!(f, "System error: {}", err),
            Error::StateError(err) => write!(f, "State error: {}", err),
            Error::StorageError(err) => write!(f, "Storage error: {}", err),
            Error::StakeError(err) => write!(f, "Stake error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerializationError(err.to_string())
    }
}

impl From<CellError> for Error {
    fn from(err: CellError) -> Self {
        Error::CellError(Box::new(Error::CustomError(err.to_string())))
    }
}

impl From<ZkProofError> for Error {
    fn from(err: ZkProofError) -> Self {
        Error::ZkProofError(err)
    }
}

impl From<StateBocError> for Error {
    fn from(err: StateBocError) -> Self {
        Error::StateBocError(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientError {
    InvalidProof,
    WalletError(String),
}

impl From<ClientError> for Error {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::InvalidProof => Error::InvalidProof,
            ClientError::WalletError(msg) => Error::WalletError(msg),
        }
    }
}

impl std::error::Error for ClientError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::InvalidProof => write!(f, "Invalid proof"),
            ClientError::WalletError(msg) => write!(f, "Wallet error: {}", msg),
        }
    }
}

/// Represents a result with a success value and an error value.
pub type Result<T> = std::result::Result<T, Error>;

