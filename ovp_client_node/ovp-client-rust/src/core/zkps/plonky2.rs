// Plonky2 Zero-Knowledge Proof Circuit Implementation
//
// This module provides a high-level abstraction for building and working with zero-knowledge proof circuits
// using the Plonky2 proving system. It includes the following main components:
//
// Key Structures:
// - Column: Represents a column in the circuit's constraint system
// - VirtualCell: Represents a cell in the circuit with a target and rotation
// - Circuit: Main circuit structure that wraps Plonky2's CircuitBuilder
// - ZkCircuitBuilder: High-level builder for constructing zero-knowledge proof circuits
//
// Main Features:
// 1. Circuit Building:
//    - Public and private input management
//    - Basic arithmetic operations (add, subtract, multiply)
//    - Poseidon hash operations
//    - Constraint assertions (zero, one, equality)
//
// 2. Transaction Circuit:
//    - Built-in support for basic transaction validation
//    - Balance updates verification
//    - Nonce management
//    - Automated proof generation and verification
//
// 3. Testing:
//    - Comprehensive test suite for circuit operations
//    - Example transaction circuit validation
//
// Usage:
// The module is designed to be used with Plonky2's proving system and supports
// both native Rust and WebAssembly targets through wasm-bindgen.

// .//src/core/zkps/plonky2.rs



use crate::core::zkps::circuit_builder::VirtualCell;
use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{
        target::Target,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder, 
        circuit_data::{CircuitConfig, CircuitData},
        config::GenericConfig,
        proof::ProofWithPublicInputs,
    },
};

use std::marker::PhantomData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CircuitError {
    #[error("Failed to build circuit: {0}")]
    BuildError(String),
    #[error("Failed to generate proof: {0}")]
    ProofGenerationError(String),
    #[error("Invalid target: {0}")]
    InvalidTarget(String),
    #[error("Invalid witness: {0}")]
    InvalidWitness(String),
    #[error("Circuit verification failed: {0}")]
    VerificationError(String),
    #[error("Circuit data error: {0}")]
    CircuitDataError(String),
}

// VirtualCell implementation remains the same

pub struct Circuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    builder: CircuitBuilder<F, D>,
    witness: Option<PartialWitness<F>>,
    data: Option<CircuitData<F, C, D>>,
    _phantom: PhantomData<C>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> Circuit<F, C, D> {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            builder: CircuitBuilder::new(config),
            witness: Some(PartialWitness::new()),
            data: None,
            _phantom: PhantomData,
        }
    }

    pub fn builder(&self) -> &CircuitBuilder<F, D> {
        &self.builder
    }

    pub fn builder_mut(&mut self) -> &mut CircuitBuilder<F, D> {
        &mut self.builder
    }

    pub fn build(&mut self) -> Result<(), CircuitError> {
        let data = self.builder.build::<C>();
        self.data = Some(data);
        Ok(())
    }

    pub fn set_witness_value(&mut self, target: Target, value: F) -> Result<(), CircuitError> {
        self.witness.as_mut()
            .ok_or_else(|| CircuitError::InvalidWitness("Witness not initialized".into()))?
            .set_target(target, value);
        Ok(())
    }

    pub fn prove(mut self) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let witness = self.witness.take()
            .ok_or_else(|| CircuitError::InvalidWitness("Cannot generate proof without witness".into()))?;

        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::CircuitDataError("Circuit not built".into()))?;

        data.prove(witness)
            .map_err(|e| CircuitError::ProofGenerationError(e.to_string()))
    }

    pub fn verify(&self, proof: &ProofWithPublicInputs<F, C, D>) -> Result<(), CircuitError> {
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::CircuitDataError("Circuit not built".into()))?;

        data.verify(proof.clone())
            .map_err(|e| CircuitError::VerificationError(e.to_string()))
    }
}
