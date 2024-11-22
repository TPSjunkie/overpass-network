use crate::core::zkp::{Circuit, Witness, PublicInputs, Proof};
use crate::core::boc::StateBOC;

pub struct TestProofContext {
    pub old_state: StateBOC,
    pub new_state: StateBOC,
    pub update: StateUpdate,
    pub circuit: Circuit,
    pub witness: Witness,
    pub public_inputs: PublicInputs,
}

impl TestProofContext {
    pub fn new() -> Self {
        // Create initial test state
        let old_state = StateBOC::new(vec![
            // Initial balance
            0, 0, 0, 100,
            // Initial nonce
            0, 0, 0, 1,
        ]);

        // Create updated state
        let new_state = StateBOC::new(vec![
            // Updated balance
            0, 0, 0, 200,
            // Updated nonce
            0, 0, 0, 2,
        ]);

        // Create state update
        let update = StateUpdate {
            channel_id: [0u8; 32],
            op_code: OpCode::UpdateBalance(200),
            nonce: 2,
        };

        // Create circuit for testing
        let circuit = Circuit {
            gates: vec![
                Gate::Add(WitnessRef(0), WitnessRef(1), WitnessRef(2)),
                Gate::Hash(vec![WitnessRef(0), WitnessRef(1)], WitnessRef(3)),
            ],
            witnesses: vec![
                WitnessRef(0),
                WitnessRef(1),
                WitnessRef(2),
                WitnessRef(3),
            ],
            public_inputs: vec![
                PublicInputRef(0),
                PublicInputRef(1),
            ],
        };

        // Create witness
        let witness = Witness {
            old_state: old_state.serialize().unwrap(),
            new_state: new_state.serialize().unwrap(),
            update_data: update.serialize().unwrap(),
        };

        // Create public inputs
        let public_inputs = PublicInputs {
            old_root: old_state.compute_hash(),
            new_root: new_state.compute_hash(),
            channel_id: update.channel_id,
        };

        Self {
            old_state,
            new_state,
            update,
            circuit,
            witness,
            public_inputs,
        }
    }

    pub async fn generate_test_proof(&self, generator: &ProofGenerator) -> Result<Proof, ZkpError> {
        generator.generate_state_proof(
            &self.circuit,
            &self.witness,
            &self.public_inputs,
        ).await
    }

    pub fn verify_test_proof(&self, generator: &ProofGenerator, proof: &Proof) -> Result<bool, ZkpError> {
        generator.verify_proof(proof, &self.public_inputs)
    }
}

// Performance test utilities
pub struct PerformanceTest {
    pub num_iterations: usize,
    pub proof_sizes: Vec<usize>,
    pub generation_times: Vec<Duration>,
    pub verification_times: Vec<Duration>,
}

impl PerformanceTest {
    pub fn new(num_iterations: usize) -> Self {
        Self {
            num_iterations,
            proof_sizes: Vec::with_capacity(num_iterations),
            generation_times: Vec::with_capacity(num_iterations),
            verification_times: Vec::with_capacity(num_iterations),
        }
    }

    pub async fn run_proof_performance_test(&mut self, generator: &ProofGenerator) -> Result<(), ZkpError> {
        let context = TestProofContext::new();

        for _ in 0..self.num_iterations {
            // Measure proof generation time
            let gen_start = Instant::now();
            let proof = context.generate_test_proof(generator).await?;
            let gen_time = gen_start.elapsed();

            // Measure proof verification time
            let verify_start = Instant::now();
            context.verify_test_proof(generator, &proof)?;
            let verify_time = verify_start.elapsed();

            // Record metrics
            self.proof_sizes.push(proof.data.len());
            self.generation_times.push(gen_time);
            self.verification_times.push(verify_time);
        }

        Ok(())
    }

    pub fn print_statistics(&self) {
        println!("ZKP Performance Statistics:");
        println!("===========================");
        println!("Number of iterations: {}", self.num_iterations);
        println!();
        
        println!("Proof Generation Times:");
        println!("  Average: {:?}", self.average_generation_time());
        println!("  Min: {:?}", self.min_generation_time());
        println!("  Max: {:?}", self.max_generation_time());
        println!();
        
        println!("Proof Verification Times:");
        println!("  Average: {:?}", self.average_verification_time());
        println!("  Min: {:?}", self.min_verification_time());
        println!("  Max: {:?}", self.max_verification_time());
        println!();
        
        println!("Proof Sizes:");
        println!("  Average: {} bytes", self.average_proof_size());
        println!("  Min: {} bytes", self.min_proof_size());
        println!("  Max: {} bytes", self.max_proof_size());
    }

    fn average_generation_time(&self) -> Duration {
        let sum: Duration = self.generation_times.iter().sum();
        sum / self.num_iterations as u32
    }

    fn average_verification_time(&self) -> Duration {
        let sum: Duration = self.verification_times.iter().sum();
        sum / self.num_iterations as u32
    }

    fn average_proof_size(&self) -> usize {
        self.proof_sizes.iter().sum::<usize>() / self.num_iterations
    }

    // Helper methods for min/max calculations
    fn min_generation_time(&self) -> Duration {
        self.generation_times.iter().min().unwrap().clone()
    }

    fn max_generation_time(&self) -> Duration {
        self.generation_times.iter().max().unwrap().clone()
    }

    fn min_verification_time(&self) -> Duration {
        self.verification_times.iter().min().unwrap().clone()
    }

    fn max_verification_time(&self) -> Duration {
        self.verification_times.iter().max().unwrap().clone()
    }

    fn min_proof_size(&self) -> usize {
        *self.proof_sizes.iter().min().unwrap()
    }

    fn max_proof_size(&self) -> usize {
        *self.proof_sizes.iter().max().unwrap()
    }
}
