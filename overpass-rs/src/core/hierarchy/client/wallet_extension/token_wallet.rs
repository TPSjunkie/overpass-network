use crate::tokens::token_oc_data::{TokenOCData, TokenOCManager, TokenOCTransaction};

impl TokenWallet {
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
}
