// ./src/core/state/state_types.rs
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub state_type: StateType,
    pub state_data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StateType {
    State,
    StateTransition,
    StateTransitionProof,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StateTransition {
    pub old_state: Vec<u8>,
    pub new_state: Vec<u8>,
    pub old_blinding: [u8; 32],
    pub new_blinding: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct User;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StateTransitionType {
    StateTransition,
    StateTransitionProof,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StateTransitionResult {
    Success,
    Failure,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StateTransitionError {
    InvalidOldState,
    InvalidNewState,
    InvalidOldBlinding,
    InvalidNewBlinding,
    OldStateIsEqualToNewState,
    OldStateIsZeroed,
    NewStateIsZeroed,
    OldStateBlindingIsZeroed,
    NewStateBlindingIsZeroed,
    OldStateBlindingIsEqualToNewStateBlinding,
    OldStateBlindingIsZero,
    NewStateBlindingIsZero,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StateTransitionProof {
    pub old_state: Vec<u8>,
    pub new_state: Vec<u8>,
    pub old_blinding: [u8; 32],
    pub new_blinding: [u8; 32],
    pub proof: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum StateTransitionProofType {
    StateTransitionProof,
}
