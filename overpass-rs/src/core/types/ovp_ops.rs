use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpCode {
    Root(RootOpCode),
    Intermediate(IntermediateOpCode),
    Wallet(WalletOpCode),
    Channel(ChannelOpCode),
    Storage(StorageOpCode),
}

impl OpCode {
    #[inline]
    pub fn to_u8(&self) -> u8 {
        match self {
            OpCode::Root(op) => u8::from(*op),
            OpCode::Intermediate(op) => u8::from(*op),
            OpCode::Wallet(op) => u8::from(*op),
            OpCode::Channel(op) => u8::from(*op),
            OpCode::Storage(op) => u8::from(*op),
        }
    }

    pub fn from_u8(value: u8) -> Option<OpCode> {
        // Try each conversion in order, being careful about overlapping ranges
        if let Ok(op) = RootOpCode::try_from(value) {
            return Some(OpCode::Root(op));
        }
        if let Ok(op) = IntermediateOpCode::try_from(value) {
            return Some(OpCode::Intermediate(op));
        }
        if let Ok(op) = WalletOpCode::try_from(value) {
            return Some(OpCode::Wallet(op));
        }
        if let Ok(op) = StorageOpCode::try_from(value) {
            return Some(OpCode::Storage(op));
        }
        if let Ok(op) = ChannelOpCode::try_from(value) {
            return Some(OpCode::Channel(op));
        }
        None
    }
}

pub trait Operation {
    fn op_code(&self) -> OpCode;
    fn validate(&self) -> bool;
    fn execute(&self) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub op_code: OpCode,
    pub message: Option<String>,
    pub data: Option<Vec<u8>>,
}

// Channel Operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ChannelOpCode {
    CreatePayment = 0xA0,
    UpdateState = 0xA1,
    FinalizeState = 0xA2,
    DisputeState = 0xA3,
    InitChannel = 0xA4,
    ValidatePayment = 0xB0,
    ValidateState = 0xB1,
    ValidateFinalState = 0xB2,
    ValidateDispute = 0xB3,
    ValidateSettlement = 0xB4,
    ProcessPayment = 0xD1,
    InitiateSettlement = 0xE0,
    ProcessSettlement = 0xE1,
    FinalizeSettlement = 0xE2,
    ValidateTransition = 0xF0,
    VerifyProof = 0xF1,
    GetChannel = 0xF2,
}

// Root Operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum RootOpCode {
    SubmitEpoch = 0x01,
    ValidateEpoch = 0x02,
    FinalizeEpoch = 0x03,
    UpdateGlobalRoot = 0x10,
    ValidateGlobalState = 0x11,
    RegisterIntermediate = 0x20,
    RemoveIntermediate = 0x21,
    ValidateIntermediate = 0x22,
}

// Storage Operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum StorageOpCode {
    ChargeNode = 0xC0,
    DischargeNode = 0xC1,
    ValidateBattery = 0xC2,
    PropagateState = 0xD0,
    SyncState = 0xD1,
    ValidateSync = 0xD2,
    ReplicateState = 0xE0,
    ValidateReplica = 0xE1,
}

// Wallet Operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WalletOpCode {
    CreateChannel = 0x70,
    UpdateChannel = 0x71,
    CloseChannel = 0x72,
    ValidateChannel = 0x73,
    UpdateWalletTree = 0x80,
    ValidateWalletTree = 0x81,
    UpdateWalletState = 0x90,
    ValidateWalletState = 0x91,
    UpdateBalance = 0xB0,
    ValidateBalance = 0xB1,
    CreateTransaction = 0xC0,
    ValidateTransaction = 0xC1,
    ProcessTransaction = 0xC2,
    GenerateWalletProof = 0xD0,
    VerifyWalletProof = 0xD1,
}

// Intermediate Operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum IntermediateOpCode {
    RequestChannelOpen = 0x20,
    ApproveChannelOpen = 0x21,
    RequestChannelClose = 0x22,
    ProcessChannelClose = 0x23,
    RequestManualRebalance = 0x24,
    CheckChannelBalances = 0x25,
    ValidateRebalance = 0x26,
    ExecuteRebalance = 0x27,
    RegisterWallet = 0x30,
    UpdateWalletRoot = 0x31,
    ValidateWalletRoot = 0x32,
    ProcessWalletRoot = 0x33,
    ValidateStateUpdate = 0x34,
    AssignStorageNode = 0x40,
    UpdateStorageNode = 0x41,
    ValidateStorageNode = 0x42,
    StoreWalletState = 0x43,
    VerifyStorageState = 0x44,
    ReplicateState = 0x45,
    PrepareRootSubmission = 0x50,
    SubmitToRoot = 0x51,
    UpdateIntermediateState = 0x52,
    ValidateIntermediateState = 0x53,
    UpdateTree = 0x60,
    ValidateTree = 0x61,
}

// Implement From<OpCode> for u8 for all operation code types
impl From<ChannelOpCode> for u8 {
    fn from(code: ChannelOpCode) -> Self {
        code as u8
    }
}

impl From<RootOpCode> for u8 {
    fn from(code: RootOpCode) -> Self {
        code as u8
    }
}

impl From<StorageOpCode> for u8 {
    fn from(code: StorageOpCode) -> Self {
        code as u8
    }
}

impl From<WalletOpCode> for u8 {
    fn from(code: WalletOpCode) -> Self {
        code as u8
    }
}

impl From<IntermediateOpCode> for u8 {
    fn from(code: IntermediateOpCode) -> Self {
        code as u8
    }
}

// Implement TryFrom<u8> for all operation code types
impl TryFrom<u8> for ChannelOpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xA0 => Ok(Self::CreatePayment),
            0xA1 => Ok(Self::UpdateState),
            0xA2 => Ok(Self::FinalizeState),
            0xA3 => Ok(Self::DisputeState),
            0xA4 => Ok(Self::InitChannel),
            0xB0 => Ok(Self::ValidatePayment),
            0xB1 => Ok(Self::ValidateState),
            0xB2 => Ok(Self::ValidateFinalState),
            0xB3 => Ok(Self::ValidateDispute),
            0xB4 => Ok(Self::ValidateSettlement),
            0xD1 => Ok(Self::ProcessPayment),
            0xE0 => Ok(Self::InitiateSettlement),
            0xE1 => Ok(Self::ProcessSettlement),
            0xE2 => Ok(Self::FinalizeSettlement),
            0xF0 => Ok(Self::ValidateTransition),
            0xF1 => Ok(Self::VerifyProof),
            0xF2 => Ok(Self::GetChannel),
            _ => Err("Invalid Channel operation code"),
        }
    }
}

impl TryFrom<u8> for RootOpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::SubmitEpoch),
            0x02 => Ok(Self::ValidateEpoch),
            0x03 => Ok(Self::FinalizeEpoch),
            0x10 => Ok(Self::UpdateGlobalRoot),
            0x11 => Ok(Self::ValidateGlobalState),
            0x20 => Ok(Self::RegisterIntermediate),
            0x21 => Ok(Self::RemoveIntermediate),
            0x22 => Ok(Self::ValidateIntermediate),
            _ => Err("Invalid Root operation code"),
        }
    }
}

impl TryFrom<u8> for StorageOpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xC0 => Ok(Self::ChargeNode),
            0xC1 => Ok(Self::DischargeNode),
            0xC2 => Ok(Self::ValidateBattery),
            0xD0 => Ok(Self::PropagateState),
            0xD1 => Ok(Self::SyncState),
            0xD2 => Ok(Self::ValidateSync),
            0xE0 => Ok(Self::ReplicateState),
            0xE1 => Ok(Self::ValidateReplica),
            _ => Err("Invalid Storage operation code"),
        }
    }
}

impl TryFrom<u8> for WalletOpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x70 => Ok(Self::CreateChannel),
            0x71 => Ok(Self::UpdateChannel),
            0x72 => Ok(Self::CloseChannel),
            0x73 => Ok(Self::ValidateChannel),
            0x80 => Ok(Self::UpdateWalletTree),
            0x81 => Ok(Self::ValidateWalletTree),
            0x90 => Ok(Self::UpdateWalletState),
            0x91 => Ok(Self::ValidateWalletState),
            0xB0 => Ok(Self::UpdateBalance),
            0xB1 => Ok(Self::ValidateBalance),
            0xC0 => Ok(Self::CreateTransaction),
            0xC1 => Ok(Self::ValidateTransaction),
            0xC2 => Ok(Self::ProcessTransaction),
            0xD0 => Ok(Self::GenerateWalletProof),
            0xD1 => Ok(Self::VerifyWalletProof),
            _ => Err("Invalid Wallet operation code"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WalletExtensionStateChangeOp {
    CreateChannel = 0x70,
    UpdateChannel = 0x71,
    CloseChannel = 0x72,
    ValidateChannel = 0x73,
    ProcessChannel = 0x74,
    ValidateTransaction = 0x80,
    ProcessTransaction = 0x81,
    FinalizeState = 0x82,
}

impl From<WalletExtensionStateChangeOp> for u8 {
    fn from(code: WalletExtensionStateChangeOp) -> Self {
        code as u8
    }
}
impl TryFrom<u8> for WalletExtensionStateChangeOp {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x70 => Ok(Self::CreateChannel),
            0x71 => Ok(Self::UpdateChannel),
            0x72 => Ok(Self::CloseChannel),
            0x73 => Ok(Self::ValidateChannel),
            0x74 => Ok(Self::ProcessChannel),
            0x80 => Ok(Self::ValidateTransaction),
            0x81 => Ok(Self::ProcessTransaction),
            0x82 => Ok(Self::FinalizeState),
            _ => Err("Invalid WalletExtensionStateChangeOp operation code"),
        }
    }
}

impl TryFrom<u8> for IntermediateOpCode {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x20 => Ok(Self::RequestChannelOpen),
            0x21 => Ok(Self::ApproveChannelOpen),
            0x22 => Ok(Self::RequestChannelClose),
            0x23 => Ok(Self::ProcessChannelClose),
            0x24 => Ok(Self::RequestManualRebalance),
            0x25 => Ok(Self::CheckChannelBalances),
            0x26 => Ok(Self::ValidateRebalance),
            0x27 => Ok(Self::ExecuteRebalance),
            0x30 => Ok(Self::RegisterWallet),
            0x31 => Ok(Self::UpdateWalletRoot),
            0x32 => Ok(Self::ValidateWalletRoot),
            0x33 => Ok(Self::ProcessWalletRoot),
            0x34 => Ok(Self::ValidateStateUpdate),
            0x40 => Ok(Self::AssignStorageNode),
            0x41 => Ok(Self::UpdateStorageNode),
            0x42 => Ok(Self::ValidateStorageNode),
            0x43 => Ok(Self::StoreWalletState),
            0x44 => Ok(Self::VerifyStorageState),
            0x45 => Ok(Self::ReplicateState),
            0x50 => Ok(Self::PrepareRootSubmission),
            0x51 => Ok(Self::SubmitToRoot),
            0x52 => Ok(Self::UpdateIntermediateState),
            0x53 => Ok(Self::ValidateIntermediateState),
            0x60 => Ok(Self::UpdateTree),
            0x61 => Ok(Self::ValidateTree),
            _ => Err("Invalid Intermediate operation code"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_opcode_conversion() {
        assert_eq!(
            ChannelOpCode::try_from(0xA0),
            Ok(ChannelOpCode::CreatePayment)
        );
        assert_eq!(
            ChannelOpCode::try_from(0xFF),
            Err("Invalid Channel operation code")
        );
        assert_eq!(u8::from(ChannelOpCode::CreatePayment), 0xA0);
    }

    #[test]
    fn test_root_opcode_conversion() {
        assert_eq!(RootOpCode::try_from(0x01), Ok(RootOpCode::SubmitEpoch));
        assert_eq!(
            RootOpCode::try_from(0xFF),
            Err("Invalid Root operation code")
        );
        assert_eq!(u8::from(RootOpCode::SubmitEpoch), 0x01);
    }

    #[test]
    fn test_storage_opcode_conversion() {
        assert_eq!(StorageOpCode::try_from(0xC0), Ok(StorageOpCode::ChargeNode));
        assert_eq!(
            StorageOpCode::try_from(0xFF),
            Err("Invalid Storage operation code")
        );
        assert_eq!(u8::from(StorageOpCode::ChargeNode), 0xC0);
    }

    #[test]
    fn test_wallet_opcode_conversion() {
        assert_eq!(
            WalletOpCode::try_from(0x70),
            Ok(WalletOpCode::CreateChannel)
        );
        assert_eq!(
            WalletOpCode::try_from(0xFF),
            Err("Invalid Wallet operation code")
        );
        assert_eq!(u8::from(WalletOpCode::CreateChannel), 0x70);
    }

    #[test]
    fn test_intermediate_opcode_conversion() {
        assert_eq!(
            IntermediateOpCode::try_from(0x20),
            Ok(IntermediateOpCode::RequestChannelOpen)
        );
        assert_eq!(
            IntermediateOpCode::try_from(0xFF),
            Err("Invalid Intermediate operation code")
        );
        assert_eq!(u8::from(IntermediateOpCode::RequestChannelOpen), 0x20);
    }

    #[test]
    fn test_opcode_overlaps() {
        // Test that potentially overlapping values resolve to the correct type
        assert!(matches!(
            OpCode::from_u8(0xC0),
            Some(OpCode::Storage(StorageOpCode::ChargeNode))
        ));
        assert!(matches!(
            OpCode::from_u8(0xD0),
            Some(OpCode::Storage(StorageOpCode::PropagateState))
        ));
    }

    #[test]
    fn test_opcode_to_u8() {
        let channel_op = ChannelOpCode::CreatePayment;
        assert_eq!(OpCode::Channel(channel_op).to_u8(), 0xA0);

        let root_op = RootOpCode::SubmitEpoch;
        assert_eq!(OpCode::Root(root_op).to_u8(), 0x01);
    }

    #[test]
    fn test_opcode_from_u8() {
        assert_eq!(
            OpCode::from_u8(0xA0),
            Some(OpCode::Channel(ChannelOpCode::CreatePayment))
        );
        assert_eq!(
            OpCode::from_u8(0x01),
            Some(OpCode::Root(RootOpCode::SubmitEpoch))
        );
        assert_eq!(OpCode::from_u8(0xFF), None);
    }
}
