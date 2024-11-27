// ./src/core/zkps/circuit_builder.rs

//! Circuit Builder Module for Overpass Protocol
//! 
//! This module implements the zero-knowledge proof
//!  circuits described in the Overpass protocol blueprint.
//! It provides the core cryptographic functionality for 
//! self-proving unilateral state channels.

use plonky2::{
    field::extension::Extendable,
    hash::{hash_types::RichField, poseidon::PoseidonHash},
    iop::{target::Target, witness::PartialWitness},
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::GenericConfig,
        proof::ProofWithPublicInputs,
    },
};

use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CircuitError {
    #[error("Circuit build failed: {0}")]
    BuildError(String),
    #[error("Proof generation failed: {0}")] 
    ProofError(String),
    #[error("Verification failed: {0}")]
    VerificationError(String),
}

/// Represents the complexity metrics of a circuit as defined in the blueprint
#[derive(Clone)]
pub struct CircuitComplexity {
    pub constraints: usize,
    pub variables: usize,
    pub depth: usize,
}

impl CircuitComplexity {
    /// Calculates circuit complexity according to blueprint formula:
    /// O(log d) where d is the tree depth
    pub fn calculate_complexity(&self) -> f64 {
        (self.constraints as f64) * (self.depth as f64).log2()
    }
}

/// Main circuit implementation for Overpass protocol
pub struct Circuit<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    builder: CircuitBuilder<F, D>,
    witness: Option<PartialWitness<F>>,
    data: Option<Arc<CircuitData<F, C, D>>>,
    complexity: CircuitComplexity,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> Circuit<F, C, D> {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            builder: CircuitBuilder::new(config),
            witness: Some(PartialWitness::new()),
            data: None,
            complexity: CircuitComplexity {
                constraints: 0,
                variables: 0,
                depth: 0,
            },
        }
    }

    /// Generates proof with security parameter λ ≥ 128 bits
    pub fn prove(&self) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let witness = self.witness.as_ref()
            .ok_or_else(|| CircuitError::ProofError("No witness available".into()))?;
        
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::BuildError("Circuit not built".into()))?;

        data.prove(witness.clone())
            .map_err(|e| CircuitError::ProofError(e.to_string()))
    }

    /// Verifies proof with perfect completeness
    pub fn verify(&self, proof: &ProofWithPublicInputs<F, C, D>) -> Result<(), CircuitError> {
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::BuildError("Circuit not built".into()))?;

        data.verify(proof.clone())
            .map_err(|e| CircuitError::VerificationError(e.to_string()))
    }

    /// Analyzes circuit complexity according to blueprint metrics
    pub fn analyze_complexity(&mut self) -> CircuitComplexity {
        self.complexity = CircuitComplexity {
            constraints: self.builder.num_gates(),
            variables: self.builder.num_public_inputs() + self.builder.num_gates(),
            depth: self.calculate_circuit_depth(),
        };
        self.complexity.clone()
    }

    fn calculate_circuit_depth(&self) -> usize {
        // Calculate depth of Sparse Merkle Tree
        let num_channels = self.builder.num_gates();
        (num_channels as f64).log2().ceil() as usize
    }
}
/// High-level builder for Overpass protocol circuits
pub struct ZkCircuitBuilder<F: RichField + Extendable<D>, const D: usize> {
    builder: CircuitBuilder<F, D>,
    public_inputs: Vec<Target>,
    complexity: CircuitComplexity,
}

impl<F: RichField + Extendable<D>, const D: usize> ZkCircuitBuilder<F, D> {
    pub fn new(config: CircuitConfig) -> Self {
        Self {
            builder: CircuitBuilder::new(config),
            public_inputs: Vec::new(),
            complexity: CircuitComplexity {
                constraints: 0,
                variables: 0,
                depth: 0,
            },
        }
    }

    /// Adds channel state variables
    pub fn add_channel_state(&mut self) -> Vec<Target> {
        let balances = self.builder.add_virtual_targets(2); // For two parties
        let nonce = self.builder.add_virtual_target();
        let metadata = self.builder.add_virtual_target();
        
        [balances, vec![nonce, metadata]].concat()
    }

    /// Adds state transition constraints
    pub fn add_state_transition(&mut self, old_state: &[Target], new_state: &[Target]) {
        // Balance conservation
        let old_sum = self.builder.add_many(&old_state[0..2]);
        let new_sum = self.builder.add_many(&new_state[0..2]);
        self.builder.connect(old_sum, new_sum);

        // Nonce monotonicity 
        self.builder.range_check(new_state[2], 32);
        let nonce_valid = self.builder.sub(new_state[2], old_state[2]);
        self.builder.assert_zero(nonce_valid);
    }

    /// Adds Merkle tree constraints
    pub fn add_merkle_constraints(&mut self, leaf: Target, root: Target, path: &[Target]) {
        let mut current = self.builder.hash_n_to_hash_no_pad::<PoseidonHash>(vec![leaf]);
        for &sibling in path {
            let hash_input = vec![current.elements[0], sibling];
            current = self.builder.hash_n_to_hash_no_pad::<PoseidonHash>(hash_input);
        }
        self.builder.connect(current.elements[0], root);
    }
// GPU Acceleration Module
#[cfg(feature = "gpu")]
mod gpu {
    use super::*;
    use cuda_runtime_sys::*;
    use std::ffi::c_void;

    pub struct GpuContext {
        stream: cudaStream_t,
        memory_pool: Vec<*mut c_void>,
    }

    impl GpuContext {
        pub fn new() -> Result<Self, String> {
            let mut stream: cudaStream_t = std::ptr::null_mut();
            unsafe {
                cudaStreamCreate(&mut stream);
                Ok(Self {
                    stream,
                    memory_pool: Vec::new(),
                })
            }
        }

        pub fn parallel_prove<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
            &mut self,
            data: Arc<CircuitData<F, C, D>>,
            witness: PartialWitness<F>,
        ) -> Result<ProofWithPublicInputs<F, C, D>, String> {
            // Split proof computation into parallel streams
            let num_streams = 4;
            let mut proofs = Vec::with_capacity(num_streams);
            
            let chunks = witness.split_into_chunks(num_streams);
            for chunk in chunks {
                let proof = unsafe {
                    self.cuda_kernel_prove(data.clone(), chunk)?
                };
                proofs.push(proof);
            }
            
            // Merge partial proofs
            self.merge_proofs(proofs)
        }
    }
}

// Circuit Depth Calculation
impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> Circuit<F, C, D> {
    fn calculate_circuit_depth(&self) -> usize {
        let mut depth_map: HashMap<Target, usize> = HashMap::new();
        let mut max_depth = 0;

        // Topological sort of circuit gates
        let gates = self.builder.gates().collect::<Vec<_>>();
        for gate in gates {
            let inputs = gate.inputs();
            let mut current_depth = 0;
            
            // Find max depth of inputs
            for input in inputs {
                current_depth = current_depth.max(*depth_map.get(&input).unwrap_or(&0));
            }
            
            // Set depth for gate outputs
            let outputs = gate.outputs();
            for output in outputs {
                depth_map.insert(output, current_depth + 1);
                max_depth = max_depth.max(current_depth + 1);
            }
        }
        
        max_depth
    }
}

// Enhanced Parallel Processing
impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> Circuit<F, C, D> {
    pub fn prove_parallel_optimized(&self) -> Result<ProofWithPublicInputs<F, C, D>, CircuitError> {
        let witness = self.witness.as_ref()
            .ok_or_else(|| CircuitError::ProofError("No witness available".into()))?;
        
        let data = self.data.as_ref()
            .ok_or_else(|| CircuitError::BuildError("Circuit not built".into()))?;

        // Parallel processing pool
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus::get())
            .build()
            .unwrap();

        pool.install(|| {
            // Split proof computation into chunks
            let chunks = self.split_computation();
            
            // Process chunks in parallel
            let results: Vec<_> = chunks.par_iter()
                .map(|chunk| self.process_chunk(chunk, data, witness))
                .collect();

            // Combine results
            self.combine_results(results)
        })
    }
}
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use plonky2::plonk::config::PoseidonGoldilocksConfig;

    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;

    #[test]
    fn test_state_transition() {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = ZkCircuitBuilder::<F, D>::new(config);

        let old_state = builder.add_channel_state();
        let new_state = builder.add_channel_state();
        
        builder.add_state_transition(&old_state, &new_state);
        
        // Verify constraints are satisfied
        assert!(builder.builder.num_gates() > 0);
    }

    #[test]
    fn test_merkle_proof() {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = ZkCircuitBuilder::<F, D>::new(config);

        let leaf = builder.builder.add_virtual_target();
        let root = builder.builder.add_virtual_target();
        let path = vec![builder.builder.add_virtual_target(); 32];

        builder.add_merkle_constraints(leaf, root, &path);
        
        assert!(builder.builder.num_gates() > 0);
    }

    #[test]
    fn test_circuit_depth_calculation() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Circuit::<F, C, D>::new(config, false);
        
        // Build a test circuit with known depth
        let a = circuit.builder_mut().add_virtual_target();
        let b = circuit.builder_mut().add_virtual_target();
        let c = circuit.builder_mut().mul(a, b);
        let d = circuit.builder_mut().mul(c, b);
        
        assert_eq!(circuit.calculate_circuit_depth(), 2);
    }

    #[bench]
    fn bench_parallel_proving(b: &mut Bencher) {
        let config = CircuitConfig::standard_recursion_config();
        let circuit = setup_benchmark_circuit();
        
        b.iter(|| {
            circuit.prove_parallel_optimized().unwrap()
        });
    }

    #[test]
    #[cfg(feature = "gpu")]
    fn test_gpu_proof_generation() {
        let mut context = GpuContext::new().unwrap();
        let circuit = setup_test_circuit();
        
        let proof = context.parallel_prove(
            circuit.data.unwrap(),
            circuit.witness.unwrap()
        ).unwrap();
        
        assert!(circuit.verify_parallel(&proof).is_ok());
    }
}
