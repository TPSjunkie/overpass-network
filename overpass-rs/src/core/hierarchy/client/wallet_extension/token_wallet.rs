use crate::core::hierarchy::client::transaction::transaction_oc_data::TransactionOCData;
use crate::core::hierarchy::client::wallet_extension::token_oc_data::TokenOCData;

pub struct TokenWallet {
    token_oc_data: Option<TokenOCData>,
    transaction_oc_data: Option<TransactionOCData>,
}

impl TokenWallet {
    pub fn new(token_oc_data: TokenOCData, transaction_oc_data: TransactionOCData) -> Self {
        Self {
            token_oc_data: Some(token_oc_data),
            transaction_oc_data: Some(transaction_oc_data),
        }
    }

    pub fn get_token_oc_data(&self) -> Option<&TokenOCData> {
        self.token_oc_data.as_ref()
    }
}
