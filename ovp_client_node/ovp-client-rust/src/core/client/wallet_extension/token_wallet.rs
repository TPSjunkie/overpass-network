use crate::core::client::transaction::transaction_oc_data::TransactionOCData;
use crate::core::client::wallet_extension::token_oc_data::TokenOCData;
use crate::core::client::wallet_extension::wallet_extension_types::WalletExtension;

pub struct TokenWallet {
    token_oc_data: Option<TokenOCData>,
    transaction_oc_data: Option<TransactionOCData<WalletExtension>>,
}
impl TokenWallet {
    pub fn new(
        token_oc_data: TokenOCData,
        transaction_oc_data: TransactionOCData<WalletExtension>,
    ) -> Self {
        Self {
            token_oc_data: Some(token_oc_data),
            transaction_oc_data: Some(transaction_oc_data),
        }
    }

    pub fn get_token_oc_data(&self) -> Option<&TokenOCData> {
        self.token_oc_data.as_ref()
    }

    pub fn get_transaction_oc_data(&self) -> Option<&TransactionOCData<WalletExtension>> {
        self.transaction_oc_data.as_ref()
    }

    pub fn set_token_oc_data(&mut self, token_oc_data: TokenOCData) {
        self.token_oc_data = Some(token_oc_data);
    }

    pub fn set_transaction_oc_data(
        &mut self,
        transaction_oc_data: TransactionOCData<WalletExtension>,
    ) {
        self.transaction_oc_data = Some(transaction_oc_data);
    }

    pub fn clear_token_oc_data(&mut self) {
        self.token_oc_data = None;
    }

    pub fn clear_transaction_oc_data(&mut self) {
        self.transaction_oc_data = None;
    }

    pub fn has_token_oc_data(&self) -> bool {
        self.token_oc_data.is_some()
    }

    pub fn has_transaction_oc_data(&self) -> bool {
        self.transaction_oc_data.is_some()
    }
}
