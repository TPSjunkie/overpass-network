//! Proof Implementation for Overpass Protocol
//! 
//! Implements zero-knowledge proof generation, verification, and aggregation with
//! security parameter λ ≥ 128 bits as specified in the Overpass protocol blueprint.

use crate::core::zkps::{
    zkp_interface::ProofWithMetadataJS,
    circuit_builder::Circuit,
    plonky2::Circuit as Plonky2Circuit,
};
use serde::{Deserialize, Serialize};
use crate::common::error::client_errors::{SystemError, SystemErrorType};
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    plonk::{
        config::GenericConfig,
        proof::ProofWithPublicInputs,
    },
};
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a zero-knowledge proof with associated data
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ZkProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<u64>,
    pub merkle_root: Vec<u8>,
    pub timestamp: u64,
    pub security_bits: usize,
}

/// Types of proofs supported by the protocol
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProofType {
    StateTransition = 0,
    BalanceTransfer = 1,
    MerkleInclusion = 2,
    CrossChain = 3,
    Recursive = 4,
}

/// Metadata associated with a proof for tracking and verification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofMetadata {
    pub proof_type: ProofType,
    pub channel_id: Option<[u8; 32]>,
    pub created_at: u64,
    pub verified_at: Option<u64>,
    pub version: i32,
    pub height_bounds: (u64, u64),
    pub security_bits: usize,
}

/// Bundle containing a proof and its recursive components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofBundle {
    pub proof: ZkProof,
    pub metadata: ProofMetadata,
    pub recursive_proofs: Vec<ProofBundle>,
}

/// Verifies proofs with security bound 2^-λ
pub struct ProofVerifier<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> {
    circuit: Plonky2Circuit<F, C, D>,
}

impl<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize> ProofVerifier<F, C, D> {
    /// Creates a new verifier with given circuit
    pub fn new(circuit: Plonky2Circuit<F, C, D>) -> Self {
        Self { circuit }
    }

    /// Verifies proof in constant time as specified in paper
    pub fn verify(&self, proof: &ZkProof) -> Result<bool, SystemError> {
        // Verify security parameter
        if proof.security_bits < 128 {
            return Err(SystemError::new(
                SystemErrorType::SecurityError,
                "Security parameter λ must be at least 128 bits".into()
            ));
        }

        // Deserialize and verify proof
        let proof_data = bincode::deserialize::<ProofWithPublicInputs<F, C, D>>(&proof.proof_data)
            .map_err(|e| SystemError::new(
                SystemErrorType::SerializationError,
                format!("Failed to deserialize proof: {}", e),
            ))?;

        self.circuit.verify_constant_time(&proof_data)
            .map_err(|e| SystemError::new(
                SystemErrorType::VerificationError, 
                format!("Proof verification failed: {}", e),
            ))?;

        Ok(true)
    }

    /// Verifies recursive proof bundle
    pub fn verify_recursive(&self, bundle: &ProofBundle) -> Result<bool, SystemError> {
        // Verify main proof
        self.verify(&bundle.proof)?;

        // Verify all recursive proofs
        for recursive_proof in &bundle.recursive_proofs {
            self.verify_recursive(recursive_proof)?;
        }

        Ok(true)
    }
}

impl ZkProof {
    /// Creates new proof with required security parameter
    pub fn new(
        proof_data: Vec<u8>,
        public_inputs: Vec<u64>,
        security_bits: usize,
    ) -> Result<Self, SystemError> {
        // Validate inputs
        if proof_data.is_empty() {
            return Err(SystemError::new(
                SystemErrorType::InvalidInput,
                "Proof data cannot be empty".into(),
            ));
        }

        if security_bits < 128 {
            return Err(SystemError::new(
                SystemErrorType::SecurityError,
                "Security parameter λ must be at least 128 bits".into(),
            ));
        }

        Ok(Self {
            proof_data,
            public_inputs,
            merkle_root: Vec::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            security_bits,
        })
    }

    /// Generates proof with O(log n) complexity
    pub fn generate<F, C, const D: usize>(
        circuit: &mut Plonky2Circuit<F, C, D>,
        public_inputs: &[F],
    ) -> Result<Self, SystemError>
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>
    {
        // Set public inputs
        for (i, input) in public_inputs.iter().enumerate() {
            circuit.set_public_input(i, *input).map_err(|e| SystemError::new(
                SystemErrorType::CircuitError,
                format!("Failed to set public input {}: {}", i, e)
            ))?;
        }

        // Build and prove circuit
        circuit.build().map_err(|e| SystemError::new(
            SystemErrorType::CircuitError,
            format!("Failed to build circuit: {}", e)
        ))?;

        let proof = circuit.prove_parallel().map_err(|e| SystemError::new(
            SystemErrorType::ProofGenerationError,
            format!("Failed to generate proof: {}", e)
        ))?;

        // Serialize proof
        let proof_data = bincode::serialize(&proof).map_err(|e| SystemError::new(
            SystemErrorType::SerializationError,
            format!("Failed to serialize proof: {}", e)
        ))?;

        // Convert public inputs
        let public_inputs = public_inputs.iter()
            .map(|&x| x.to_canonical_u64())
            .collect();

        Self::new(proof_data, public_inputs, 128)
    }

    /// Verifies proof with 2^-λ security bound
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

    /// Aggregates multiple proofs into single proof
    pub fn aggregate<F, C, const D: usize>(
        proofs: Vec<Self>,
        circuit: &mut Plonky2Circuit<F, C, D>,
    ) -> Result<Self, SystemError>
    where
        F: RichField + Extendable<D>,
        C: GenericConfig<D, F = F>,
    {
        // Deserialize proofs
        let proof_data: Vec<ProofWithPublicInputs<F, C, D>> = proofs.iter()
            .map(|p| bincode::deserialize(&p.proof_data))
            .collect::<Result<_, _>>()
            .map_err(|e| SystemError::new(
                SystemErrorType::DeserializationError,
                format!("Failed to deserialize proofs for aggregation: {}", e)
            ))?;

        // Aggregate proofs
        let aggregated = circuit.aggregate_proofs(proof_data).map_err(|e| SystemError::new(
            SystemErrorType::ProofGenerationError,
            format!("Failed to aggregate proofs: {}", e)
        ))?;

        // Serialize aggregated proof
        let proof_data = bincode::serialize(&aggregated).map_err(|e| SystemError::new(
            SystemErrorType::SerializationError,
            format!("Failed to serialize aggregated proof: {}", e)
        ))?;

        // Combine public inputs
        let public_inputs = proofs.into_iter()
            .flat_map(|p| p.public_inputs)
            .collect();

        Self::new(proof_data, public_inputs, 128)
    }
}

impl ProofBundle {
    /// Creates new proof bundle
    pub fn new(
        proof: ZkProof,
        proof_type: ProofType,
        channel_id: Option<[u8; 32]>,
    ) -> Self {
        let metadata = ProofMetadata {
            proof_type,
            channel_id,
            created_at: proof.timestamp,
            verified_at: None,
            version: 1,
            height_bounds: (0, 0),
            security_bits: proof.security_bits,
        };

        Self {
            proof,
            metadata,
            recursive_proofs: Vec::new(),
        }
    }

    /// Adds recursive proof to bundle
    pub fn add_recursive_proof(&mut self, recursive: ProofBundle) {
        self.recursive_proofs.push(recursive);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        plonk::{
            circuit_data::CircuitConfig,
            config::PoseidonGoldilocksConfig,
        },
    };

    type F = GoldilocksField;
    type C = PoseidonGoldilocksConfig;
    const D: usize = 2;

    #[test]
    fn test_proof_generation_and_verification() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Plonky2Circuit::<F, C, D>::new(config);
        
        let public_inputs = vec![
            F::from_canonical_u64(100),
            F::from_canonical_u64(50),
        ];

        let proof = ZkProof::generate(&mut circuit, &public_inputs).unwrap();
        let verifier = ProofVerifier::new(circuit);
        assert!(proof.verify(&verifier).unwrap());
    }

    #[test]
    fn test_proof_aggregation() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Plonky2Circuit::<F, C, D>::new(config);
        
        circuit.enable_recursion().unwrap();

        let proofs = (0..4).map(|i| {
            let inputs = vec![F::from_canonical_u64(i as u64)];
            ZkProof::generate(&mut circuit, &inputs).unwrap()
        }).collect::<Vec<_>>();

        let aggregated = ZkProof::aggregate(proofs, &mut circuit).unwrap();
        let verifier = ProofVerifier::new(circuit);
        assert!(aggregated.verify(&verifier).unwrap());
    }

    #[test]
    fn test_recursive_proof_bundle() {
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Plonky2Circuit::<F, C, D>::new(config);
        
        let inputs = vec![F::from_canonical_u64(123)];
        let proof = ZkProof::generate(&mut circuit, &inputs).unwrap();
        
        let mut bundle = ProofBundle::new(
            proof,
            ProofType::StateTransition,
            Some([0u8; 32])
        );

        let recursive_proof = ZkProof::generate(&mut circuit, &inputs).unwrap();
        let recursive_bundle = ProofBundle::new(
            recursive_proof,
            ProofType::Recursive,
            None
        );

        bundle.add_recursive_proof(recursive_bundle);
        
        let verifier = ProofVerifier::new(circuit);
        assert!(verifier.verify_recursive(&bundle).unwrap());
    }

    #[test]
    fn test_security_parameter() {
        // Valid security parameter
        let proof = ZkProof::new(
            vec![1, 2, 3],
            vec![100, 200],
            128
        ).unwrap();
        assert_eq!(proof.security_bits, 128);

        // Invalid security parameter
        let result = ZkProof::new(
            vec![1, 2, 3],
            vec![100, 200],
            64
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_constant_time_verification() {
        use std::time::Instant;
        
        let config = CircuitConfig::standard_recursion_config();
        let mut circuit = Plonky2Circuit::<F, C, D>::new(config);
        
        // Generate proofs with different input sizes
        let small_inputs = vec![F::from_canonical_u64(100)];
        let large_inputs = vec![F::from_canonical_u64(u64::MAX)];
        
        let small_proof = ZkProof::generate(&mut circuit, &small_inputs).unwrap();
        let large_proof = ZkProof::generate(&mut circuit, &large_inputs).unwrap();
        
        let verifier = ProofVerifier::new(circuit);
        
        // Measure verification times
        let start = Instant::now();
        verifier.verify(&small_proof).unwrap();
        let small_time = start.elapsed();
        
        let start = Instant::now();
        verifier.verify(&large_proof).unwrap();
        let large_time = start.elapsed();
        
        // Verification times should be very close
        let time_diff = if small_time > large_time {
            small_time - large_time
        } else {
            large_time - small_time
        };
        
        assert!(time_diff.as_micros() < 1000, "Verification should be constant time");
    }
}