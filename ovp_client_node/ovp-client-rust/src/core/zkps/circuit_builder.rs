/// This module provides a high-level abstraction for building and managing zero-knowledge proof circuits.
/// 
/// # Overview
/// The circuit builder module implements functionality for constructing, proving, and verifying
/// zero-knowledge proofs using the Plonky2 proving system. It provides:
///
/// - A `Circuit` type that manages the lifecycle of proof generation and verification
/// - A `ZkCircuitBuilder` that provides a convenient builder pattern for circuit construction
/// - Common arithmetic and cryptographic operations like addition, multiplication, and hashing
/// - Error handling via the `CircuitError` type
/// - Virtual cell abstractions for managing proof targets and rotations
///
/// # Key Types
/// - `Circuit<F,C,D>`: Main circuit type for proof generation and verification
/// - `ZkCircuitBuilder`: High-level builder interface for circuit construction
/// - `VirtualCell`: Abstraction for circuit targets and rotations
/// - `CircuitError`: Error type for circuit operations
///
/// # Example
/// 
/// let config = CircuitConfig::standard_recursion_config();
/// let mut builder = ZkCircuitBuilder::new(config);
/// 
/// // Add inputs and constraints
/// let a = builder.add_public_input();
/// let b = builder.add_public_input();
/// let sum = builder.add(a, b);
///
/// // Build and prove
/// let circuit = builder.build()?;
/// let proof = circuit.prove()?;
/// circuit.verify(&proof)?;
/// 

// ./src/core/zkps/circuit_builder.rs

use plonky2::{
    field::extension::Extendable,
    hash::{hash_types::RichField, poseidon::PoseidonHash},
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
}

#[derive(Clone, Debug)]
pub struct VirtualCell {
    target: Target,
    rotation: i32,
}

impl VirtualCell {
    pub fn new(target: Target) -> Self {
        Self {
            target,
            rotation: 0,
        }
    }

    pub fn with_rotation(target: Target, rotation: i32) -> Self {
        Self { target, rotation }
    }

    pub fn target(&self) -> Target {
        self.target
    }

    pub fn rotation(&self) -> i32 {
        self.rotation
    }
}

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

    pub fn build(&mut self) -> Result<(), CircuitError> {
        let builder = std::mem::replace(&mut self.builder, CircuitBuilder::new(CircuitConfig::default()));
        let data = builder.build::<C>();
        self.data = Some(data);
        Ok(())
    }

    pub fn verify(&self, proof: &ProofWithPublicInputs<F, C, D>) -> Result<(), CircuitError> {
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::BuildError("Circuit not built".into()))?;
        
        data.verify(proof.clone())
            .map_err(|e| CircuitError::VerificationError(e.to_string()))
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

        let data = self.data.take() 
            .ok_or_else(|| CircuitError::BuildError("Circuit not built".into()))?;

        data.prove(witness)
            .map_err(|e| CircuitError::ProofGenerationError(e.to_string()))
    }

    pub fn builder(&self) -> &CircuitBuilder<F, D> {
        &self.builder 
    }

    pub fn builder_mut(&mut self) -> &mut CircuitBuilder<F, D> {
        &mut self.builder
    }
}

pub struct ZkCircuitBuilder<F: RichField + Extendable<D>, const D: usize> {
    builder: CircuitBuilder<F, D>,
    public_inputs: Vec<Target>,
}

impl<F: RichField + Extendable<D>, const D: usize> ZkCircuitBuilder<F, D> {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            builder: CircuitBuilder::new(config),
            public_inputs: Vec::new(),
        }
    }

    pub fn add_public_input(&mut self) -> VirtualCell {
        let target = self.builder.add_virtual_public_input();
        self.public_inputs.push(target);
        VirtualCell::new(target)
    }

    pub fn add_virtual_target(&mut self) -> VirtualCell {
        VirtualCell::new(self.builder.add_virtual_target())
    }

    pub fn connect(&mut self, left: VirtualCell, right: VirtualCell) {
        self.builder.connect(left.target(), right.target());
    }

    pub fn add(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        VirtualCell::new(self.builder.add(a.target(), b.target()))
    }

    pub fn sub(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        VirtualCell::new(self.builder.sub(a.target(), b.target()))
    }

    pub fn mul(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        VirtualCell::new(self.builder.mul(a.target(), b.target()))
    }

    pub fn div(&mut self, a: VirtualCell, b: VirtualCell) -> VirtualCell {
        VirtualCell::new(self.builder.div(a.target(), b.target()))
    }

    pub fn constant(&mut self, value: F) -> VirtualCell {
        VirtualCell::new(self.builder.constant(value))
    }

    pub fn zero(&mut self) -> VirtualCell {
        VirtualCell::new(self.builder.zero())
    }

    pub fn one(&mut self) -> VirtualCell {
        VirtualCell::new(self.builder.one())
    }

    pub fn assert_zero(&mut self, value: VirtualCell) {
        self.builder.assert_zero(value.target());
    }

    pub fn assert_equal(&mut self, left: VirtualCell, right: VirtualCell) {
        self.builder.connect(left.target(), right.target());
    }

    pub fn range_check(&mut self, target: VirtualCell, bits: usize) {
        self.builder.range_check(target.target(), bits);
    }

    pub fn poseidon_hash(&mut self, inputs: &[VirtualCell]) -> VirtualCell {
        let input_targets: Vec<Target> = inputs.iter().map(|x| x.target()).collect();
        let hash = self.builder.hash_n_to_hash_no_pad::<PoseidonHash>(input_targets);
        VirtualCell::new(hash.elements[0])
    }

    pub fn build_circuit<C: GenericConfig<D, F = F>>(self) -> Result<Circuit<F, C, D>, CircuitError> {
        if self.builder.num_gates() == 0 {
            return Err(CircuitError::BuildError("Circuit has no gates".into()));
        }

        let mut circuit = Circuit {
            builder: self.builder,
            witness: Some(PartialWitness::new()),
            data: None,
            _phantom: PhantomData,
        };

        circuit.build()?;
        Ok(circuit)
    }
}
#[cfg(test)]
mod tests {
    use plonky2::field::types::Field;
    use super::*;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2::plonk::config::PoseidonGoldilocksConfig;

    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;

    fn setup_test_circuit() -> ZkCircuitBuilder<F, D> {
        let config = CircuitConfig::standard_recursion_config();
        ZkCircuitBuilder::new(config)
    }

    #[test]
    fn test_basic_arithmetic() {
        let mut builder = setup_test_circuit();
        let a = builder.add_public_input();
        let b = builder.add_public_input();
        let sum = builder.add(a.clone(), b.clone());
        let expected_sum = builder.constant(F::from_canonical_u64(15));
        builder.assert_equal(sum, expected_sum);
        let mut circuit = builder.build::<C>().unwrap();
        circuit.set_witness_value(a.target(), F::from_canonical_u64(7)).unwrap();
        circuit.set_witness_value(b.target(), F::from_canonical_u64(8)).unwrap();
        let proof = circuit.prove().unwrap();
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_arithmetic() {
        let mut builder = setup_test_circuit();
        let a = builder.add_public_input();
        let b = builder.add_public_input();
        let result = builder.build_arithmetic_circuit(
            &[a.clone(), b.clone()],
            ArithmeticOperation::Add
        ).unwrap();
        let expected = builder.constant(F::from_canonical_u64(15));
        builder.assert_equal(result, expected);
        let mut circuit = builder.build::<C>().unwrap();
        circuit.set_witness_value(a.target(), F::from_canonical_u64(7)).unwrap();
        circuit.set_witness_value(b.target(), F::from_canonical_u64(8)).unwrap();
        let proof = circuit.prove().unwrap();
        
    }
    
    #[test]
    fn test_range_check() {
        let mut builder = setup_test_circuit();
        let input = builder.add_public_input(); 
        builder.range_check(input.clone(), 8);
        let mut circuit = builder.build::<C>().unwrap();
        circuit.set_witness_value(input.target(), F::from_canonical_u64(255)).unwrap();
        let proof = circuit.prove().unwrap();
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_poseidon_hash() {
        let mut builder = setup_test_circuit();
        let input1 = builder.add_public_input();
        let input2 = builder.add_public_input(); 
        let _hash = builder.poseidon_hash(&[input1.clone(), input2.clone()]);

        let mut circuit = builder.build::<C>().unwrap();
        circuit
            .set_witness_value(input1.target(), F::from_canonical_u64(7))
            .unwrap();
        circuit
            .set_witness_value(input2.target(), F::from_canonical_u64(8))
            .unwrap();
        let _proof = circuit.prove().unwrap();
    }

    #[test]
    fn test_circuit_builder_errors() {
        let builder = setup_test_circuit();
        let result = builder.build::<C>();
        assert!(matches!(result, Err(plonky2::CircuitError::BuildError(_))));
    }
}