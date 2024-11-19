// ./src/core/zkps/proof.rs

use plonky2::{
    field::goldilocks_field::GoldilocksField,
    iop::{
        target::Target,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::PoseidonGoldilocksConfig,
        proof::ProofWithPublicInputs,
    }, util::serialization::{DefaultGateSerializer, DefaultGeneratorSerializer},
};
use plonky2_field::types::Field;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

const D: usize = 2;
type F = GoldilocksField;
type C = PoseidonGoldilocksConfig;

#[wasm_bindgen]

pub struct Plonky2System {
    circuit_config: CircuitConfig,
    state_transition_circuit: StateTransitionCircuitData,
}

#[wasm_bindgen]
pub struct Plonky2SystemHandle(Rc<Plonky2System>);

impl Default for Plonky2SystemHandle {
    fn default() -> Self {
        Self(Rc::new(Plonky2System {
            circuit_config: CircuitConfig::standard_recursion_config(),
            state_transition_circuit: StateTransitionCircuitData::default(),
        }))
    }
}

#[wasm_bindgen]
impl Plonky2SystemHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Plonky2SystemHandle, JsValue> {
        let circuit_config = CircuitConfig::standard_recursion_config();
        let builder = CircuitBuilder::<F, D>::new(circuit_config.clone());
        let state_transition_circuit = build_state_transition_circuit(builder)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let system = Plonky2System {
            circuit_config,
            state_transition_circuit,
        };

        Ok(Plonky2SystemHandle(Rc::new(system)))
    }

    pub fn generate_proof_js(
        &self,
        old_balance: u64,
        old_nonce: u64,
        new_balance: u64,
        new_nonce: u64,
        transfer_amount: u64,
    ) -> Result<Vec<u8>, JsValue> {
        self.0
            .generate_proof(
                old_balance,
                old_nonce,
                new_balance,
                new_nonce,
                transfer_amount,
            )
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn verify_proof_js(&self, proof_bytes: &[u8]) -> Result<bool, JsValue> {
        self.0
            .verify_proof(proof_bytes)
            .map(|_| true)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl Plonky2System {
    pub fn generate_proof(
        &self,
        old_balance: u64,
        old_nonce: u64,
        new_balance: u64,
        new_nonce: u64,
        transfer_amount: u64,
    ) -> Result<Vec<u8>, PlonkyError> {
        let circuit_data = &self.state_transition_circuit.circuit_data;
        let mut pw = PartialWitness::new();

        fill_state_transition_witness(
            &mut pw,
            &self.state_transition_circuit,
            old_balance,
            old_nonce,
            new_balance,
            new_nonce,
            transfer_amount,
        )?;

        let proof = circuit_data
            .prove(pw)
            .map_err(|e| PlonkyError::ProofGenerationError(e.to_string()))?;

        Ok(proof.to_bytes())
    }

    pub fn verify_proof(&self, proof_bytes: &[u8]) -> Result<(), PlonkyError> {
        let circuit_data = &self.state_transition_circuit.circuit_data;
        let common_data = &circuit_data.common;

        let proof = ProofWithPublicInputs::<F, C, D>::from_bytes(proof_bytes.to_vec(), common_data)
            .map_err(|e| PlonkyError::InvalidInput(e.to_string()))?;

        circuit_data
            .verify(proof)
            .map_err(|e| PlonkyError::InvalidInput(e.to_string()))
    }
}
// Remove the Default derive macro as it conflicts with the manual implementation
pub struct StateTransitionCircuitData {
    pub circuit_data: CircuitData<F, C, D>,
    pub old_balance_target: Target,
    pub old_nonce_target: Target,
    pub new_balance_target: Target,
    pub new_nonce_target: Target,
    pub transfer_amount_target: Target,
}

impl Default for StateTransitionCircuitData {
    fn default() -> Self {
        Self {
            circuit_data: CircuitData::from_bytes(&[], &DefaultGateSerializer, &DefaultGeneratorSerializer::<C, D>::default()).expect("Failed to create default CircuitData"),
            old_balance_target: Target::default(),
            old_nonce_target: Target::default(),
            new_balance_target: Target::default(),
            new_nonce_target: Target::default(),
            transfer_amount_target: Target::default(),
        }
    }
}
fn build_state_transition_circuit(
    mut builder: CircuitBuilder<F, D>,
) -> Result<StateTransitionCircuitData, PlonkyError> {
    let old_balance_target = builder.add_virtual_public_input();
    let old_nonce_target = builder.add_virtual_public_input();
    let new_balance_target = builder.add_virtual_public_input();
    let new_nonce_target = builder.add_virtual_public_input();
    let transfer_amount_target = builder.add_virtual_public_input();

    let one = builder.one();
    let old_nonce_plus_one = builder.add(old_nonce_target, one);
    builder.connect(old_nonce_plus_one, new_nonce_target);

    let old_balance_minus_amount = builder.sub(old_balance_target, transfer_amount_target);
    builder.connect(old_balance_minus_amount, new_balance_target);

    let two = builder.constant(F::from_canonical_u64(2));
    let double_transfer = builder.mul(transfer_amount_target, two);
    let difference = builder.sub(old_balance_target, double_transfer);
    builder.range_check(difference, 64);

    let circuit_data = builder.build::<C>();

    Ok(StateTransitionCircuitData {
        circuit_data,
        old_balance_target,
        old_nonce_target,
        new_balance_target,
        new_nonce_target,
        transfer_amount_target,
    })
}

fn fill_state_transition_witness(
    pw: &mut PartialWitness<F>,
    circuit: &StateTransitionCircuitData,
    old_balance: u64,
    old_nonce: u64,
    new_balance: u64,
    new_nonce: u64,
    transfer_amount: u64,
) -> Result<(), PlonkyError> {
    let old_balance_f = F::from_canonical_u64(old_balance);
    let old_nonce_f = F::from_canonical_u64(old_nonce);
    let new_balance_f = F::from_canonical_u64(new_balance);
    let new_nonce_f = F::from_canonical_u64(new_nonce);
    let transfer_amount_f = F::from_canonical_u64(transfer_amount);

    pw.set_target(circuit.old_balance_target, old_balance_f)
        .map_err(|e| PlonkyError::ProofGenerationError(e.to_string()))?;
    pw.set_target(circuit.old_nonce_target, old_nonce_f)
        .map_err(|e| PlonkyError::ProofGenerationError(e.to_string()))?;
    pw.set_target(circuit.new_balance_target, new_balance_f)
        .map_err(|e| PlonkyError::ProofGenerationError(e.to_string()))?;
    pw.set_target(circuit.new_nonce_target, new_nonce_f)
        .map_err(|e| PlonkyError::ProofGenerationError(e.to_string()))?;
    pw.set_target(circuit.transfer_amount_target, transfer_amount_f)
        .map_err(|e| PlonkyError::ProofGenerationError(e.to_string()))?;

    Ok(())
}

#[derive(Debug)]
pub enum PlonkyError {
    InvalidInput(String),
    ProofGenerationError(String),
}

impl std::fmt::Display for PlonkyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PlonkyError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            PlonkyError::ProofGenerationError(msg) => write!(f, "Proof generation error: {}", msg),
        }
    }
}

impl std::error::Error for PlonkyError {}
