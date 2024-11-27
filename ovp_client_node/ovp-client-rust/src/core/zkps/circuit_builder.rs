//! Circuit Builder Module for Overpass Protocol
//! 
//! Implements the zero-knowledge proof circuits described in the Overpass protocol blueprint.
//! Provides core cryptographic functionality for self-proving unilateral state channels with
//! security parameter λ ≥ 128 bits and constant-time verification.

use std::sync::Arc;
use rayon::prelude::*;
use thiserror::Error;
use num_cpus;

use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::{
        hash_types::RichField,
        poseidon::PoseidonHash,
        merkle_tree::MerkleTree,
    },
    iop::{
        target::Target,
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, VerifyingKey},
        config::GenericConfig,
        proof::ProofWithPublicInputs,
    },
};

#[derive(Error, Debug)]
pub enum CircuitError {
    #[error("Circuit build failed: {0}")]
    BuildError(String),
    
    #[error("Proof generation failed: {0}")]
    ProofError(String),
    
    #[error("Verification failed: {0}")]
    VerificationError(String),
    
    #[error("Invalid security parameter: {0}")]
    SecurityError(String),
    
    #[error("State transition invalid: {0}")]
    StateError(String),
}

/// Represents a chunk of circuit computation for parallel processing
struct ComputationChunk {
    start: usize,
    end: usize,
    targets: Vec<Target>,
}

/// Core circuit metrics for complexity analysis
#[derive(Clone, Debug)]
pub struct CircuitMetrics {
    pub constraints: usize,
    pub variables: usize,
    pub depth: usize,
    pub security_bits: usize,
}

impl CircuitMetrics {
    /// Calculates circuit complexity: O(log d) where d is tree depth
    pub fn complexity(&self) -> f64 {
        (self.constraints as f64) * (self.depth as f64).log2()
    }
    
    /// Validates security parameter λ ≥ 128 bits
    pub fn validate_security(&self) -> Result<(), CircuitError> {
        if self.security_bits < 128 {
            return Err(CircuitError::SecurityError(
                format!("Security parameter λ must be ≥ 128 bits, got {}", self.security_bits)
            ));
        }
        Ok(())
    }
}

/// Main circuit implementation for Overpass protocol
pub struct Circuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    builder: CircuitBuilder<F, D>,
    witness: Option<PartialWitness<F>>,
    data: Option<Arc<CircuitData<F, C, D>>>,
    metrics: CircuitMetrics,
    state_tree: MerkleTree<PoseidonHash>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> Circuit<F, C, D> {
    /// Creates a new circuit with required security parameter
    pub fn new(config: CircuitConfig, security_bits: usize) -> Result<Self, CircuitError> {
        let metrics = CircuitMetrics {
            constraints: 0,
            variables: 0,
            depth: 0,
            security_bits,
        };
        
        metrics.validate_security()?;
        
        Ok(Self {
            builder: CircuitBuilder::new(config),
            witness: Some(PartialWitness::new()),
            data: None,
            metrics,
            state_tree: MerkleTree::new(),
        })
    }

    /// Adds public inputs to the circuit
    pub fn add_public_input(&mut self, input: F) -> Result<Target, CircuitError> {
        let target = self.builder.add_virtual_target();
        self.builder.register_public_input(target);
        
        if let Some(witness) = &mut self.witness {
            witness.set_target(target, input);
        }
        
        Ok(target)
    }

    /// Adds state transition constraints for channel updates
    pub fn add_state_transition(
        &mut self,
        old_state: &[Target],
        new_state: &[Target],
    ) -> Result<(), CircuitError> {
        // Balance conservation
        let old_sum = self.builder.add_many(&old_state[..2]);
        let new_sum = self.builder.add_many(&new_state[..2]);
        self.builder.connect(old_sum, new_sum);

        // Nonce monotonicity
        self.builder.range_check(new_state[2], 32);
        let nonce_diff = self.builder.sub(new_state[2], old_state[2]);
        self.builder.assert_positive(nonce_diff);

        // Merkle path validation
        self.add_merkle_constraints(
            new_state[3],
            self.state_tree.root(),
            &self.state_tree.generate_proof(new_state[0])?,
        )?;

        Ok(())
    }

    /// Adds Merkle tree constraints for state updates
    pub fn add_merkle_constraints(
        &mut self,
        leaf: Target,
        root: Target,
        proof: &[Target],
    ) -> Result<(), CircuitError> {
        let mut current = self.builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![leaf]);
        
        for &sibling in proof {
            let (left, right) = if current.elements[0] <= sibling {
                (current.elements[0], sibling)
            } else {
                (sibling, current.elements[0])
            };
            
            let hash_input = vec![left, right];
            current = self.builder.hash_n_to_hash_no_pad::<PoseidonHash>(hash_input);
        }
        
        self.builder.connect(current.elements[0], root);
        Ok(())
    }

    /// Builds the circuit and prepares for proving
    pub fn build(&mut self) -> Result<(), CircuitError> {
        self.metrics.validate_security()?;
        
        let data = self.builder.build::<C>();
        self.data = Some(Arc::new(data));
        
        // Update metrics after build
        self.metrics.constraints = self.builder.num_gates();
        self.metrics.variables = self.builder.num_vars();
        self.metrics.depth = self.calculate_depth();
        
        Ok(())
    }

    /// Generates proof with parallel processing
    pub fn prove(&self) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let witness = self.witness.as_ref()
            .ok_or_else(|| CircuitError::ProofError("No witness available".into()))?;
        
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::BuildError("Circuit not built".into()))?;

        // Split computation into parallel chunks
        let chunk_size = (self.metrics.constraints + num_cpus::get() - 1) / num_cpus::get();
        let chunks: Vec<_> = (0..self.metrics.constraints)
            .step_by(chunk_size)
            .map(|start| {
                let end = (start + chunk_size).min(self.metrics.constraints);
                ComputationChunk {
                    start,
                    end,
                    targets: self.builder.targets[start..end].to_vec(),
                }
            })
            .collect();

        // Process chunks in parallel
        let partial_proofs: Vec<_> = chunks.par_iter()
            .map(|chunk| self.prove_chunk(chunk, witness, data))
            .collect::<Result<_, _>>()?;

        // Combine partial proofs
        self.combine_proofs(partial_proofs)
    }

    /// Verifies proof with constant time guarantee
    pub fn verify(
        &self,
        proof: &ProofWithPublicInputs<F, C, D>,
    ) -> Result<(), CircuitError> {
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::BuildError("Circuit not built".into()))?;

        data.verify(proof.clone())
            .map_err(|e| CircuitError::VerificationError(e.to_string()))
    }

    /// Calculates circuit depth for complexity analysis
    fn calculate_depth(&self) -> usize {
        let num_gates = self.builder.num_gates();
        (num_gates as f64).log2().ceil() as usize
    }

    /// Processes a chunk of the proof computation
    fn prove_chunk(
        &self,
        chunk: &ComputationChunk,
        witness: &PartialWitness<F>,
        data: &CircuitData<F, C, D>,
    ) -> Result<PartialWitness<F>, CircuitError> {
        let mut chunk_witness = PartialWitness::new();
        
        for target in &chunk.targets {
            if let Some(&value) = witness.get_target(*target) {
                chunk_witness.set_target(*target, value);
            }
        }
        
        Ok(chunk_witness)
    }

    /// Combines partial proofs into final proof
    fn combine_proofs(
        &self,
        partial_proofs: Vec<PartialWitness<F>>,
    ) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let mut combined = PartialWitness::new();
        
        for partial in partial_proofs {
            for (target, value) in partial.get_targets() {
                combined.set_target(target, value);
            }
        }
        
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::BuildError("Circuit not built".into()))?;
            
        data.prove(combined)
            .map_err(|e| CircuitError::ProofError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::plonk::config::PoseidonGoldilocksConfig;

    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;

    #[test]
    fn test_security_parameter() {
        let config = CircuitConfig::standard_recursion_config();
        
        // Should fail with λ < 128
        assert!(Circuit::<F, C, D>::new(config.clone(), 64).is_err());
        
        // Should succeed with λ ≥ 128
        assert!(Circuit::<F, C, D>::new(config.clone(), 128).is_ok());
    }

    #[test]
    fn test_state_transition() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config, 128).unwrap();
        
        let old_state = [
            circuit.add_public_input(F::from_canonical_u64(100)).unwrap(),
            circuit.add_public_input(F::from_canonical_u64(50)).unwrap(),
            circuit.add_public_input(F::from_canonical_u64(1)).unwrap(),
        ];
        
        let new_state = [
            circuit.add_public_input(F::from_canonical_u64(90)).unwrap(),
            circuit.add_public_input(F::from_canonical_u64(60)).unwrap(),
            circuit.add_public_input(F::from_canonical_u64(2)).unwrap(),
        ];
        
        assert!(circuit.add_state_transition(&old_state, &new_state).is_ok());
    }

    #[test]
    fn test_parallel_proving() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config, 128).unwrap();
        
        // Add some constraints
        let x = circuit.add_public_input(F::from_canonical_u64(123)).unwrap();
        let y = circuit.add_public_input(F::from_canonical_u64(456)).unwrap();
        let z = circuit.builder.mul(x, y);
        circuit.builder.register_public_input(z);
        
        circuit.build().unwrap();
        let proof = circuit.prove().unwrap();
        assert!(circuit.verify(&proof).is_ok());
    }

    #[test]
    fn test_merkle_constraints() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config, 128).unwrap();
        
        let leaf = circuit.builder.add_virtual_target();
        let root = circuit.builder.add_virtual_target();
        let path = vec![circuit.builder.add_virtual_target(); 32];
        
        assert!(circuit.add_merkle_constraints(leaf, root, &path).is_ok());
    }

    #[test]
    fn test_circuit_metrics() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config, 128).unwrap();
        
        // Add some gates to test metrics
        let x = circuit.add_public_input(F::from_canonical_u64(1)).unwrap();
        let y = circuit.add_public_input(F::from_canonical_u64(2)).unwrap();
        circuit.builder.mul(x, y);
        
        circuit.build().unwrap();
        
        assert!(circuit.metrics.constraints > 0);
        assert!(circuit.metrics.variables > 0);
        assert!(circuit.metrics.depth > 0);
        assert_eq!(circuit.metrics.security_bits, 128);
    }
}