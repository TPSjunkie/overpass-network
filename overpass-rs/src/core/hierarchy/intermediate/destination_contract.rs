// ./src/core/hierarchy/intermediate/destination_contract.rs

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DestinationContract {
    pub channel_id: String,
    pub balance: u64,
    pub signatures: HashMap<String, Vec<u8>>,
    pub settled: bool,
    pub submit_root: bool,
}

impl DestinationContract {
    pub fn new(channel_id: String, initial_balance: u64) -> Self {
        Self {
            channel_id,
            balance: initial_balance,
            signatures: HashMap::new(),
            settled: false,
            submit_root: false,
        }
    }

    pub fn settle_channel(&mut self) -> Result<(), &'static str> {
        if self.settled {
            return Err("Channel already settled");
        }
        self.settled = true;
        Ok(())
    }

    pub fn verify_signature(&self, party_id: &str, signature: &[u8]) -> bool {
        match self.signatures.get(party_id) {
            Some(stored_signature) => stored_signature == signature,
            None => false,
        }
    }

    pub fn add_signature(&mut self, party_id: String, signature: Vec<u8>) {
        self.signatures.insert(party_id, signature);
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    pub fn update_balance(&mut self, new_balance: u64) -> Result<(), &'static str> {
        if self.settled {
            return Err("Cannot update balance: channel is settled");
        }
        self.balance = new_balance;
        Ok(())
    }
}
