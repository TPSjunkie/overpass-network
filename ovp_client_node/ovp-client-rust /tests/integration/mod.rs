// ./tests/integration/mod.rs

/// Integration tests
/// These tests are designed to test the entire client stack
/// They are designed to test the entire client stack
/// They are designed to test the entire client stack
use crate::common::test_utils::*;
use crate::common::test_client::*;
use crate::common::test_contract::*;
use crate::common::test_wallet::*;
use crate::common::test_transaction::*;
use crate::common::test_state::*;
use crate::common::test_boc::*;
use crate::common::test_zkp::*;
use crate::common::test_zkp_interface::*;
use crate::common::test_zkp_transaction::*;
use crate::common::test_zkp_proof::*;
use crate::common::test_zkp_circuit::*;
use crate::common::test_zkp_witness::*;
use crate::common::test_zkp_public_inputs::*;
use crate::common::test_zkp_proof_data::*;
use crate::common::test_zkp_proof_data_length::*;
use crate::common::test_zkp_proof_data_format::*;

#[test]
fn test_client() {
    let client = Client::new();
    client.test();
}

#[test]
fn test_contract() {
    let client = Client::new();
    let contract = client.test_contract();
    contract.test();
}

#[test]
fn test_wallet() {
    let client = Client::new();
    let wallet = client.test_wallet();
    wallet.test();
}

#[test]
fn test_transaction() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    transaction.test();
}

#[test]
fn test_state() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    let state = client.test_state(&transaction);
    state.test();
}

#[test]
fn test_boc() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    let state = client.test_state(&transaction);
    let boc = client.test_boc(&state);
    boc.test();
}

#[test]
fn test_zkp() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    let state = client.test_state(&transaction);
    let boc = client.test_boc(&state);
    let zkp = client.test_zkp(&boc);
    zkp.test();
}

#[test]
fn test_zkp_interface() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    let state = client.test_state(&transaction);
    let boc = client.test_boc(&state);
    let zkp = client.test_zkp(&boc);
    let zkp_interface = client.test_zkp_interface(&zkp);
    zkp_interface.test();
}

#[test]
fn test_zkp_transaction() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    let state = client.test_state(&transaction);
    let boc = client.test_boc(&state);
    let zkp = client.test_zkp(&boc);
    let zkp_interface = client.test_zkp_interface(&zkp);
    let zkp_transaction = client.test_zkp_transaction(&zkp_interface);
    zkp_transaction.test();
}

#[test]
fn test_zkp_proof() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    let state = client.test_state(&transaction);
    let boc = client.test_boc(&state);
    let zkp = client.test_zkp(&boc);
    let zkp_interface = client.test_zkp_interface(&zkp);
    let zkp_transaction = client.test_zkp_transaction(&zkp_interface);
    let zkp_proof = client.test_zkp_proof(&zkp_transaction);
    zkp_proof.test();
}

#[test]
fn test_zkp_circuit() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    let state = client.test_state(&transaction);
    let boc = client.test_boc(&state);
    let zkp = client.test_zkp(&boc);
    let zkp_interface = client.test_zkp_interface(&zkp);
    let zkp_transaction = client.test_zkp_transaction(&zkp_interface);
    let zkp_proof = client.test_zkp_proof(&zkp_transaction);
    let zkp_circuit = client.test_zkp_circuit(&zkp_proof);
    zkp_circuit.test();
}

#[test]
fn test_zkp_witness() {
    let client = Client::new();
    let wallet = client.test_wallet();
    let transaction = client.test_transaction(&wallet);
    let state = client.test_state(&transaction);
    let boc = client.test_boc(&state);
    let zkp = client.test_zkp(&boc);
    let zkp_interface = client.test_zkp_interface(&zkp);
    let zkp_transaction = client.test_zkp_transaction(&zkp_interface);
    let zkp_proof = client.test_zkp_proof(&zkp_transaction);
    let zkp_circuit = client.test_zkp_circuit(&zkp_proof);
    let zkp_witness = client.test_zkp_witness(&zkp_circuit);
    zkp_witness.test();
}