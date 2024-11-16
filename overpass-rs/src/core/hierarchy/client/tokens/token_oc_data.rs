use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenOCData {
    pub token_oc_master_address: String,
    pub token_oc_wallet_address: String,
    pub token_oc_wallet_balance: u64,
    pub token_oc_wallet_transactions: Vec<TokenOCTransaction>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenOCTransaction {
    pub transaction_id: String,
    pub token_oc_wallet_address: String,
    pub token_oc_wallet_balance: u64,
}

impl TokenOCData {
    pub fn new(
        token_oc_master_address: String,
        token_oc_wallet_address: String,
        token_oc_wallet_balance: u64,
        token_oc_wallet_transactions: Vec<TokenOCTransaction>,
    ) -> Self {
        TokenOCData {
            token_oc_master_address,
            token_oc_wallet_address,
            token_oc_wallet_balance,
            token_oc_wallet_transactions,
        }
    }

    pub fn get_token_oc_master_address(&self) -> String {
        self.token_oc_master_address.clone()
    }

    pub fn get_token_oc_wallet_address(&self) -> String {
        self.token_oc_wallet_address.clone()
    }

    pub fn get_token_oc_wallet_balance(&self) -> u64 {
        self.token_oc_wallet_balance
    }

    pub fn get_token_oc_wallet_transactions(&self) -> Vec<TokenOCTransaction> {
        self.token_oc_wallet_transactions.clone()
    }

    pub fn set_token_oc_wallet_balance(&mut self, token_oc_wallet_balance: u64) {
        self.token_oc_wallet_balance = token_oc_wallet_balance;
    }

    pub fn set_token_oc_wallet_transactions(
        &mut self,
        token_oc_wallet_transactions: Vec<TokenOCTransaction>,
    ) {
        self.token_oc_wallet_transactions = token_oc_wallet_transactions;
    }

    pub fn add_token_oc_wallet_transaction(
        &mut self,
        token_oc_wallet_transaction: TokenOCTransaction,
    ) {
        self.token_oc_wallet_transactions
            .push(token_oc_wallet_transaction);
    }

    pub fn remove_token_oc_wallet_transaction(
        &mut self,
        token_oc_wallet_transaction: TokenOCTransaction,
    ) {
        self.token_oc_wallet_transactions
            .retain(|x| x != &token_oc_wallet_transaction);
    }

    pub fn get_token_oc_wallet_transaction(
        &self,
        transaction_id: String,
    ) -> Option<TokenOCTransaction> {
        self.token_oc_wallet_transactions
            .iter()
            .find(|x| x.transaction_id == transaction_id)
            .cloned()
    }

    pub fn get_token_oc_wallet_transaction_by_index(
        &self,
        index: usize,
    ) -> Option<TokenOCTransaction> {
        self.token_oc_wallet_transactions.get(index).cloned()
    }

    pub fn get_token_oc_wallet_transaction_by_wallet_address(
        &self,
        token_oc_wallet_address: String,
    ) -> Option<TokenOCTransaction> {
        self.token_oc_wallet_transactions
            .iter()
            .find(|x| x.token_oc_wallet_address == token_oc_wallet_address)
            .cloned()
    }

    pub fn get_token_oc_wallet_transaction_by_balance(
        &self,
        token_oc_wallet_balance: u64,
    ) -> Option<TokenOCTransaction> {
        self.token_oc_wallet_transactions
            .iter()
            .find(|x| x.token_oc_wallet_balance == token_oc_wallet_balance)
            .cloned()
    }
}

pub struct TokenOCManager {
    token_oc_data: TokenOCData,
}

impl TokenOCManager {
    pub fn new(token_oc_data: TokenOCData) -> Self {
        TokenOCManager { token_oc_data }
    }

    pub fn get_token_oc_data(&self) -> TokenOCData {
        self.token_oc_data.clone()
    }

    pub fn set_token_oc_data(&mut self, token_oc_data: TokenOCData) {
        self.token_oc_data = token_oc_data;
    }

    pub fn get_token_oc_master_address(&self) -> String {
        self.token_oc_data.get_token_oc_master_address()
    }

    pub fn get_token_oc_wallet_address(&self) -> String {
        self.token_oc_data.get_token_oc_wallet_address()
    }

    pub fn get_token_oc_wallet_balance(&self) -> u64 {
        self.token_oc_data.get_token_oc_wallet_balance()
    }

    pub fn get_token_oc_wallet_transactions(&self) -> Vec<TokenOCTransaction> {
        self.token_oc_data.get_token_oc_wallet_transactions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_oc_data_creation() {
        let transactions = vec![TokenOCTransaction {
            transaction_id: "tx1".to_string(),
            token_oc_wallet_address: "wallet1".to_string(),
            token_oc_wallet_balance: 100,
        }];

        let data = TokenOCData::new(
            "master".to_string(),
            "wallet1".to_string(),
            100,
            transactions.clone(),
        );

        assert_eq!(data.get_token_oc_master_address(), "master");
        assert_eq!(data.get_token_oc_wallet_address(), "wallet1");
        assert_eq!(data.get_token_oc_wallet_balance(), 100);
        assert_eq!(data.get_token_oc_wallet_transactions(), transactions);
    }

    #[test]
    fn test_token_oc_transaction_operations() {
        let mut data =
            TokenOCData::new("master".to_string(), "wallet1".to_string(), 100, Vec::new());

        let transaction = TokenOCTransaction {
            transaction_id: "tx1".to_string(),
            token_oc_wallet_address: "wallet1".to_string(),
            token_oc_wallet_balance: 100,
        };

        data.add_token_oc_wallet_transaction(transaction.clone());
        assert_eq!(data.get_token_oc_wallet_transactions().len(), 1);

        let found = data.get_token_oc_wallet_transaction("tx1".to_string());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), transaction);

        data.remove_token_oc_wallet_transaction(transaction);
        assert_eq!(data.get_token_oc_wallet_transactions().len(), 0);
    }

    #[test]
    fn test_token_oc_manager() {
        let data = TokenOCData::new("master".to_string(), "wallet1".to_string(), 100, Vec::new());
        let manager = TokenOCManager::new(data.clone());

        assert_eq!(manager.get_token_oc_data(), data);
        assert_eq!(manager.get_token_oc_master_address(), "master");
        assert_eq!(manager.get_token_oc_wallet_balance(), 100);
    }
}