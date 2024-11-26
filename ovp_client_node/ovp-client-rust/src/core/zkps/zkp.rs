
    //! # Zero Knowledge Proof Circuit Module
    //!
    //! This module provides a high-level abstraction for building and working with zero-knowledge proof circuits
    //! using the Plonky2 proving system.
    //!
    //! ## Core Components
    //!
    //! * `Circuit` - The main circuit structure that handles proof generation and verification
    //! * `VirtualCell` - Represents a cell in the arithmetic circuit with optional rotation
    //! * `ZkCircuitBuilder` - A builder pattern implementation for constructing ZK circuits
    //! * `CircuitError` - Error types specific to circuit operations
    //!
    //! ## Features
    //!
    //! * Arithmetic operations (add, subtract, multiply, divide)
    //! * Public and private inputs
    //! * Range checks
    //! * Poseidon hash computations
    //! * Circuit verification
    //! * Witness generation
    //!
    //! ## Example Usage
    //!
    //! 
    //! let mut builder = ZkCircuitBuilder::new(CircuitConfig::standard_recursion_config());
    //! let a = builder.add_public_input();
    //! let b = builder.add_public_input();
    //! let sum = builder.add(a, b);
    //! 
    //!
    //! ## Error Handling
    //!
    //! The module uses a custom `CircuitError` enum that covers various failure modes:
    //! * Circuit building errors
    //! * Proof generation failures
    //! * Invalid target/witness errors
    //! * Verification failures

    use crate::common::error::client_errors::{SystemError, SystemErrorType};
    use crate::core::zkps::circuit_builder::Circuit;
    use plonky2::{
        field::extension::Extendable,
        hash::hash_types::RichField,
        plonk::{
            config::GenericConfig,
            proof::ProofWithPublicInputs,
        },
    };
    use serde::{Deserialize, Serialize};
    
    #[derive(Clone, Debug, Serialize, Deserialize, Default)]
    pub struct ZkProof {
        pub proof_data: Vec<u8>,
        pub public_inputs: Vec<u64>,
        pub merkle_root: Vec<u8>,
        pub timestamp: u64,
    }
    
    #[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum ProofType {
        StateTransition = 0,
        BalanceTransfer = 1,
        MerkleInclusion = 2,
    }
    
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ProofMetadata {
        pub proof_type: ProofType,
        pub channel_id: Option<[u8; 32]>,
        pub created_at: u64,
        pub verified_at: Option<u64>,
        pub version: i32,
        pub height_bounds: (u64, u64),
    }
    
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct ProofBundle {
        pub proof: ZkProof,
        pub metadata: ProofMetadata,
    }
    
    pub struct ProofVerifier<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
        circuit: Circuit<F, C, D>,
    }
    
    impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> ProofVerifier<F, C, D> {
        pub fn new(circuit: Circuit<F, C, D>) -> Self {
            Self { circuit }
        }
    
        pub fn verify(&self, proof: &ZkProof) -> Result<bool, SystemError> {
            let proof_data = bincode::deserialize::<ProofWithPublicInputs<F, C, D>>(&proof.proof_data)
                .map_err(|e| SystemError::new(
                    SystemErrorType::SerializationError,
                    format!("Failed to deserialize proof: {}", e),
                ))?;
    
            self.circuit.verify(&proof_data)
                .map_err(|e| SystemError::new(
                    SystemErrorType::VerificationError,
                    format!("Proof verification failed: {}", e),
                ))?;
    
            Ok(true)
        }    }
    
    impl ZkProof {
        pub fn new(proof_data: Vec<u8>, public_inputs: Vec<u64>) -> Result<Self, SystemError> {
            if proof_data.is_empty() {
                return Err(SystemError::new(
                    SystemErrorType::InvalidInput,
                    "Proof data cannot be empty".to_string(),
                ));
            }
    
            Ok(Self {
                proof_data,
                public_inputs,
                merkle_root: Vec::new(),
                timestamp: current_timestamp(),
            })
        }
    
        pub fn generate<F, C, const D: usize>(
            circuit: &mut Circuit<F, C, D>,
            public_inputs: &[F],
        ) -> Result<Self, SystemError>
        where
            F: RichField + Extendable<D>,
            C: GenericConfig<D, F = F>,
        {
            // Set public inputs
            for (i, &input) in public_inputs.iter().enumerate() {
                let target = circuit.add_virtual_target();
                circuit.set_public_input(target, input)
                    .map_err(|e| SystemError::new(
                        SystemErrorType::ProofGenerationError,
                        format!("Failed to set public input {}: {}", i, e),
                    ))?;
            }

            // Build circuit if not already built
            circuit.build()
                .map_err(|e| SystemError::new(
                    SystemErrorType::CircuitError,
                    format!("Failed to build circuit: {}", e),
                ))?;

            // Generate proof
            let proof = circuit.prove()
                .map_err(|e| SystemError::new(
                    SystemErrorType::ProofGenerationError,
                    format!("Failed to generate proof: {}", e),
                ))?;

            // Serialize proof
            let proof_data = bincode::serialize(&proof)
                .map_err(|e| SystemError::new(
                    SystemErrorType::SerializationError,
                    format!("Failed to serialize proof: {}", e),
                ))?;

            // Convert public inputs to u64
            let public_inputs = public_inputs.iter()
                .map(|&x| x.to_canonical_u64())
                .collect();

            Ok(Self {
                proof_data,
                public_inputs,
                merkle_root: Vec::new(),
                timestamp: current_timestamp(),
            })
        }
    
        pub fn verify<F, C, const D: usize>(
            &self,
            verifier: &ProofVerifier<F, C, D>,
        ) -> Result<bool, SystemError>
        where
            F: RichField + Extendable<D>,
            C: GenericConfig<D, F = F>,
        {
            verifier.verify(self)
        }
    }    
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
    
    #[cfg(test)]
    mod tests {
        use plonky2::plonky2_field::types::Field;
use super::*;
                use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
                use plonky2::{
                    field::{goldilocks_field::GoldilocksField, types::Field64},
                    plonk::{
                        circuit_data::CircuitConfig,
                        config::PoseidonGoldilocksConfig,
                    },
                };
        type F = GoldilocksField;
        type C = PoseidonGoldilocksConfig;
        const D: usize = 2;
    
        fn setup_test_circuit() -> (Circuit<F, C, D>, Vec<F>) {
            let config = CircuitConfig::standard_recursion_config();
            let mut builder = ZkCircuitBuilder::new(config);
    
            let old_balance = builder.add_public_input();
            let new_balance = builder.add_public_input();
            let amount = builder.add_public_input();
    
            // Create constraints
            let computed_new_balance = builder.sub(old_balance.clone(), amount.clone());
            builder.assert_equal(computed_new_balance, new_balance.clone());
    
            let circuit = builder.build().expect("Failed to build circuit");
            let public_inputs = vec![
                F::from_canonical_u64(1000), // old_balance
                F::from_canonical_u64(900),  // new_balance
                F::from_canonical_u64(100),  // amount
            ];
    
            (circuit, public_inputs)
        }        #[test]
        fn test_proof_generation_and_verification() {
            let (mut circuit, public_inputs) = setup_test_circuit();
    
            // Generate proof
            let zk_proof = ZkProof::generate(&mut circuit, &public_inputs)
                .expect("Failed to generate proof");
    
            // Create verifier
            let verifier = ProofVerifier::new(circuit);
    
            // Verify proof
            let verification_result = zk_proof.verify(&verifier)
                .expect("Failed to verify proof");
    
            assert!(verification_result, "Proof verification failed");
        }
    
        #[test]
        fn test_invalid_proof() {
            let (circuit, _) = setup_test_circuit();
            let verifier = ProofVerifier::new(circuit);
    
            // Create invalid proof
            let invalid_proof = ZkProof {
                proof_data: vec![0; 32],
                public_inputs: vec![1000, 900, 100],
                merkle_root: vec![0; 32],
                timestamp: current_timestamp(),
            };
    
            // Verify should fail
            assert!(invalid_proof.verify(&verifier).is_err());
        }
    }