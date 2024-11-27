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
use serde::{Deserialize, Serialize, Serializer};

#[cfg(feature = "rayon")]
use rayon::prelude::*;
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
    }    
}

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
        for (i, &input) in public_inputs.iter().enumerate() {
            let target = circuit.add_virtual_target();
            circuit.set_public_input(target, input)
                .map_err(|e| SystemError::new(
                    SystemErrorType::ProofGenerationError,
                    format!("Failed to set public input {}: {}", i, e),
                ))?;
        }

        circuit.build()
            .map_err(|e| SystemError::new(
                SystemErrorType::CircuitError,
                format!("Failed to build circuit: {}", e),
            ))?;

        let proof = circuit.prove_parallel()
            .map_err(|e| SystemError::new(
                SystemErrorType::ProofGenerationError,
                format!("Failed to generate proof: {}", e),
            ))?;

        let proof_data = bincode::serialize(&proof)
            .map_err(|e| SystemError::new(
                SystemErrorType::SerializationError,
                format!("Failed to serialize proof: {}", e),
            ))?;

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
use plonky2::plonky2_field::types::Field;
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
        let mut circuit = Circuit::<F, C, D>::new(config);
        
        let public_inputs = vec![
            F::from_canonical_u64(100),
            F::from_canonical_u64(50),
        ];

        let proof = ZkProof::generate(&mut circuit, &public_inputs).unwrap();
        let verifier = ProofVerifier::new(circuit);
        assert!(proof.verify(&verifier).unwrap());
    }

    #[test]
    fn test_invalid_proof() {
        let config = CircuitConfig::standard_recursion_config();
        let circuit = Circuit::<F, C, D>::new(config);
        let verifier = ProofVerifier::new(circuit);

        let invalid_proof = ZkProof {
            proof_data: vec![0; 32],
            public_inputs: vec![100, 50],
            merkle_root: vec![0; 32],
            timestamp: current_timestamp(),
        };

        assert!(invalid_proof.verify(&verifier).is_err());
    }
}
#[test]
fn test_parallel_proof_generation() {
    let config = CircuitConfig::standard_recursion_config();
    let mut circuit = Circuit::<F, C, { D }>::new(config);
    
    let public_inputs: Vec<F> = (0..1000).map(|i| 
        F::from_canonical_u64(i as u64)
    ).collect();

    let proof = ZkProof::generate(&mut circuit, &public_inputs).unwrap();
    let verifier = ProofVerifier::new(circuit);
    assert!(proof.verify(&verifier).unwrap());
}

#[test]
fn test_proof_metadata() {
    let config = CircuitConfig::standard_recursion_config();
    let mut circuit = Circuit::<F, C, D>::new(config);
    
    let public_inputs = vec![F::from_canonical_u64(123)];
    let proof = ZkProof::generate(&mut circuit, &public_inputs).unwrap();
    
    let metadata = ProofMetadata {
        proof_type: ProofType::StateTransition,
        channel_id: Some([0u8; 32]),
        created_at: proof.timestamp,
        verified_at: None,
        version: 1,
        height_bounds: (0, 1000),
    };

    let bundle = ProofBundle { proof, metadata };
    assert_eq!(bundle.metadata.proof_type, ProofType::StateTransition);
}

#[test]
fn test_merkle_root_verification() {
    let config = CircuitConfig::standard_recursion_config();
    let mut circuit = Circuit::<F, C, D>::new(config);
    
    let public_inputs = vec![F::from_canonical_u64(123)];
    let mut proof = ZkProof::generate(&mut circuit, &public_inputs).unwrap();
    
    let merkle_root = vec![1u8; 32];
    proof.merkle_root = merkle_root.clone();
    
    let verifier = ProofVerifier::new(circuit);
    assert!(proof.verify(&verifier).unwrap());
    assert_eq!(proof.merkle_root, merkle_root);
}

