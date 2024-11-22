use plonky2::hash::hash_types::RichField;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2_field::extension::Extendable;
use std::sync::Arc;
use thiserror::Error;

use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::replication::consistency::ConsistencyValidator;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::storage_node::replication::state::{
    ConsistencyState, DistributionState, ReplicationState, StateManager,
};
use crate::core::types::boc::BOC;
use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
use crate::core::zkps::plonky2::Plonky2System;

const D: usize = 2; // Extension degree
type C = PoseidonGoldilocksConfig;

#[derive(Error, Debug)]
pub enum VerificationError {
    #[error("Consistency verification failed: {0}")]
    ConsistencyError(String),
    #[error("Distribution verification failed: {0}")]
    DistributionError(String),
    #[error("State validation failed: {0}")]
    StateError(String),
    #[error("ZKP verification failed: {0}")]
    ZkpError(String),
}

impl From<SystemError> for VerificationError {
    fn from(error: SystemError) -> Self {
        match error.error_type {
            SystemErrorType::ConsistencyError => VerificationError::ConsistencyError(error.message),
            SystemErrorType::DistributionError => {
                VerificationError::DistributionError(error.message)
            }
            SystemErrorType::VerificationError => VerificationError::ZkpError(error.message),
            _ => VerificationError::StateError(error.message),
        }
    }
}

pub type VerificationResult<T> = Result<T, VerificationError>;

pub struct VerificationManager<F: RichField + Extendable<D>> {
    state_manager: Arc<StateManager>,
    consistency_validator: Arc<ConsistencyValidator<F, Plonky2System>>,
    distribution_manager: Arc<DistributionManager>,
    circuit_builder: ZkCircuitBuilder<F, D>,
    plonky_config: C,
}
impl<F: RichField + Extendable<D>> VerificationManager<F> {
    pub fn new(
        state_manager: StateManager,
        consistency_validator: ConsistencyValidator<F, Plonky2System>,
        distribution_manager: DistributionManager,
        plonky_config: C,
    ) -> Result<Self, SystemError> {
        let config = CircuitConfig::standard_recursion_config();

        Ok(Self {
            state_manager: Arc::new(state_manager),
            consistency_validator: Arc::new(consistency_validator),
            distribution_manager: Arc::new(distribution_manager),
            circuit_builder: ZkCircuitBuilder::new(config),
            plonky_config,
        })
    }

    pub async fn verify(&self) -> VerificationResult<bool> {
        let state = self
            .state_manager
            .get_replication_state()
            .map_err(|e| VerificationError::StateError(e.to_string()))?;

        if !state.verify() {
            return Err(VerificationError::StateError(
                "Invalid replication state".to_string(),
            ));
        }

        let circuit = self.build_verification_circuit(&state).await?;

        let consistency_result = self.verify_consistency(&circuit).await?;
        let distribution_result = self.verify_distribution(&circuit).await?;

        self.verify_zkp(&circuit).await?;

        Ok(consistency_result && distribution_result)
    }

    async fn build_verification_circuit(
        &self,
        state: &ReplicationState,
    ) -> VerificationResult<BOC> {
        let mut builder = CircuitBuilder::<F, D>::new(CircuitConfig::standard_recursion_config());

        let hash_target = builder.add_virtual_target();
        builder.register_public_inputs(&[hash_target]);

        let circuit = builder.build();

        let mut boc = BOC::new();
        boc.set_hash(circuit.compute_root_hash());
        boc.set_data(&circuit.serialize());

        Ok(boc)
    }

    async fn verify_consistency(&self, circuit: &BOC) -> VerificationResult<bool> {
        let state_hash = circuit.hash();
        let proof = self.state_manager.generate_proof(state_hash)?;

        self.consistency_validator
            .validate_consistency(state_hash, &proof, circuit)
            .await
            .map_err(|e| VerificationError::ConsistencyError(e.to_string()))
    }

    async fn verify_distribution(&self, circuit: &BOC) -> VerificationResult<bool> {
        self.distribution_manager
            .validate_state_distribution(circuit)
            .await
            .map_err(|e| VerificationError::DistributionError(e.to_string()))
    }

    async fn verify_zkp(&self, circuit: &BOC) -> VerificationResult<()> {
        let proof = self.state_manager.generate_proof(circuit.hash())?;

        self.consistency_validator
            .verify_proof(&proof)
            .await
            .map_err(|e| VerificationError::ZkpError(e.to_string()))
    }

    pub fn get_verification_state(&self) -> VerificationResult<VerificationState> {
        let state = self
            .state_manager
            .get_replication_state()
            .map_err(|e| VerificationError::StateError(e.to_string()))?;

        let consistency_state = self
            .consistency_validator
            .get_consistency_state()
            .map_err(|e| VerificationError::ConsistencyError(e.to_string()))?;

        let distribution_state = self
            .distribution_manager
            .get_distribution_state()
            .map_err(|e| VerificationError::DistributionError(e.to_string()))?;

        Ok(VerificationState {
            replication_state: state,
            consistency_state,
            distribution_state,
        })
    }
}

#[derive(Clone, Debug)]
pub struct VerificationState {
    pub replication_state: ReplicationState,
    pub consistency_state: ConsistencyState,
    pub distribution_state: DistributionState,
}
#[cfg(test)]
mod tests {
    use super::*;
    use plonky2::field::goldilocks_field::GoldilocksField;
    use wasm_bindgen_test::*;

    type F = GoldilocksField;

    async fn setup_test_verification() -> Result<VerificationManager<F>, SystemError> {
        let state_manager = StateManager::create()
            .await
            .expect("Failed to create state manager");

        let plonky_system = Arc::new(Plonky2System::new(Default::default())?);

        let consistency_validator = ConsistencyValidator::new(
            plonky_system.clone(),
            Default::default(), // Pass default proof config
        );

        let distribution_manager = DistributionManager::new(
            1000,               // replication threshold
            5000,               // replication interval
            Default::default(), // Pass default distribution config
        );

        VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
            PoseidonGoldilocksConfig::new(),
        )
    }

    #[wasm_bindgen_test]
    async fn test_verification_manager() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        // Create test state
        let state = ReplicationState::new_test();
        verification_manager
            .state_manager
            .store_replication_state(&state)
            .await
            .expect("Failed to store test state");

        let result = verification_manager.verify().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[wasm_bindgen_test]
    async fn test_verification_state() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        // Store test states
        let replication_state = ReplicationState::new_test();
        let consistency_state = ConsistencyState::new_test();
        let distribution_state = DistributionState::new_test();

        verification_manager
            .state_manager
            .store_replication_state(&replication_state)
            .await
            .expect("Failed to store replication state");

        verification_manager
            .consistency_validator
            .store_consistency_state(&consistency_state)
            .await
            .expect("Failed to store consistency state");

        verification_manager
            .distribution_manager
            .store_distribution_state(&distribution_state)
            .await
            .expect("Failed to store distribution state");

        let state = verification_manager.get_verification_state().await;
        assert!(state.is_ok());

        let state = state.unwrap();
        assert!(state.replication_state.verify());
        assert!(state.consistency_state.verify());
        assert!(state.distribution_state.verify());
    }

    #[wasm_bindgen_test]
    async fn test_invalid_state() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        // Store invalid state
        let mut invalid_state = ReplicationState::new_test();
        invalid_state.invalidate(); // Mark state as invalid

        verification_manager
            .state_manager
            .store_replication_state(&invalid_state)
            .await
            .expect("Failed to store invalid state");

        let result = verification_manager.verify().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VerificationError::StateError(_)
        ));
    }

    #[wasm_bindgen_test]
    async fn test_verification_error_handling() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        // Store valid state but force validation error
        let state = ReplicationState::new_test();
        verification_manager
            .state_manager
            .store_replication_state(&state)
            .await
            .expect("Failed to store test state");

        verification_manager
            .consistency_validator
            .set_force_verification_error(true)
            .await;

        let result = verification_manager.verify().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            VerificationError::ConsistencyError(_)
        ));
    }

    #[wasm_bindgen_test]
    async fn test_circuit_building() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        let state = ReplicationState::new_test();
        let circuit = verification_manager
            .build_verification_circuit(&state)
            .await;

        assert!(circuit.is_ok());
        let circuit = circuit.unwrap();
        assert!(circuit.verify_structure());
    }

    #[wasm_bindgen_test]
    async fn test_proof_verification() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        let state = ReplicationState::new_test();
        let circuit = verification_manager
            .build_verification_circuit(&state)
            .await
            .expect("Failed to build circuit");

        let proof_result = verification_manager.verify_zkp(&circuit).await;
        assert!(proof_result.is_ok());
    }
}
