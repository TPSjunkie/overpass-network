use crate::core::error::errors::SystemError;
use crate::core::hierarchy::client::wallet_extension::wallet_utils::ByteArray32;
use crate::core::tokens::token_oc_data::{TokenOCData, TokenOCManager, TokenOCTransaction};

pub fn new(token_oc_data: TokenOCData, transaction_oc_data: TransactionOCData) -> Self {
    Self {
        token_oc_data: Some(token_oc_data),
        transaction_oc_data: Some(transaction_oc_data),
        ..Default::default()
    }
}

pub fn get_token_oc_data(&self) -> Option<&TokenOCData> {
    self.token_oc_data.as_ref()
}
