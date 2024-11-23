use crate::common::error::client_errors::{ChannelError, ChannelErrorType};
use crate::common::types::ops::{ContractOpCode, OpCode};
use crate::common::types::state_boc::STATEBOC;
use js_sys::Uint8Array;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
pub type ChannelId = String;
pub type ChannelBalance = u64;
pub type ChannelNonce = u64;
pub type ChannelSeqNo = u64;
pub type ChannelSignature = String;
pub type PrivateChannelState = HashMap<String, String>;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
interface ByteArray32 {
    toArray(): Uint8Array;
}
"#;
#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ByteArray32(#[wasm_bindgen(skip)] pub [u8; 32]);
pub struct CreateChannelParams {
    pub counterp_channel_nonceay32: ByteArray32,
    pub initial_balance: u64,
    pub config: ChannelConfig,
    pub spending_limit: u64,
}

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub timeout: u64,
    pub min_balance: u64,
    pub max_balance: u64,
}

pub struct ChannelUpdate {
    pub new_state: PrivateChannelState,
    pub balance: u64,
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelStatus {
    Active,
    TransactionPending,
    DisputeOpen,
    Closing,
    Closed,
}

#[wasm_bindgen]
pub struct Transaction {
    sender: String,
    nonce: ChannelNonce,
    sequence_number: ChannelSeqNo,
    amount: ChannelBalance,
}

#[wasm_bindgen]
impl Transaction {
    #[wasm_bindgen(constructor)]
    pub fn new(
        sender: &str,
        nonce: ChannelNonce,
        sequence_number: ChannelSeqNo,
        amount: ChannelBalance,
    ) -> Transaction {
        Transaction {
            sender: sender.to_string(),
            nonce,
            sequence_number,
            amount,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn sender(&self) -> String {
        self.sender.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn nonce(&self) -> ChannelNonce {
        self.nonce
    }

    #[wasm_bindgen(getter)]
    pub fn sequence_number(&self) -> ChannelSeqNo {
        self.sequence_number
    }

    #[wasm_bindgen(getter)]
    pub fn amount(&self) -> ChannelBalance {
        self.amount
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ByteArray32Local(#[wasm_bindgen(skip)] pub [u8; 32]);
#[wasm_bindgen]
impl ByteArray32 {
    #[wasm_bindgen(constructor)]
    pub fn new(array: &[u8]) -> Result<ByteArray32, JsValue> {
        if array.len() != 32 {
            return Err(JsValue::from_str("Array must be 32 bytes long"));
        }
        let mut result = [0u8; 32];
        result.copy_from_slice(array);
        Ok(ByteArray32(result))
    }

    #[wasm_bindgen(js_name = fromWasmAbi)]
    pub fn from_wasm_abi(val: JsValue) -> Result<ByteArray32, JsValue> {
        let array = Uint8Array::new(&val);
        let vec = array.to_vec();
        Self::new(&vec)
    }

    #[wasm_bindgen(js_name = toWasmAbi)]
    pub fn to_wasm_abi(&self) -> JsValue {
        let array = Uint8Array::new_with_length(32);
        array.copy_from(&self.0);
        array.into()
    }

    pub fn to_array(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_string(&self) -> String {
        hex::encode(self.0.to_vec())
    }
    pub fn from_string(val: &str) -> Result<ByteArray32, JsValue> {
        let array = hex::decode(val).map_err(|_| JsValue::from_str("Invalid hex string"))?;
        Self::new(&array)
    }
}

#[wasm_bindgen]
pub struct ChannelContract {
    id: String,
    state: String,
    balance: ChannelBalance,
    nonce: ChannelNonce,
    seqno: ChannelSeqNo,
    op_code: ContractOpCode,
    status: ChannelStatus,
    timeout: Option<u64>,
    recipient_acceptance: Option<String>,
    challenger: Option<String>,
    initiated_at: Option<u64>,
    final_state: Option<JsValue>,
}

#[wasm_bindgen]
impl ChannelContract {
    #[wasm_bindgen(constructor)]
    pub fn new(id: &str) -> ChannelContract {
        ChannelContract {
            id: id.to_string(),
            state: String::new(),
            balance: 0,
            nonce: 0,
            seqno: 0,
            op_code: ContractOpCode::InitChannel,
            status: ChannelStatus::Active,
            timeout: None,
            recipient_acceptance: None,
            challenger: None,
            initiated_at: None,
            final_state: None,
        }
    }

    #[wasm_bindgen]
    pub fn update_balance(&mut self, amount: ChannelBalance) -> Result<(), JsValue> {
        let new_balance = self
            .balance
            .checked_add(amount)
            .ok_or_else(|| JsValue::from_str("Balance overflow"))?;
        self.balance = new_balance;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn create_state_boc(&self) -> Result<Box<[u8]>, JsValue> {
        let boc = self
            .create_state_boc_internal()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let serialized_boc = bincode::serialize(&boc)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(serialized_boc.into_boxed_slice())
    }
    fn create_state_boc_internal(&self) -> Result<STATEBOC, ChannelError> {
        use crate::common::types::state_boc::{Cell, CellType};
        let mut boc = STATEBOC::new();
        let state_hash = self.calculate_state_hash()?;
        let state_cell = Cell::new(
            self.serialize_state()?,
            Vec::new(),
            Vec::new(),
            CellType::Ordinary,
            state_hash,
            None,
        );
        boc.add_cell(state_cell);
        boc.add_root(0);
        Ok(boc)
    }
    fn serialize_state(&self) -> Result<Vec<u8>, ChannelError> {
        let mut data = Vec::new();
        data.extend_from_slice(self.id.as_bytes());
        data.extend_from_slice(&self.balance.to_le_bytes());
        data.extend_from_slice(&self.nonce.to_le_bytes());
        data.extend_from_slice(&self.seqno.to_le_bytes());
        data.push(u8::from(self.op_code));

        let state_bytes = self.state.as_bytes();
        data.extend_from_slice(&(state_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(state_bytes);

        Ok(data)
    }

    fn calculate_state_hash(&self) -> Result<[u8; 32], ChannelError> {
        let mut hasher = Sha256::new();
        hasher.update(&self.serialize_state()?);
        let hash = hasher.finalize();
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash);
        Ok(hash_array)
    }

    #[wasm_bindgen]
    pub fn process_transaction(&mut self, tx: &Transaction) -> Result<Box<[u8]>, JsValue> {
        self.validate_transaction(tx)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.apply_transaction(tx)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.create_state_boc()
    }

    fn validate_transaction(&self, tx: &Transaction) -> Result<(), ChannelError> {
        if tx.sender != self.id {
            return Err(ChannelError::new(
                ChannelErrorType::InvalidTransaction,
                "Invalid transaction sender".to_string(),
            ));
        }

        if tx.nonce != self.nonce + 1 {
            return Err(ChannelError::new(
                ChannelErrorType::InvalidNonce,
                "Invalid nonce".to_string(),
            ));
        }

        if tx.sequence_number != self.seqno + 1 {
            return Err(ChannelError::new(
                ChannelErrorType::InvalidSequence,
                "Invalid sequence number".to_string(),
            ));
        }

        if tx.amount > self.balance / 2 {
            return Err(ChannelError::new(
                ChannelErrorType::SpendingLimitExceeded,
                "Spending limit exceeded".to_string(),
            ));
        }

        Ok(())
    }

    fn apply_transaction(&mut self, tx: &Transaction) -> Result<(), ChannelError> {
        self.balance = self.balance.checked_sub(tx.amount).ok_or_else(|| {
            ChannelError::new(
                ChannelErrorType::InsufficientBalance,
                "Insufficient balance".to_string(),
            )
        })?;

        self.nonce += 1;
        self.seqno += 1;

        Ok(())
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> String {
        self.id.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn balance(&self) -> ChannelBalance {
        self.balance
    }

    #[wasm_bindgen(getter)]
    pub fn nonce(&self) -> ChannelNonce {
        self.nonce
    }

    #[wasm_bindgen(getter)]
    pub fn seqno(&self) -> ChannelSeqNo {
        self.seqno
    }

    #[wasm_bindgen(getter)]
    pub fn op_code(&self) -> ContractOpCode {
        self.op_code
    }

    #[wasm_bindgen(getter)]
    pub fn status(&self) -> ChannelStatus {
        self.status
    }

    #[wasm_bindgen]
    pub fn get_timeout(&self) -> JsValue {
        match self.timeout {
            Some(value) => JsValue::from_f64(value as f64),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen]
    pub fn set_timeout(&mut self, timeout: JsValue) {
        if let Some(value) = timeout.as_f64() {
            self.timeout = Some(value as u64);
        } else {
            self.timeout = None;
        }
    }

    #[wasm_bindgen]
    pub fn get_recipient_acceptance(&self) -> JsValue {
        match &self.recipient_acceptance {
            Some(value) => JsValue::from_str(value),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen]
    pub fn set_recipient_acceptance(&mut self, acceptance: JsValue) {
        if let Some(value) = acceptance.as_string() {
            self.recipient_acceptance = Some(value);
        } else {
            self.recipient_acceptance = None;
        }
    }

    #[wasm_bindgen]
    pub fn get_challenger(&self) -> JsValue {
        match &self.challenger {
            Some(value) => JsValue::from_str(value),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen]
    pub fn set_challenger(&mut self, challenger: JsValue) {
        if let Some(value) = challenger.as_string() {
            self.challenger = Some(value);
        } else {
            self.challenger = None;
        }
    }

    #[wasm_bindgen]
    pub fn get_initiated_at(&self) -> JsValue {
        match self.initiated_at {
            Some(value) => JsValue::from_f64(value as f64),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen]
    pub fn set_initiated_at(&mut self, initiated_at: JsValue) {
        if let Some(value) = initiated_at.as_f64() {
            self.initiated_at = Some(value as u64);
        } else {
            self.initiated_at = None;
        }
    }

    #[wasm_bindgen]
    pub fn get_final_state(&self) -> JsValue {
        match &self.final_state {
            Some(value) => value.clone(),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen]
    pub fn set_final_state(&mut self, final_state: JsValue) {
        if final_state.is_undefined() || final_state.is_null() {
            self.final_state = None;
        } else {
            self.final_state = Some(final_state);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_transaction(
        sender: &str,
        nonce: u64,
        sequence_number: u64,
        amount: u64,
    ) -> Transaction {
        Transaction::new(sender, nonce, sequence_number, amount)
    }

    #[test]
    fn test_new_channel_contract() {
        let id = "test_channel";
        let contract = ChannelContract::new(id);

        assert_eq!(contract.id(), id);
        assert_eq!(contract.balance(), 0);
        assert_eq!(contract.nonce(), 0);
        assert_eq!(contract.seqno(), 0);
        assert_eq!(contract.op_code(), ContractOpCode::InitChannel);
        assert_eq!(contract.status(), ChannelStatus::Active);
    }

    #[test]
    fn test_process_valid_transaction() {
        let mut contract = ChannelContract::new("test_channel");
        contract.update_balance(1000).unwrap();

        let tx = create_test_transaction("test_channel", 1, 1, 400);

        let result = contract.process_transaction(&tx);
        assert!(result.is_ok());
        assert_eq!(contract.balance(), 600);
        assert_eq!(contract.nonce(), 1);
        assert_eq!(contract.seqno(), 1);
    }

    #[test]
    fn test_validate_transaction_failures() {
        let contract = ChannelContract::new("test_channel");

        let tx = create_test_transaction("wrong_sender", 1, 1, 100);
        assert!(contract.validate_transaction(&tx).is_err());

        let tx = create_test_transaction("test_channel", 2, 1, 100);
        assert!(contract.validate_transaction(&tx).is_err());
    }

    #[test]
    fn test_spending_limit() {
        let mut contract = ChannelContract::new("test_channel");
        contract.update_balance(1000).unwrap();

        let tx = create_test_transaction("test_channel", 1, 1, 501);

        assert!(contract.process_transaction(&tx).is_err());
    }
}
