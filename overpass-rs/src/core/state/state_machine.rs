// ./src/core/state/state_machine.rs

// This module provides the implementation of the StateMachine struct, which is used to manage the state of the Overpass Network.
// It includes methods to handle state updates, transactions, and proofs.

use crate::core::types::ovp_ops::*;
use crate::core::types::ovp_types::*;

pub struct StateMachine<StateUpdate> {
    current_state: State,
    action_history: Vec<StateUpdate>,
    transaction_history: Vec<Transaction>,
    state_updates: Vec<NetworkMessageType>,
    proofs: Vec<ZkProof>,
}

pub struct StateMachineError {
    pub error_type: StateMachineErrorType,
    pub id: [u8; 32],
    pub data: Vec<u8>,
}
/// Implements the core functionality of the StateMachine struct, which manages the state of the Overpass Network.
///
/// The StateMachine provides methods to apply transactions, apply state updates, generate proofs, and verify proofs.
/// It maintains the current state, transaction history, state updates, and proofs.
///
/// The `apply_transaction` method validates and applies a transaction to the current state, updating the transaction history.
/// The `apply_state_update` method validates and applies a state update to the current state, updating the state updates history.
/// The `generate_proof` method generates a zero-knowledge proof for the current state, transaction history, and state updates.
/// The `verify_proof` method verifies a given zero-knowledge proof against the current state.
///
/// The `validate_transaction` and `validate_state_update` methods are private helper functions that perform validation checks on transactions and state updates, respectively.

pub fn apply_transaction(
    &mut self,
    transaction: &Transaction,
) -> Result<(), StateMachine<crate::core::types::NetworkMessageType>> {
    if !self.validate_transaction(transaction) {
        return Err(StateMachineError::InvalidTransaction);
    }

    self.current_state.apply_transaction(transaction)?;
    self.transaction_history.push(transaction.clone());
    Ok(())
}

pub fn apply_state_update(
    &mut self,
    state_update: &crate::core::types::NetworkMessageType,
) -> Result<(), StateMachineError> {
    if !self.validate_state_update(state_update) {
        return Err(StateMachineError::InvalidStateUpdate);
    }

    self.current_state.apply_update(state_update)?;
    self.state_updates.push(state_update.clone());
    Ok(())
}

pub fn generate_proof(&self) -> Result<ZkProof, StateMachineError> {
    let proof = ZkProof::generate(
        &self.current_state,
        &self.transaction_history,
        &self.state_updates,
    )?;

    Ok(proof)
}

pub fn verify_proof(&self, proof: &ZkProof) -> Result<(), StateMachineError> {
    if !proof.verify(&self.current_state) {
        return Err(StateMachineError::InvalidProof);
    }
    Ok(())
}

fn validate_transaction(&self, transaction: &Transaction) -> bool {
    // Validate transaction signature
    if !transaction.verify_signature() {
        return false;
    }

    // Check if transaction conflicts with current state
    if self.current_state.has_conflicts(transaction) {
        return false;
    }

    true
}

fn validate_state_update(&self, state_update: &crate::core::types::NetworkMessageType) -> bool {
    // Validate state update signature
    if !state_update.verify_signature() {
        return false;
    }

    // Check if state update is consistent with current state
    if !self.current_state.is_consistent_with(state_update) {
        return false;
    }

    true
}
