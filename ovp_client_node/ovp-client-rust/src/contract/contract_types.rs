// ./src/contract/contract_types.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Contract {
    pub address: String,
    pub code: Vec<u8>,
    pub initial_state: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExecutionContext {
    pub contract: Contract,
    pub stack: Vec<Vec<u8>>,
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    pub memory: Vec<u8>,
    pub program_counter: usize,
    pub gas_remaining: u64,
    pub return_data: Vec<u8>,
    pub caller: String,
    pub value: u64,
    pub input_data: Vec<u8>,
}