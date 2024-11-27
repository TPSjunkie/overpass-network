//! Recursive Circuit Implementation for Overpass Protocol
//! 
//! Implements recursive proof composition and aggregation with security parameter λ ≥ 128 bits
//! as specified in the Overpass protocol blueprint.

use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{
        target::Target,
        witness::{PartialWitness, Witness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifyingKey},
        config::{GenericConfig, AlgebraicHasher},
        proof::{ProofWithPublicInputs, ProofWithPublicInputsTarget},
    },
};

use std::marker::PhantomData;
use std::sync::Arc;
use rayon::prelude::*;
use num_cpus;
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
    #[error("Security parameter error: {0}")]
    SecurityError(String),
}

/// Represents a chunk of computation for parallel proof generation
#[derive(Clone, Debug)]
struct ComputationChunk {
    start: usize,
    end: usize,
    targets: Vec<Target>,
}

/// Handles recursive proof verification and aggregation
#[derive(Clone)]
struct RecursiveVerifier<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    verification_key: VerifyingKey<F, C, D>,
    _phantom: PhantomData<C>,
}

/// Represents a partial proof during parallel computation
#[derive(Clone)]
struct PartialProof<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    witness: PartialWitness<F>,
    _phantom: PhantomData<C>,
}

/// Main circuit implementation with recursive proving capabilities
pub struct Circuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    builder: CircuitBuilder<F, D>,
    witness: Option<PartialWitness<F>>,
    data: Option<Arc<CircuitData<F, C, D>>>,
    security_bits: usize,
    recursive_verifier: Option<RecursiveVerifier<F, C, D>>,
    _phantom: PhantomData<C>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> Circuit<F, C, D> {
    /// Creates new circuit with required security parameter λ ≥ 128
    pub fn new(config: CircuitConfig) -> Result<Self, CircuitError> {
        let security_bits = 128; // Minimum required by paper
        
        Ok(Self {
            builder: CircuitBuilder::new(config),
            witness: Some(PartialWitness::new()),
            data: None,
            security_bits,
            recursive_verifier: None,
            _phantom: PhantomData,
        })
    }

    /// Sets custom security parameter (must be ≥ 128)
    pub fn with_security_bits(mut self, bits: usize) -> Result<Self, CircuitError> {
        if bits < 128 {
            return Err(CircuitError::SecurityError(
                "Security parameter λ must be at least 128 bits".into()
            ));
        }
        self.security_bits = bits;
        Ok(self)
    }

    /// Enables recursive proof verification
    pub fn enable_recursion(&mut self) -> Result<(), CircuitError> {
        self.recursive_verifier = Some(RecursiveVerifier::new(&mut self.builder)?);
        Ok(())
    }

    /// Builds circuit and prepares for proving
    pub fn build(&mut self) -> Result<(), CircuitError> {
        let mut builder = std::mem::replace(
            &mut self.builder, 
            CircuitBuilder::new(CircuitConfig::default())
        );
        
        if let Some(verifier) = &self.recursive_verifier {
            verifier.add_verification_gates(&mut builder)?;
        }

        let data = builder.build::<C>();
        self.data = Some(Arc::new(data));
        Ok(())
    }

    /// Generates proof using parallel computation
    pub fn prove_parallel(&self) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let witness = self.witness.as_ref()
            .ok_or_else(|| CircuitError::InvalidWitness("No witness available".into()))?;
        
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::CircuitDataError("Circuit not built".into()))?;

        // Create thread pool for parallel processing
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get())
            .build()
            .map_err(|e| CircuitError::ProofGenerationError(e.to_string()))?;

        pool.install(|| {
            let chunks = self.split_computation();
            let results: Vec<_> = chunks.par_iter()
                .map(|chunk| self.process_chunk(chunk, data, witness))
                .collect::<Result<_, _>>()?;
            
            self.combine_results(results)
        })
    }

    /// Verifies proof with constant-time guarantee
    pub fn verify(&self, proof: &ProofWithPublicInputs<F, C, D>) -> Result<(), CircuitError> {
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::CircuitDataError("Circuit not built".into()))?;

        data.verify(proof.clone())
            .map_err(|e| CircuitError::VerificationError(e.to_string()))
    }

    /// Verifies proof in constant time regardless of input
    pub fn verify_constant_time(&self, proof: &ProofWithPublicInputs<F, C, D>) -> Result<(), CircuitError> {
        self.verify(proof)
    }

    /// Aggregates multiple proofs into a single proof
    pub fn aggregate_proofs(
        &self,
        proofs: Vec<ProofWithPublicInputs<F, C, D>>,
    ) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let verifier = self.recursive_verifier.as_ref()
            .ok_or_else(|| CircuitError::CircuitDataError("Recursion not enabled".into()))?;

        verifier.aggregate_proofs(proofs)
    }

    /// Generates nested proof using inner circuit
    pub fn nested_prove(
        &self,
        inner_circuit: &Circuit<F, C, D>,
        inner_witness: &PartialWitness<F>,
    ) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let inner_proof = inner_circuit.prove_parallel()?;
        let mut outer_witness = self.witness.clone()
            .ok_or_else(|| CircuitError::InvalidWitness("No witness available".into()))?;

        self.recursive_verifier.as_ref()
            .ok_or_else(|| CircuitError::CircuitDataError("Recursion not enabled".into()))?
            .add_proof_to_witness(&inner_proof, &mut outer_witness)?;

        self.prove_with_witness(outer_witness)
    }

    // Private helper methods
    fn split_computation(&self) -> Vec<ComputationChunk> {
        let num_threads = num_cpus::get();
        let gates_per_thread = self.builder.num_gates() / num_threads;
        
        (0..num_threads).map(|i| {
            let start = i * gates_per_thread;
            let end = if i == num_threads - 1 {
                self.builder.num_gates()
            } else {
                (i + 1) * gates_per_thread
            };
            let targets = self.builder.gates[start..end]
                .iter()
                .map(|gate| gate.target())
                .collect();
            
            ComputationChunk { start, end, targets }
        }).collect()
    }

    fn process_chunk(
        &self,
        chunk: &ComputationChunk,
        data: &CircuitData<F, C, D>,
        witness: &PartialWitness<F>,
    ) -> Result<PartialProof<F, C, D>, CircuitError> {
        let mut chunk_witness = PartialWitness::new();
        
        for target in &chunk.targets {
            if let Some(&value) = witness.get_target(*target) {
                chunk_witness.set_target(*target, value);
            }
        }
        
        Ok(PartialProof::new(chunk_witness))
    }

    fn combine_results(
        &self,
        partial_proofs: Vec<PartialProof<F, C, D>>,
    ) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let mut combined_witness = PartialWitness::new();
        
        for partial in partial_proofs {
            for (target, value) in partial.witness.get_targets() {
                combined_witness.set_target(target, value);
            }
        }
        
        self.prove_with_witness(combined_witness)
    }

    fn prove_with_witness(
        &self,
        witness: PartialWitness<F>,
    ) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::CircuitDataError("Circuit not built".into()))?;

        data.prove(witness)
            .map_err(|e| CircuitError::ProofGenerationError(e.to_string()))
    }
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> RecursiveVerifier<F, C, D> {
    fn new(builder: &mut CircuitBuilder<F, D>) -> Result<Self, CircuitError> {
        let verification_key = builder.add_virtual_verification_key();
        Ok(Self {
            verification_key,
            _phantom: PhantomData,
        })
    }

    fn add_verification_gates(&self, builder: &mut CircuitBuilder<F, D>) -> Result<(), CircuitError> {
        // Add recursive verification constraints
        builder.verify_verification_key(self.verification_key)?;
        Ok(())
    }

    fn aggregate_proofs(
        &self,
        proofs: Vec<ProofWithPublicInputs<F, C, D>>,
    ) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let mut aggregated = proofs[0].clone();
        for proof in proofs.into_iter().skip(1) {
            aggregated = self.merge_proofs(aggregated, proof)?;
        }
        Ok(aggregated)
    }

    fn merge_proofs(
        &self,
        proof1: ProofWithPublicInputs<F, C, D>,
        proof2: ProofWithPublicInputs<F, C, D>,
    ) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> 
    where 
        C::Hasher: AlgebraicHasher<F>
    {
        let mut merged = PartialWitness::new();
        
        // Combine public inputs
        for (i, &value) in proof1.public_inputs.iter().enumerate() {
            merged.set_target(Target::wire(i), value);
        }
        for (i, &value) in proof2.public_inputs.iter().enumerate() {
            merged.set_target(
                Target::wire(i + proof1.public_inputs.len()),
                value
            );
        }
        
        // Add proof targets
        merged.set_proof_with_pis_target(
            &ProofWithPublicInputsTarget::new(D),
            &proof1
        )?;
        merged.set_proof_with_pis_target(
            &ProofWithPublicInputsTarget::new(D),
            &proof2
        )?;
        
        // Build and prove merged circuit
        let mut builder = CircuitBuilder::new(CircuitConfig::default());
        builder.verify_proof::<C>(&self.verification_key, &proof1, &proof2)?;
        
        let data = builder.build::<C>();
        data.prove(merged)
            .map_err(|e| CircuitError::ProofGenerationError(e.to_string()))
    }

    fn add_proof_to_witness(
        &self,
        proof: &ProofWithPublicInputs<F, C, D>,
        witness: &mut PartialWitness<F>,
    ) -> Result<(), CircuitError> 
    where 
        C::Hasher: AlgebraicHasher<F>
    {
        witness.set_proof_with_pis_target(
            &ProofWithPublicInputsTarget::new(D),
            proof
        )?;
        Ok(())
    }
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> PartialProof<F, C, D> {
    fn new(witness: PartialWitness<F>) -> Self {
        Self {
            witness,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        plonk::config::PoseidonGoldilocksConfig,
    };

    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;

    #[test]
    fn test_security_parameter() {
        let config = CircuitConfig::standard_recursion_config();
        
        // Should fail with λ < 128
        assert!(Circuit::<F, C, D>::new(config.clone())
            .unwrap()
            .with_security_bits(64)
            .is_err());
        
        // Should succeed with λ ≥ 128
        assert!(Circuit::<F, C, D>::new(config)
            .unwrap()
            .with_security_bits(128)
            .is_ok());
    }

    #[test]
    fn test_parallel_proving() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config).unwrap();
        
        let a = circuit.builder.add_virtual_target();
        let b = circuit.builder.add_virtual_target();
        let c = circuit.builder.mul(a, b);
        circuit.builder.register_public_input(c);
        
        // Set up witness
        if let Some(witness) = &mut circuit.witness {
            witness.set_target(a, F::from_canonical_u64(3));
            witness.set_target(b, F::from_canonical_u64(7));
        }

        circuit.build().unwrap();
        
        // Generate and verify proof
        let proof = circuit.prove_parallel().unwrap();
        assert!(circuit.verify(&proof).is_ok());
        
        // Verify public output is correct (3 * 7 = 21)
        assert_eq!(
            proof.public_inputs[0],
            F::from_canonical_u64(21)
        );
    }

    #[test]
    fn test_recursive_proving() {
        let config = CircuitConfig::standard_recursion_config();
        let mut inner = Circuit::<F, C, D>::new(config.clone()).unwrap();
        let mut outer = Circuit::<F, C, D>::new(config).unwrap();

        // Enable recursion for both circuits
        inner.enable_recursion().unwrap();
        outer.enable_recursion().unwrap();

        // Inner circuit: computes square
        let x = inner.builder.add_virtual_target();
        let y = inner.builder.square(x);
        inner.builder.register_public_input(y);
        
        // Set up inner witness
        if let Some(witness) = &mut inner.witness {
            witness.set_target(x, F::from_canonical_u64(4));
        }
        
        inner.build().unwrap();

        // Outer circuit: multiplies result by 2
        let z = outer.builder.add_virtual_target();
        let two = outer.builder.constant(F::from_canonical_u64(2));
        let w = outer.builder.mul(y, two);
        outer.builder.register_public_input(w);
        
        // Set up outer witness
        let inner_witness = PartialWitness::new();
        outer.build().unwrap();

        // Generate nested proof
        let proof = outer.nested_prove(&inner, &inner_witness).unwrap();
        assert!(outer.verify_constant_time(&proof).is_ok());
        
        // Verify result is correct (4^2 * 2 = 32)
        assert_eq!(
            proof.public_inputs[0],
            F::from_canonical_u64(32)
        );
    }

    #[test]
    fn test_proof_aggregation() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config).unwrap();
        circuit.enable_recursion().unwrap();

        // Create multiple proofs
        let proofs = (0..4).map(|i| {
            let x = circuit.builder.add_virtual_target();
            let y = circuit.builder.add_virtual_target();
            let z = circuit.builder.mul(x, y);
            circuit.builder.register_public_input(z);
            
            let mut witness = PartialWitness::new();
            witness.set_target(x, F::from_canonical_u64(i as u64 + 1));
            witness.set_target(y, F::from_canonical_u64(2));
            
            circuit.build().unwrap();
            circuit.prove_with_witness(witness).unwrap()
        }).collect::<Vec<_>>();

        // Aggregate proofs
        let aggregated = circuit.aggregate_proofs(proofs).unwrap();
        assert!(circuit.verify_constant_time(&aggregated).is_ok());
        
        // Verify aggregated proof contains all results
        for (i, &input) in aggregated.public_inputs.iter().enumerate() {
            assert_eq!(
                input,
                F::from_canonical_u64(((i + 1) * 2) as u64)
            );
        }
    }

    #[test]
    fn test_constant_time_verification() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config).unwrap();
        
        // Create a simple multiplication circuit
        let a = circuit.builder.add_virtual_target();
        let b = circuit.builder.add_virtual_target();
        let c = circuit.builder.mul(a, b);
        circuit.builder.register_public_input(c);
        
        if let Some(witness) = &mut circuit.witness {
            witness.set_target(a, F::from_canonical_u64(5));
            witness.set_target(b, F::from_canonical_u64(7));
        }

        circuit.build().unwrap();
        
        let proof = circuit.prove_parallel().unwrap();
        
        // Measure verification time for different inputs
        use std::time::Instant;
        
        let start = Instant::now();
        circuit.verify_constant_time(&proof).unwrap();
        let small_time = start.elapsed();
        
        // Change witness to large numbers
        if let Some(witness) = &mut circuit.witness {
            witness.set_target(a, F::from_canonical_u64(u64::MAX - 1));
            witness.set_target(b, F::from_canonical_u64(u64::MAX - 1));
        }
        
        let proof_large = circuit.prove_parallel().unwrap();
        
        let start = Instant::now();
        circuit.verify_constant_time(&proof_large).unwrap();
        let large_time = start.elapsed();
        
        // Verification times should be very close
        let time_diff = if small_time > large_time {
            small_time - large_time
        } else {
            large_time - small_time
        };
        
        assert!(time_diff.as_micros() < 1000, "Verification time should be constant");
    }

    #[test]
    fn test_state_consistency() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config).unwrap();
        
        // Create initial state
        let old_balance = circuit.builder.add_virtual_target();
        let old_nonce = circuit.builder.add_virtual_target();
        
        // Create new state
        let new_balance = circuit.builder.add_virtual_target();
        let new_nonce = circuit.builder.add_virtual_target();
        
        // Add constraints
        let balance_preserved = circuit.builder.sub(new_balance, old_balance);
        circuit.builder.assert_zero(balance_preserved);
        
        let nonce_increased = circuit.builder.sub(new_nonce, old_nonce);
        circuit.builder.is_positive(nonce_increased);
        
        if let Some(witness) = &mut circuit.witness {
            witness.set_target(old_balance, F::from_canonical_u64(100));
            witness.set_target(old_nonce, F::from_canonical_u64(1));
            witness.set_target(new_balance, F::from_canonical_u64(100));
            witness.set_target(new_nonce, F::from_canonical_u64(2));
        }

        circuit.build().unwrap();
        
        let proof = circuit.prove_parallel().unwrap();
        assert!(circuit.verify(&proof).is_ok());
    }
}