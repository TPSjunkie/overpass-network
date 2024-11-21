use std::sync::Arc;
use thiserror::Error;
use plonky2::hash::hash_types::RichField;
use plonky2_field::extension::Extendable;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::config::{PoseidonGoldilocksConfig, GenericConfig};

use crate::core::storage_node::replication::state::{
    StateManager, ReplicationState, ConsistencyState, DistributionState
};
use crate::core::storage_node::replication::consistency::ConsistencyValidator;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
use crate::core::zkps::plonky2::Plonky2System;
use crate::core::types::boc::BOC;
use crate::core::error::errors::SystemError;

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
        VerificationError::StateError(error.to_string())
    }
}

pub type VerificationResult<T> = Result<T, VerificationError>;

pub struct VerificationManager<F: RichField + Extendable<D>> {
    state_manager: Arc<StateManager>,
    consistency_validator: Arc<ConsistencyValidator<Plonky2System>>,
    distribution_manager: Arc<DistributionManager>,
    circuit_builder: ZkCircuitBuilder<F, D>,
    plonky_config: C,
}

impl<F: RichField + Extendable<D>> VerificationManager<F> {
    pub fn new(
        state_manager: StateManager,
        consistency_validator: ConsistencyValidator<Plonky2System>,
        distribution_manager: DistributionManager,
        plonky_config: C,
    ) -> Result<Self, SystemError> {        
        Ok(Self {
            state_manager: Arc::new(state_manager),
            consistency_validator: Arc::new(consistency_validator),
            distribution_manager: Arc::new(distribution_manager),
            circuit_builder: ZkCircuitBuilder::new(GenericConfig::new(plonky_config)),
            plonky_config,
        })
    }

    pub async fn verify(&self) -> VerificationResult<bool> {
        let state = self
            .state_manager
            .in_current_span()
            .map_err(|e| VerificationError::StateError(e.to_string()))?;
            
        if !state.verify_validity() {
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

    async fn build_verification_circuit(&self, state: &ReplicationState) -> VerificationResult<BOC> {
        let mut builder = CircuitBuilder::<F, D>::new(GenericConfig::new(self.plonky_config));
        
        // Add state hash as circuit target
        let hash_target = builder.add_virtual_public_input();
        builder.register_public_input(hash_target);
        
        let circuit = builder.build::<C>();
        
        // Create BOC from circuit data
        let mut boc = BOC::new();
        boc.set_hash(circuit.hash());
        // Note: Assuming BOC has a method to set cells, adjust as needed
        // boc.set_cells(circuit.cells());
        
        Ok(boc)
    }

    async fn verify_consistency(&self, circuit: &BOC) -> VerificationResult<bool> {
        let state_hash = circuit.get_hash();
        let proof = self.state_manager.generate_proof(state_hash)?;
        
        self.consistency_validator
            .verify_consistency(state_hash, &proof, circuit)
            .await
            .map_err(|e| VerificationError::ConsistencyError(e.to_string()))
    }

    async fn verify_distribution(&self, circuit: &BOC) -> VerificationResult<bool> {
        self.distribution_manager
            .verify_distribution(circuit)
            .await
            .map_err(|e| VerificationError::DistributionError(e.to_string()))
    }

    async fn verify_zkp(&self, circuit: &BOC) -> VerificationResult<()> {
        let proof = self.state_manager.generate_proof(circuit.get_hash())?;
        
        self.consistency_validator
            .verify_proof(&proof)
            .await
            .map_err(|e| VerificationError::ZkpError(e.to_string()))
    }
     
    pub fn get_verification_state(&self) -> VerificationResult<VerificationState> {
        let state = self.state_manager.get_current_state()
            .map_err(|e| VerificationError::StateError(e.to_string()))?;
            
        let consistency_state = self.consistency_validator
            .get_current_state()
            .map_err(|e| VerificationError::ConsistencyError(e.to_string()))?;
            
        let distribution_state = self.distribution_manager
            .get_current_state()
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
    use plonky2::plonk::config::GenericConfig;

    type F = <PoseidonGoldilocksConfig as GenericConfig<D>>::F;

    async fn setup_test_verification() -> Result<VerificationManager<F>, SystemError> {
        let state_manager = StateManager::new()?;
        let plonky_system = Arc::new(Plonky2System::default());
        
        let consistency_validator = ConsistencyValidator::new(
            Arc::clone(&plonky_system),
            Arc::new(state_manager.clone())
        );
        
        let distribution_manager = DistributionManager::new(
            Arc::clone(&plonky_system),
            1000, // replication threshold
            5000  // replication interval
        );
        
        VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
            PoseidonGoldilocksConfig::default()
        )
    }

    #[tokio::test]
    async fn test_verification_manager() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        let result = verification_manager.verify().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_verification_state() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        let state = verification_manager.get_verification_state();
        assert!(state.is_ok());
        
        let state = state.unwrap();
        assert!(state.replication_state.verify_validity());
        assert!(state.consistency_state.verify_validity());
        assert!(state.distribution_state.verify_validity());
    }

    #[tokio::test]
    async fn test_invalid_state() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        // Corrupt state by inserting invalid data
        verification_manager.state_manager.corrupt_state().await;
        
        let result = verification_manager.verify().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerificationError::StateError(_)));
    }

    #[tokio::test]
    async fn test_verification_error_handling() {
        let verification_manager = setup_test_verification()
            .await
            .expect("Failed to create verification manager");

        // Force validation error by corrupting consistency validator
        verification_manager.consistency_validator.set_error_state(true).await;
        
        let result = verification_manager.verify().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerificationError::ConsistencyError(_)));
    }
}