use crate::common::types::state_boc::STATEBOC;
use crate::core::client::channel::channel_contract::ChannelContract;
use crate::core::state::sparse_merkle_tree_wasm::SparseMerkleTreeWasm;
use crate::core::zkps::plonky2::Plonky2SystemHandle;
use crate::core::zkps::proof::ZkProof;

use ed25519_dalek::Signature;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};

struct PlonkySystemHandleWrapper(Arc<Plonky2SystemHandle>);

impl fmt::Debug for PlonkySystemHandleWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlonkySystemHandleWrapper").finish()
    }
}

impl Default for PlonkySystemHandleWrapper {
    fn default() -> Self {
        Self(Arc::new(Plonky2SystemHandle::new().unwrap()))
    }
}

impl Clone for PlonkySystemHandleWrapper {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct WalletStateUpdate {
    pub old_balance: u64,
    pub old_nonce: u64,
    pub new_balance: u64,
    pub new_nonce: u64,
    pub transfer_amount: u64,
    pub merkle_root: [u8; 32],
}

// Remove the Debug derive as it's already implemented manually
#[derive(Clone)]
pub struct PrivateChannelState {
    pub balance: u64,
    pub nonce: u64,
    pub sequence_number: u64,
    pub merkle_root: [u8; 32],
}

impl Default for PrivateChannelState {
    fn default() -> Self {
        Self {
            balance: 0,
            nonce: 0,
            sequence_number: 0,
            merkle_root: [0; 32],
        }
    }
}
// instead of creating new definitions here.
#[derive(Clone, Default, Debug)]
pub struct RebalanceConfig {
    pub min_balance: u64,
    pub max_balance: u64,
    pub rebalance_threshold: u64,
    pub auto_rebalance: bool,
    pub rebalance_interval: u64,
    pub last_rebalance_timestamp: u64,
    pub target_balance: u64,
    pub allowed_deviation: u64,
    pub emergency_threshold: u64,
    pub max_rebalance_attempts: u32,
}

#[derive(Clone, Default, Debug)]
pub struct ChannelConfig {
    pub channel_id: [u8; 32],
    pub capacity: u64,
    pub min_deposit: u64,
    pub max_deposit: u64,
    pub timeout_period: u64,
    pub fee_rate: u64,
    pub is_active: bool,
    pub participants: Vec<[u8; 32]>,
    pub creation_timestamp: u64,
    pub last_update_timestamp: u64,
    pub settlement_delay: u64,
    pub dispute_window: u64,
    pub max_participants: u32,
    pub channel_type: u8,
    pub security_deposit: u64,
    pub auto_close_threshold: u64,
}

#[derive(Clone)]
pub struct TransactionRequest {
    pub channel_id: [u8; 32],
    pub recipient: [u8; 32],
    pub amount: u64,
    pub fee: u64,
}

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
    pub signature: Signature,
    pub zk_proof: Vec<u8>,
    pub merkle_proof: Vec<u8>,
    pub previous_state: Vec<u8>,
    pub new_state: Vec<u8>,
    pub fee: u64,
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            id: [0u8; 32],
            channel_id: [0u8; 32],
            sender: [0u8; 32],
            recipient: [0u8; 32],
            amount: 0,
            nonce: 0,
            sequence_number: 0,
            timestamp: 0,
            status: TransactionStatus::Pending,
            signature: Signature::from_slice(&[0u8; 64]).expect("Invalid signature"),
            zk_proof: Vec::new(),
            merkle_proof: Vec::new(),
            previous_state: Vec::new(),
            new_state: Vec::new(),
            fee: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Rejected,
    Processing,
}

#[derive(Debug, Clone)]
pub struct ChannelClosureRequest {
    pub channel_id: [u8; 32],
    pub final_balance: u64,
    pub boc: Vec<u8>,
    pub proof: ZkProof,
    pub signature: Vec<u8>,
    pub timestamp: u64,
    pub merkle_proof: Vec<u8>,
    pub previous_state: Vec<u8>,
    pub new_state: Vec<u8>,
}

impl Default for ChannelClosureRequest {
    fn default() -> Self {
        Self {
            channel_id: [0; 32],
            final_balance: 0,
            boc: Vec::new(),
            proof: ZkProof::default(),
            signature: Vec::new(),
            timestamp: 0,
            merkle_proof: Vec::new(),
            previous_state: Vec::new(),
            new_state: Vec::new(),
        }
    }
}

impl Default for WalletExtension {
    fn default() -> Self {
        Self {
            wallet_id: [0; 32],
            channels: HashMap::new(),
            total_locked_balance: 0,
            rebalance_config: RebalanceConfig::default(),
            proof_system: Arc::new(Plonky2SystemHandle::default()),
            state_tree: Arc::new(RwLock::new(SparseMerkleTreeWasm::default())),
            root_hash: [0; 32],
            balance: 0,
            encrypted_states: HashMap::new(),
        }
    }
}

pub struct WalletExtension {
    pub wallet_id: [u8; 32],
    pub channels: HashMap<[u8; 32], Arc<RwLock<ChannelContract>>>,
    pub total_locked_balance: u64,
    pub rebalance_config: RebalanceConfig,
    pub proof_system: Arc<Plonky2SystemHandle>,
    pub state_tree: Arc<RwLock<SparseMerkleTreeWasm>>,
    pub root_hash: [u8; 32],
    pub balance: u64,
    pub encrypted_states: HashMap<[u8; 32], Vec<u8>>,
}

impl fmt::Debug for WalletExtension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WalletExtension")
            .field("wallet_id", &self.wallet_id)
            .field("channels", &"<channels>")
            .field("total_locked_balance", &self.total_locked_balance)
            .field("rebalance_config", &self.rebalance_config)
            .field("proof_system", &"<proof_system>")
            .field("state_tree", &"<state_tree>")
            .field("root_hash", &self.root_hash)
            .field("balance", &self.balance)
            .field("encrypted_states", &"<encrypted_states>")
            .finish()
    }
}

pub struct WalletExtensionStateChange {
    pub op: WalletExtensionStateChangeOp,
    pub channel_id: [u8; 32],
    pub wallet_id: [u8; 32],
    pub state: WalletExtension,
    pub balance: u64,
    pub root_hash: [u8; 32],
    pub proof: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
    pub nonce: u64,
    pub fee: u64,
    pub merkle_proof: Vec<u8>,
    pub previous_state: Vec<u8>,
    pub new_state: Vec<u8>,
}

pub enum WalletExtensionStateChangeOp {
    ChannelStateTransition(ChannelStateTransition),
    ChannelStateTransitionProof(ChannelStateTransitionProof),
    ChannelClosure(ChannelClosureRequest),
    WalletRoot(WalletRootProof),
    MerkleInclusion(MerkleInclusionProof),
    StateTransition(StateTransition),
    BalanceTransfer(BalanceTransfer),
    EmergencyClose(EmergencyClose),
    Rebalance(Rebalance),
    UpdateChannelConfig(UpdateChannelConfig),
    UpdateSpendingLimit(UpdateSpendingLimit),
    UpdateFeeRate(UpdateFeeRate),
    UpdateTimeoutPeriod(UpdateTimeoutPeriod),
    UpdateDisputePeriod(UpdateDisputePeriod),
    UpdateAutoClose(UpdateAutoClose),
    UpdateAutoCloseThreshold(UpdateAutoCloseThreshold),
}

pub struct ChannelStateTransition {
    pub channel_id: [u8; 32],
}

pub struct ChannelStateTransitionProof {
    pub channel_id: [u8; 32],
}

pub struct WalletRootProof {
    pub wallet_id: [u8; 32],
}
pub struct MerkleInclusionProof {
    pub channel_id: [u8; 32],
}
pub struct StateTransition {
    pub channel_id: [u8; 32],
}
pub struct BalanceTransfer {
    pub channel_id: [u8; 32],
    pub sender: [u8; 32],
    pub recipient: [u8; 32],
    pub amount: u64,
}
pub struct EmergencyClose {
    pub channel_id: [u8; 32],
}
pub struct Rebalance {
    pub channel_id: [u8; 32],
}
pub struct UpdateChannelConfig {
    pub channel_id: [u8; 32],
}
pub struct UpdateSpendingLimit {
    pub channel_id: [u8; 32],
}
pub struct UpdateFeeRate {
    pub channel_id: [u8; 32],
}
pub struct UpdateTimeoutPeriod {
    pub channel_id: [u8; 32],
}
pub struct UpdateDisputePeriod {
    pub channel_id: [u8; 32],
}
pub struct UpdateAutoClose {
    pub channel_id: [u8; 32],
}

pub struct UpdateAutoCloseThreshold {
    pub channel_id: [u8; 32],
}
#[derive(Debug, Clone, Default)]
pub struct WalletExtensionConfig {
    pub channel_config: ChannelConfig,
    pub spending_limit: u64,
}

pub struct Channel {
    pub channel_id: [u8; 32],
    pub wallet_id: [u8; 32],
    pub state: Arc<RwLock<PrivateChannelState>>,
    pub state_history: Vec<StateTransition>,
    pub participants: Vec<[u8; 32]>,
    pub config: ChannelConfig,
    pub spending_limit: u64,
    pub proof_system: Arc<PlonkySystemHandleWrapper>,
    pub(crate) boc_history: Vec<STATEBOC>,
    pub(crate) proof: Vec<u8>,
}

impl fmt::Debug for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Channel")
            .field("channel_id", &self.channel_id)
            .field("wallet_id", &self.wallet_id)
            .field("state", &self.state)
            .field("state_history", &self.state_history)
            .field("participants", &self.participants)
            .field("config", &self.config)
            .field("spending_limit", &self.spending_limit)
            .field("proof_system", &self.proof_system)
            .field("boc_history", &self.boc_history)
            .field("proof", &self.proof)
            .finish()
    }
}
impl Default for Channel {
    fn default() -> Self {
        Self {
            channel_id: [0u8; 32],
            wallet_id: [0u8; 32],
            state: Arc::new(RwLock::new(PrivateChannelState::default())),
            state_history: Vec::new(),
            participants: Vec::new(),
            config: ChannelConfig::default(),
            spending_limit: 0,
            proof_system: Arc::new(PlonkySystemHandleWrapper::default()),
            boc_history: Vec::new(),
            proof: Vec::new(),
        }
    }
}
impl Channel {
    pub fn new(
        channel_id: [u8; 32],
        wallet_id: [u8; 32],
        state: PrivateChannelState,
        state_history: Vec<StateTransition>,
        participants: Vec<[u8; 32]>,
        config: ChannelConfig,
        spending_limit: u64,
        proof_system: Arc<PlonkySystemHandleWrapper>,
        boc_history: Vec<STATEBOC>,
        proof: Vec<u8>,
    ) -> Self {
        Self {
            channel_id,
            wallet_id,
            state: Arc::new(RwLock::new(state)),
            state_history,
            participants,
            config,
            spending_limit,
            proof_system,
            boc_history,
            proof,
        }
    }
}

impl PrivateChannelState {
    pub fn new(balance: u64, nonce: u64, sequence_number: u64, merkle_root: [u8; 32]) -> Self {
        Self {
            balance,
            nonce,
            sequence_number,
            merkle_root,
        }
    }
}
impl fmt::Debug for PrivateChannelState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateChannelState")
            .field("balance", &self.balance)
            .field("nonce", &self.nonce)
            .field("sequence_number", &self.sequence_number)
            .field("merkle_root", &self.merkle_root)
            .finish()
    }
}

impl PrivateChannelState {
    pub fn default() -> Self {
        Self::new(0, 0, 0, [0u8; 32])
    }
}

impl PrivateChannelState {
    pub fn update_balance(&mut self, amount: u64) {
        self.balance += amount;
    }
    pub fn update_nonce(&mut self, nonce: u64) {
        self.nonce = nonce;
    }
    pub fn update_sequence_number(&mut self, sequence_number: u64) {
        self.sequence_number = sequence_number;
    }
    pub fn update_merkle_root(&mut self, merkle_root: [u8; 32]) {
        self.merkle_root = merkle_root;
    }
}
impl fmt::Debug for StateTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateTransition")
            .field("channel_id", &self.channel_id)
            .finish()
    }
}
