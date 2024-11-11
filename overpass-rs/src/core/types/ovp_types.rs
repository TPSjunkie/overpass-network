// src/core/types/ovp_types.rs
use crate::core::storage_node::epidemic::{BatteryPropagation, SynchronizationManager};
use crate::core::storage_node::storage_node::StorageNode;
use crate::core::types::ovp_ops::ChannelOpCode;
use crate::core::ZkProof;
use crate::SparseMerkleTree;
use crate::ZkProofSystem;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::AtomicU64;
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
use std::time::Duration;

// Type aliases for clarity
pub type ChannelBalance = u64;
pub type ChannelId = [u8; 32];
pub type ChannelState = String;
pub type ChannelNonce = u64;
pub type ChannelSeqNo = u64;
pub type ChannelProof = [u8; 64];
pub type ChannelSignature = [u8; 64];
pub type ChannelTransaction = [u8; 32];
pub type ChannelWitness = Vec<[u8; 32]>;
pub type WalletMerkleRoot = [u8; 32];
pub type ChannelMerkleProof = Vec<[u8; 32]>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PrivateChannelState {
    pub balance: ChannelBalance,
    pub nonce: ChannelNonce,
    pub sequence_number: ChannelSeqNo,
    pub proof: ChannelProof,
    pub signature: ChannelSignature,
    pub transaction: ChannelTransaction,
    pub witness: ChannelWitness,
    pub merkle_root: WalletMerkleRoot,
    pub merkle_proof: ChannelMerkleProof,
    pub last_update: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChannelStatus {
    Active,
    TransactionPending {
        timeout: u64,
        reciepent_acceptance: ChannelSignature,
    },
    DisputeOpen {
        timeout: u64,
        challenger: [u8; 32],
    },
    Closing {
        initiated_at: u64,
        final_state: Box<PrivateChannelState>,
    },
    Closed,
}

// Intermediate Contract
pub struct IntermediateContract {
    pub id: [u8; 32],
    pub data: Vec<u8>,
}

// Intermediate Proof Exporter
pub struct IntermediateProofExporter {
    pub id: [u8; 32],
    pub data: Vec<u8>,
}

// Intermediate Tree Manager
pub struct IntermediateTreeManager {
    pub id: [u8; 32],
    pub data: Vec<u8>,
}

// Validation
pub struct Validation {
    pub id: [u8; 32],
    pub data: Vec<u8>,
}

// Settlement I
pub struct SettlementI {
    pub id: [u8; 32],
    pub data: Vec<u8>,
}

// Wallet Verifier
pub struct WalletVerifier {
    pub id: [u8; 32],
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Channel {
    pub channel_id: [u8; 32],
    pub wallet_id: [u8; 32],
    pub state: Arc<RwLock<PrivateChannelState>>,
    pub state_history: Vec<StateTransition>,
    pub participants: Vec<[u8; 32]>,
    pub config: ChannelConfig,
    pub spending_limit: u64,
    pub proof_system: Arc<ZkProofSystem>,
    pub(crate) boc_history: Vec<BOC>,
    pub(crate) proof_history: Vec<ZkProof>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub min_balance: u64,
    pub max_balance: u64,
    pub challenge_period: u64,
    pub max_state_size: usize,
    pub max_participants: usize,
}

#[derive(Clone, Debug)]
pub struct StateTransition {
    pub old_state: PrivateChannelState,
    pub new_state: PrivateChannelState,
    pub proof: ZkProof,
    pub timestamp: u64,
}

// Next section of src/core/types/ovp_types.rs

/// Core transaction type with privacy-preserving attributes
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub id: [u8; 32],
    pub channel_id: [u8; 32],
    pub sender: [u8; 32],
    pub recipient: [u8; 32],
    pub amount: u64,
    pub nonce: u64,
    pub sequence_number: u64,
    pub timestamp: u64,
    pub status: TransactionStatus,
    pub signature: [u8; 64],
    pub zk_proof: Vec<u8>,
    pub merkle_proof: Vec<u8>,
    pub previous_state: Vec<u8>,
    pub new_state: Vec<u8>,
    pub fee: u64,
}

pub struct ChannelContract {
    pub id: ChannelId,
    pub state: ChannelState,
    pub balance: ChannelBalance,
    pub nonce: ChannelNonce,
    pub seqno: ChannelSeqNo,
    pub op_code: ChannelOpCode,
}

/// type definitions for states
/// State is a struct that represents the state of a wallet
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub root: u64,
    pub balance: u64,
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProofType {
    StateTransition,
    BalanceTransfer,
    MerkleInclusion,
    Aggregate,
}

pub struct ProverKeys {
    pub state_key: Vec<u8>,
    pub transfer_key: Vec<u8>,
    pub merkle_key: Vec<u8>,
}

pub struct VerifierKeys {
    pub state_key: Vec<u8>,
    pub transfer_key: Vec<u8>,
    pub merkle_key: Vec<u8>,
}

// Circuit types needed for ZkProofSystem
pub struct StateCircuit;
pub struct TransferCircuit;
pub struct MerkleCircuit;

#[derive(Clone, Debug)]
pub struct Plonky2System {
    pub circuit_config: CircuitConfig,
    pub prover_key: Arc<RwLock<ProverOnlyCircuitData>>,
    pub verifier_key: Arc<RwLock<VerifierOnlyCircuitData>>,
}

// Placeholder structs for Plonky2 types
pub struct CircuitConfig;
pub struct ProverOnlyCircuitData;
pub struct VerifierOnlyCircuitData;

// Error and Result types section of src/core/types/ovp_types.rs

// src/core/types/ovp_types.rs

#[derive(Debug)]
pub enum SystemErrorType {
    ProofVerificationFailed,
    ChannelError,
    InsufficientBalance,
    InvalidFee,
    InvalidStateTransition,
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
    InvalidProof,
    InvalidState,
    InsufficientFunds,
    InsufficientCharge, // Add this variant
    SpendingLimitExceeded,
    InvalidProofData,
    InvalidProofDataLength,
    InvalidProofDataFormat,
    InvalidProofDataSignature,
    InvalidProofDataPublicKey,
    InvalidProofDataHash,
    ChannelNotFound,
    PeerNotFound,
    StorageError(String),
    StakeError(String),
    NetworkError(String),
    ChargingTooFrequent,         // Add this variant
    MaxChargingAttemptsExceeded, // Add this variant
}

#[derive(Debug)]
pub struct SystemError {
    pub id: [u8; 32],
    pub data: Vec<u8>,
    pub error_type: SystemErrorType,
    pub error_data: SystemErrorData,
    pub error_data_id: [u8; 32],
}

#[derive(Debug)]
pub struct SystemErrorData {
    pub id: [u8; 32],
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum ChannelError {
    InvalidStateTransition,
    InvalidState,
    InvalidSignature,
    InvalidPublicKey,
    InvalidAddress,
    InvalidAmount,
    InvalidChannel,
    InvalidNonce,
    InvalidSequence,
    InvalidTimestamp,
    BatteryError,
    InvalidProof,
    InsufficientFunds,
    InsufficientCharge,
    SpendingLimitExceeded,
    InvalidProofData,
    InvalidProofDataLength,
    InvalidProofDataFormat,
    InvalidProofDataSignature,
    InvalidProofDataPublicKey,
    InvalidProofDataHash,
    ChannelNotFound,
    StorageError(String),
    StakeError(String),
    NetworkError(String),
}

pub struct OMError {
    pub id: [u8; 32],
    pub data: Vec<u8>,
}

impl std::fmt::Display for OMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct OMErrorData {
    pub message: String,
}

// Result type aliases for cleaner error handling
pub type OMResult<T> = Result<T, OMError>;
pub type OMErrorResult<T> = Result<T, OMError>;
pub type TvmResult<T> = Result<T, OMError>;

// Storage Node and Battery System section of src/core/types/ovp_types.rs

#[derive(Clone, Debug)]
pub struct StorageNodeImpl<RootTree, IntermediateTreeManager, K, V> {
    pub node_id: [u8; 32],
    pub stake: u64,
    pub stored_bocs: Arc<RwLock<HashMap<[u8; 32], BOC>>>,
    pub stored_proofs: Arc<RwLock<HashMap<[u8; 32], ZkProof>>>,
    pub intermediate_trees: Arc<RwLock<HashMap<[u8; 32], IntermediateOp>>>,
    pub root_trees: Arc<RwLock<HashMap<[u8; 32], RootTree>>>,
    pub tree_manager: Arc<RwLock<IntermediateTreeManager>>,
    pub peers: Arc<RwLock<HashSet<[u8; 32]>>>,
    pub config: StorageNodeConfig,
    pub battery_system: Arc<RwLock<BatteryChargingSystem>>,
    pub epidemic_protocol: Arc<RwLock<EpidemicProtocol<K, V, RootTree, IntermediateTreeManager>>>,
}

#[derive(Clone, Debug)]
pub struct StorageNodeConfig {
    pub min_battery: u64,
    pub max_battery: u64,
    pub charge_rate: u64,
    pub discharge_rate: u64,
    pub suspension_period: Duration,
    pub sync_interval: Duration,
    pub charge_cooldown: u64,
    pub max_charge_attempts: u64,
    pub charge_wait_ms: u64,
    pub max_rebalance_amount: u64,
}

pub struct BatteryChargingSystem {
    pub battery_level: AtomicU64,
    pub config: StorageNodeConfig,
    pub last_charge_time: AtomicU64,
    pub reward_multiplier: AtomicU64,
    pub stake_amount: AtomicU64,
}

#[derive(Debug, Clone)]
pub struct BatteryMetrics {
    pub charge_percentage: f64,
    pub current_level: u64,
    pub reward_multiplier: u64,
    pub stake_amount: u64,
    pub last_check: u64,
}

// BOC (Bag of Cells) related types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BOC {
    pub cells: Vec<Cell>,
    pub roots: Vec<usize>, // Indices into cells array
}

impl Default for BOC {
    fn default() -> Self {
        BOC {
            cells: Vec::new(),
            roots: Vec::new(),
        }
    }
}

// Root Submission
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RootSubmission {
    pub boc: BOC,
    pub zk_proof: ZkProof,
    pub timestamp: u64,
    pub(crate) zkp: Vec<u8>,
}

// Slice of CellData
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellDataSlice {
    pub data: Vec<u8>,
    pub references: Vec<CellReference>,
    pub cell_type: CellType,
    pub merkle_hash: [u8; 32],
}

// CellReference related types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellReferenceSlice {
    pub cell_index: usize,
    pub merkle_hash: [u8; 32],
}
// CellReference related types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CellReference {
    pub cell_index: usize,
    pub merkle_hash: [u8; 32],
}
// CellType related types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CellType {
    Ordinary,
    PrunedBranch,
    LibraryReference,
    MerkleProof,
    MerkleUpdate,
}

// CellData related types
pub trait CellData {
    fn get_data(&self) -> &Vec<u8>;
    fn get_references(&self) -> &Vec<CellReference>;
    fn get_cell_type(&self) -> &CellType;
    fn get_merkle_hash(&self) -> &[u8; 32];
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cell {
    pub data: Vec<u8>,
    pub references: Vec<CellReference>,
    pub cell_type: CellType,
    pub merkle_hash: [u8; 32],
}

/// CellData related types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SliceData {
    pub data: Vec<u8>,
    pub references: Vec<CellReference>,
    pub cell_type: CellType,
    pub merkle_hash: [u8; 32],
}

#[derive(Debug, Clone)]
pub struct EpidemicProtocol<K, V, RootTree, IntermediateTreeManager> {
    pub storage_node: Arc<StorageNode<RootTree, IntermediateTreeManager>>,
    pub synchronization_manager:
        Arc<RwLock<SynchronizationManager<RootTree, IntermediateTreeManager>>>,
    pub battery_propagation: Arc<RwLock<BatteryPropagation<RootTree, IntermediateTreeManager>>>,
    pub last_check: u64,
    pub redundancy_factor: f64,
    pub propagation_probability: f64,
}

impl<K, V, RootTree, IntermediateTreeManager>
    EpidemicProtocol<K, V, RootTree, IntermediateTreeManager>
{
    pub fn new(
        storage_node: Arc<StorageNode<RootTree, IntermediateTreeManager>>,
        synchronization_manager: Arc<
            RwLock<SynchronizationManager<RootTree, IntermediateTreeManager>>,
        >,
        battery_propagation: Arc<RwLock<BatteryPropagation<RootTree, IntermediateTreeManager>>>,
        redundancy_factor: f64,
        propagation_probability: f64,
    ) -> Self {
        Self {
            storage_node,
            synchronization_manager,
            battery_propagation,
            last_check: 0,
            redundancy_factor,
            propagation_probability,
        }
    }
}

pub struct ChallengeRecord {
    pub node_id: [u8; 32],
    pub client_id: [u8; 32],
    pub challenge: [u8; 32],
    pub response: [u8; 32],
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct EpidemicMetrics {
    pub last_check: u64,
}

// Intermediate and Root Contract section of src/core/types/ovp_types.rs

pub struct IntermediateContractGeneric<
    WalletStateUpdate,
    WalletRootState,
    RebalanceRequest,
    ChannelClosureRequest,
    ChannelRequest,
    RootTree,
    IntermediateTreeManager,
> {
    pub pending_channels: HashMap<String, ChannelRequest>,
    pub closing_channels: HashMap<String, ChannelClosureRequest>,
    pub rebalance_queue: VecDeque<RebalanceRequest>,
    pub wallet_states: HashMap<String, WalletRootState>,
    pub pending_updates: HashMap<String, Vec<WalletStateUpdate>>,
    pub storage_nodes: StorageNode<RootTree, IntermediateTreeManager>,
    pub last_root_submission: Option<(u64, Vec<u8>)>,
    pub(crate) hash: [i32; 32], // (epoch, root)
}
#[derive(Clone, Debug)]

pub struct WalletStateUpdate {
    pub wallet_id: [u8; 32],
    pub new_balance: u64,
    pub merkle_root: [u8; 32],
    pub epoch_id: u64,
}

#[derive(Clone, Debug)]
pub struct WalletRootState {
    pub root_id: [u8; 32],
    pub merkle_tree: SparseMerkleTree,
    pub zk_proof: ZkProof,
    pub epoch_id: u64,
}

#[derive(Clone, Debug)]
pub struct RebalanceRequest {
    pub channel_id: [u8; 32],
    pub amount: u64,
    pub proof: ZkProof,
    pub timestamp: u64,
    pub public_key: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct ChannelClosureRequest {
    pub channel_id: [u8; 32],
    pub final_balance: u64,
    pub boc: Vec<u8>,
    pub proof: ZkProof,
    pub signature: [u8; 64],
}

#[derive(Clone, Debug)]
pub struct ChannelRequest {
    pub channel_id: [u8; 32],
    pub initial_state: PrivateChannelState,
    pub initial_balance: u64,
    pub initial_epoch: u64,
    pub initial_root: [u8; 32],
    pub channel_config: ChannelConfig,
    pub proof: ZkProof,
}

// Root Contract Types
#[derive(Clone, Debug)]
pub struct Epoch {
    pub epoch_id: u64,
    pub root_hash: [u8; 32],
    pub timestamp: u64,
    pub state_commitments: Vec<[u8; 32]>, // Commitments from intermediate contracts
    pub status: EpochStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EpochStatus {
    Active,
    Finalized,
    Reverted,
}

#[derive(Clone, Debug)]
pub struct GlobalState {
    pub global_root_hash: [u8; 32],
    pub total_balance: u64,
    pub active_wallet_count: u64,
    pub last_epoch_id: u64,
    pub state_transitions: Vec<StateTransitionRecord>,
}

#[derive(Clone, Debug)]
pub struct StateTransitionRecord {
    pub epoch_id: u64,
    pub root_hash: [u8; 32],
    pub affected_wallet_ids: Vec<[u8; 32]>,
    pub timestamp: u64,
}

// Tracking Epoch Progression
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpochTracker {
    pub epochs: Vec<Epoch>,
    pub current_epoch_id: u64,
}

// BuilderData
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuilderData {
    pub data: Vec<u8>,
}
// Intermediate Operations
#[derive(Clone, Debug)]
pub struct IntermediateOp {
    pub intermediate_op_id: u64,
    pub intermediate_op_type: IntermediateOpType,
    pub intermediate_op_data: IntermediateOpDataType,
}

#[derive(Clone, Debug)]
pub enum IntermediateOpType {
    CreateWalletExtension,
    UpdateWalletExtension,
    CloseWalletExtension,
    RebalanceChannels,
    CreateChannel,
    UpdateChannel,
    CloseChannel,
    CreateRoot,
    UpdateRoot,
    CloseRoot,
}

#[derive(Clone, Debug)]
pub enum IntermediateOpDataType {
    CreateWalletExtension(CreateWalletExtensionData),
    UpdateWalletExtension(UpdateWalletExtensionData),
    CloseWalletExtension(CloseWalletExtensionData),
    RebalanceChannels,
    CreateChannel(ChannelRequest),
    UpdateChannel(WalletStateUpdate),
    CloseChannel,
    CreateRoot(WalletRootState),
    UpdateRoot(WalletRootState),
    CloseRoot,
}

// Wallet Extension and Management section of src/core/types/ovp_types.rs

/// Manages multiple channels for a wallet, enabling rebalancing and proof generation
#[derive(Clone, Debug)]
pub struct WalletExtension {
    pub wallet_id: [u8; 32],
    pub channels: HashMap<[u8; 32], Arc<RwLock<Channel>>>,
    pub total_locked_balance: u64,
    pub rebalance_config: RebalanceConfig,
    pub proof_system: Arc<ZkProofSystem>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RebalanceConfig {
    pub enabled: bool,
    pub min_imbalance_threshold: u64,
    pub max_rebalance_amount: u64,
    pub target_ratios: HashMap<[u8; 32], f64>,
    pub rebalance_interval: u64,
}

#[derive(Clone, Debug)]
pub struct RebalanceOperation {
    pub from_channel: [u8; 32],
    pub to_channel: [u8; 32],
    pub amount: u64,
    pub proof: ZkProof,
}

/// Storage management for encrypted wallet states and state commitments
#[derive(Clone, Debug)]
pub struct WalletStorageManager {
    pub encrypted_states: HashMap<[u8; 32], EncryptedWalletState>,
    pub commitment_proofs: HashMap<[u8; 32], ZkProof>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedWalletState {
    pub encrypted_data: Vec<u8>,
    pub public_commitment: [u8; 32],
    pub proof_of_encryption: ZkProof,
}

/// Tracks wallet balances and state transitions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletBalanceTracker {
    pub wallet_balances: HashMap<[u8; 32], u64>,
    pub state_transitions: HashMap<[u8; 32], Vec<[u8; 32]>>, // List of Merkle root hashes per wallet ID
}

/// Root state tracker for managing wallet root transitions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RootStateTracker {
    pub root_history: Vec<WalletRoot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletRoot {
    pub root_id: [u8; 32],
    pub wallet_merkle_proofs: Vec<[u8; 32]>,
    pub aggregated_balance: u64,
}

// Wallet Extension Operation Data Types
#[derive(Clone, Debug)]
pub struct CreateWalletExtensionData {
    pub wallet_id: [u8; 32],
    pub initial_balance: u64,
}

#[derive(Clone, Debug)]
pub struct UpdateWalletExtensionData {
    pub wallet_id: [u8; 32],
    pub new_balance: u64,
}

#[derive(Clone, Debug)]
pub struct CloseWalletExtensionData {
    pub wallet_id: [u8; 32],
    pub final_balance: u64,
}

/// Represents common message information
#[derive(Clone, Debug)]
pub struct CommonMsgInfo {
    pub data: CommonMsgInfoData,
}

#[derive(Clone, Debug)]
pub struct CommonMsgInfoData {
    pub data: Vec<u8>,
}

/// Network message types for wallet operations
#[derive(Clone, Debug)]
pub enum NetworkMessageType {
    StateUpdate(WalletStateUpdate),
    RebalanceRequest(RebalanceRequest),
    ChannelOperation(ChannelOpType),
    ProofSubmission(ZkProof),
}

#[derive(Clone, Debug)]
pub enum ChannelOpType {
    Open(ChannelRequest),
    Update(ChannelUpdate),
    Close(ChannelClosureRequest),
}

#[derive(Clone, Debug)]
pub struct ChannelUpdate {
    pub channel_id: [u8; 32],
    pub new_state: PrivateChannelState,
    pub proof: ZkProof,
    pub(crate) balance: u64,
    pub(crate) id: [u8; 32],
    pub(crate) sequence_number: i32,
    pub(crate) status: ChannelStatus,
    pub(crate) zk_proof: Option<_>,
    pub(crate) merkle_proof: Option<_>,
    pub(crate) previous_state: Option<_>,
}

// NFT and Payment Types section of src/core/types/ovp_types.rs

/// Represents an Overpass NFT, including metadata and current state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NFT {
    pub nft_id: [u8; 32],
    pub owner: [u8; 32],
    pub metadata: NFTMetadata,
    pub nonce: u64,
    pub sequence_number: u64,
    pub last_update: u64,
    pub status: ChannelStatus,
}

/// Metadata for NFTs, including name, description, and image URL
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NFTMetadata {
    pub name: String,
    pub description: String,
    pub image: String,
}

/// Represents Overpass' native token (OVP) for off-chain transactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OVPToken {
    pub amount: u64,
    pub recipient: [u8; 32],
}

/// Types of payment supported within Overpass (OVP, Jetton, NFT)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OffchainPayment {
    pub payment_type: PaymentType,
    pub data: PaymentData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaymentType {
    Ovp,
    Jetton,
    Nft,
}

/// Payment data for different asset types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaymentData {
    Ovp(OVPToken),
    Jetton(JettonPayment),
    Nft(NFTPayment),
}

/// Payment-specific structure for Jetton transactions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JettonPayment {
    pub jetton_id: [u8; 32],
    pub amount: u64,
    pub recipient: [u8; 32],
}

/// Payment-specific structure for NFT transfers
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NFTPayment {
    pub nft_id: [u8; 32],
    pub recipient: [u8; 32],
}

/// Status of a payment (Pending, Confirmed, or Failed)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Challenge and dispute resolution structures
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Challenge {
    pub channel_id: [u8; 32],
    pub challenger: [u8; 32],
    pub challenged_state: PrivateChannelState,
    pub proof: ZkProof,
    pub timestamp: u64,
    pub response_deadline: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeResponse {
    pub challenge_id: [u8; 32],
    pub responder: [u8; 32],
    pub newer_state: PrivateChannelState,
    pub proof: ZkProof,
    pub timestamp: u64,
}

/// Transaction validation and completion status tracking
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub proof: ZkProof,
    pub boc: BOC,
    pub status: ProcessingStatus,
}

#[derive(Clone, Debug)]
pub enum ProcessingStatus {
    Success,
    ChannelClosed,
    StatesAggregated,
    Failed(String),
}

/// Network message types for client and storage requests
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientNetworkRequest {
    pub client_id: [u8; 32],
    pub message: MessageType,
}

/// Response from network to client request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientNetworkResponse {
    pub client_id: [u8; 32],
    pub response: NetworkResponse,
    pub timestamp: u64,
}

/// Storage-related network request for data retrieval or verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageNetworkRequest {
    pub storage_id: [u8; 32],
    pub message: StorageMessageType,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StorageMessageType {
    RetrieveData {
        key: [u8; 32],
    },
    StoreData {
        key: [u8; 32],
        value: Vec<u8>,
    },
    VerifyState {
        state_root: [u8; 32],
        proof: Vec<u8>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageNetworkResponse {
    pub storage_id: [u8; 32],
    pub response: NetworkResponse,
    pub data: Option<Vec<u8>>, // Used for data retrieval responses
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageType {
    TransactionMessage(TransactionMessage),
    StateUpdateMessage(StateUpdateMessage),
    ValidationRequestMessage(ValidationRequestMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionMessage {
    pub transaction_id: [u8; 32],
    pub sender: [u8; 32],
    pub recipient: [u8; 32],
    pub amount: u64,
    pub timestamp: u64,
    pub signature: [u8; 64],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateUpdateMessage {
    pub wallet_id: [u8; 32],
    pub new_balance: u64,
    pub merkle_root: [u8; 32],
    pub epoch_id: u64,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationRequestMessage {
    pub request_id: [u8; 32],
    pub payload: Vec<u8>, // Serialized data to validate
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkResponse {
    Success { message_id: [u8; 32] },
    Failure { message_id: [u8; 32], error: String },
}

/// Server network messaging types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerNetworkRequest {
    pub server_id: [u8; 32],
    pub message: MessageType,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerNetworkResponse {
    pub server_id: [u8; 32],
    pub response: NetworkResponse,
    pub timestamp: u64,
}

/// Epidemic protocol message types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EpidemicMessage {
    pub node_id: [u8; 32],
    pub message_type: EpidemicMessageType,
    pub payload: Vec<u8>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EpidemicMessageType {
    StateSync,
    BatteryUpdate,
    PeerDiscovery,
    ReplicationRequest,
}

// Bridge and Cross-chain Types section of src/core/types/ovp_types.rs

/// Destination contract for cross-chain operations
pub struct DestinationContract {
    pub channel_id: Channel,
    pub source_address: [u8; 32],
    pub destination_address: [u8; 32],
    pub initial_balance: u128,
    pub current_balance: u128,
    pub is_settled: bool,
}
/// Bitcoin bridge types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BTCBridgeState {
    pub btc_address: [u8; 32],
    pub locked_amount: u64,
    pub proof_height: u32,
    pub required_confirmations: u32,
    pub status: BridgeStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BTCProof {
    pub tx_hash: [u8; 32],
    pub merkle_proof: Vec<[u8; 32]>,
    pub block_header: [u8; 80],
    pub confirmation_height: u32,
}

/// Ethereum bridge types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ETHBridgeState {
    pub eth_address: [u8; 20],
    pub locked_amount: u128,
    pub chain_id: u64,
    pub status: BridgeStatus,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ETHProof {
    pub tx_hash: [u8; 32],
    pub receipt_proof: Vec<u8>,
    pub block_number: u64,
    pub contract_address: [u8; 20],
}

/// TON bridge types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TONBridgeState {
    pub ton_address: [u8; 32],
    pub locked_amount: u128,
    pub workchain_id: i32,
    pub status: BridgeStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TONProof {
    pub message_hash: [u8; 32],
    pub proof: Vec<u8>,
    pub block_proof: Vec<u8>,
    pub merkle_proof: Vec<u8>,
}

/// Shared bridge types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BridgeStatus {
    Pending,
    Locked,
    Released,
    Failed,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrossChainTransaction {
    pub source_chain: ChainType,
    pub destination_chain: ChainType,
    pub source_tx: Vec<u8>,
    pub proof: BridgeProof,
    pub amount: u128,
    pub recipient: [u8; 32],
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChainType {
    Bitcoin,
    Ethereum,
    TON,
    Overpass,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BridgeProof {
    BTC(BTCProof),
    ETH(ETHProof),
    TON(TONProof),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub min_confirmation_blocks: HashMap<ChainType, u32>,
    pub max_transaction_amount: HashMap<ChainType, u128>,
    pub bridge_contracts: HashMap<ChainType, Vec<u8>>, // Contract addresses
    pub relayer_addresses: HashMap<ChainType, Vec<u8>>,
    pub validation_threshold: u32,
}

/// Bridge operation types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BridgeOperation {
    Lock {
        chain: ChainType,
        amount: u128,
        recipient: [u8; 32],
    },
    Release {
        chain: ChainType,
        proof: BridgeProof,
        recipient: [u8; 32],
    },
    Validate {
        operation_id: [u8; 32],
        proof: BridgeProof,
    },
}

// Database and Chain Stats Types section of src/core/types/ovp_types.rs

/// Database Models for Epochs and Transactions
#[derive(Serialize, Deserialize, Debug)]
pub struct EpochData {
    pub id: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub block_count: u64,
    pub transaction_count: u64,
    pub total_volume: u64,
    pub rewards_distributed: u64,
    pub total_fees: u64,
    pub validator_count: u64,
    pub total_stake: u64,
    pub participation_rate: f64,
    pub status: String,
}

/// Chain statistics for monitoring
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainStats {
    pub total_epochs: u64,
    pub total_transactions: u64,
    pub total_volume: u64,
    pub total_blocks: u64,
    pub average_block_time: f64,
}

/// Performance metrics for network monitoring
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub latency: HashMap<[u8; 32], u64>, // node_id -> latency in ms
    pub throughput: u64,                 // transactions per second
    pub active_channels: u64,
    pub total_nodes: u64,
    pub storage_usage: u64,                     // bytes
    pub battery_levels: HashMap<[u8; 32], f64>, // node_id -> battery percentage
}

/// API Types for external interaction
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub code: u32,
    pub message: String,
    pub details: Option<String>,
}

/// Metrics collection types
#[derive(Clone, Debug)]
pub struct MetricsCollection {
    pub storage: Arc<MetricsStorage>,
}

#[derive(Clone)]

pub struct MetricsStorage {
    pub data: Arc<Mutex<HashMap<String, u64>>>,
}

/// Validation rule types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_id: u32,
    pub rule_type: ValidationRuleType,
    pub parameters: HashMap<String, String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ValidationRuleType {
    TransactionLimit,
    BalanceCheck,
    ProofVerification,
    StateConsistency,
    BatteryLevel,
    NetworkHealth,
}

/// Configuration types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemConfig {
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub metrics: MetricsConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub max_peers: u32,
    pub target_peer_count: u32,
    pub ping_interval: u64,
    pub sync_interval: u64,
    pub max_sync_batch: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub max_storage_size: u64,
    pub garbage_collection_interval: u64,
    pub replication_factor: u32,
    pub max_batch_size: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub collection_interval: u64,
    pub retention_period: u64,
    pub max_metrics_size: u64,
}
// Protocol and Consensus Types section of src/core/types/ovp_types.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalState {
    pub proposal_id: [u8; 32],
    pub proposer: [u8; 32],
    pub epoch: u64,
    pub state_root: [u8; 32],
    pub votes: HashMap<[u8; 32], Vote>, // validator -> vote
    pub timestamp: u64,
    pub status: ProposalStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Voting,
    Accepted,
    Rejected,
    Timeout,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vote {
    pub validator: [u8; 32],
    pub vote_type: VoteType,
    pub signature: [u8; 64],
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VoteType {
    Accept,
    Reject,
    Abstain,
}

/// Protocol message types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProtocolMessage {
    Proposal(ProposalMessage),
    Vote(VoteMessage),
    Commit(CommitMessage),
    Sync(SyncMessage),
    Heartbeat(HeartbeatMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalMessage {
    pub proposal_id: [u8; 32],
    pub epoch: u64,
    pub state_root: [u8; 32],
    pub previous_root: [u8; 32],
    pub timestamp: u64,
    pub proposer_signature: [u8; 64],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteMessage {
    pub proposal_id: [u8; 32],
    pub vote: Vote,
    pub justification: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitMessage {
    pub proposal_id: [u8; 32],
    pub commit_root: [u8; 32],
    pub signatures: Vec<([u8; 32], [u8; 64])>, // (validator_id, signature)
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncMessage {
    pub from_epoch: u64,
    pub to_epoch: u64,
    pub roots: Vec<[u8; 32]>,
    pub proof: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub node_id: [u8; 32],
    pub timestamp: u64,
    pub battery_level: f64,
    pub storage_usage: u64,
    pub peer_count: u32,
}

// DEX Types section of src/core/types/ovp_types.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<Order>,
    pub asks: Vec<Order>,
    pub pair_id: [u8; 32],
    pub last_price: u64,
    pub last_update: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_id: [u8; 32],
    pub maker: [u8; 32],
    pub pair_id: [u8; 32],
    pub side: OrderSide,
    pub amount: u64,
    pub price: u64,
    pub timestamp: u64,
    pub status: OrderStatus,
    pub proof: ZkProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Trade {
    pub trade_id: [u8; 32],
    pub maker_order: [u8; 32],
    pub taker_order: [u8; 32],
    pub amount: u64,
    pub price: u64,
    pub timestamp: u64,
    pub maker_proof: ZkProof,
    pub taker_proof: ZkProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LiquidityPool {
    pub pool_id: [u8; 32],
    pub token_a: [u8; 32],
    pub token_b: [u8; 32],
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub total_shares: u64,
    pub fee_rate: u64,
    pub last_update: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolShare {
    pub pool_id: [u8; 32],
    pub owner: [u8; 32],
    pub share_amount: u64,
    pub proof: ZkProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settlement {
    pub settlement_id: [u8; 32],
    pub trades: Vec<Trade>,
    pub net_transfers: Vec<Transfer>,
    pub merkle_root: [u8; 32],
    pub timestamp: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transfer {
    pub from: [u8; 32],
    pub to: [u8; 32],
    pub token: [u8; 32],
    pub amount: u64,
    pub proof: ZkProof,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleProof {
    // Add necessary fields for MerkleProof
}
