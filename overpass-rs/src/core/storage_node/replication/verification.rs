use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;

use crate::core::storage_node::replication::state::{
    StateManager, ReplicationState, ConsistencyState, DistributionState
};
use crate::core::storage_node::replication::consistency::ConsistencyValidator;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
use crate::core::zkps::plonky2::Plonky2System;
use crate::core::types::boc::BOC;
use crate::core::error::errors::SystemError;

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

pub type VerificationResult<T> = Result<T, VerificationError>;

pub struct VerificationManager<F: RichField + Extendable<D>, const D: usize> {
    state_manager: Arc<StateManager>,
    consistency_validator: Arc<ConsistencyValidator<Plonky2System>>,
    distribution_manager: Arc<DistributionManager>,
    circuit_builder: ZkCircuitBuilder<F, D>,
}

impl<F: RichField + Extendable<D>, const D: usize> VerificationManager<F, D> {
    pub fn new(
        state_manager: StateManager,
        consistency_validator: ConsistencyValidator<Plonky2System>,
        distribution_manager: DistributionManager,
        config: CircuitConfig,
    ) -> Result<Self, SystemError> {
        Ok(Self {
            state_manager: Arc::new(state_manager),
            consistency_validator: Arc::new(consistency_validator),
            distribution_manager: Arc::new(distribution_manager),
            circuit_builder: ZkCircuitBuilder::new(config),
        })
    }

    pub async fn verify(&self) -> VerificationResult<bool> {
        let state = self.state_manager.get_state()?;
        if !state.is_valid() {
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
        self.circuit_builder
            .build_verification_circuit(state)
            .map_err(|e| VerificationError::StateError(e.to_string()))
    }

    async fn verify_consistency(&self, circuit: &BOC) -> VerificationResult<bool> {
        self.consistency_validator
            .verify_consistency(circuit)
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
        self.consistency_validator
            .verify_proof(circuit)
            .await
            .map_err(|e| VerificationError::ZkpError(e.to_string()))
    }

    pub fn get_verification_state(&self) -> VerificationResult<VerificationState> {
        let state = self.state_manager.get_state()?;
        let consistency_state = self.consistency_validator.get_state()?;
        let distribution_state = self.distribution_manager.get_state()?;

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
    use crate::core::zkps::plonky2::Config as PlonkyConfig;

    #[tokio::test]
    async fn test_verification_manager() {
        let state_manager = StateManager::new().expect("Failed to create state manager");
        let plonky_system = Plonky2System::default();
        let consistency_validator = ConsistencyValidator::new(Arc::new(plonky_system.clone()));
        let distribution_manager = DistributionManager::new(
            Arc::new(plonky_system),
            Default::default(),
            Default::default(),
        );
        
        let verification_manager = VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
            PlonkyConfig::default(),
        ).expect("Failed to create verification manager");

        let result = verification_manager.verify().await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
 
    #[tokio::test]
    async fn test_verification_state() {
        let state_manager = StateManager::new().expect("Failed to create state manager");
        let plonky_system = Plonky2System::default();
        let consistency_validator = ConsistencyValidator::new(Arc::new(plonky_system.clone()));
        let distribution_manager = DistributionManager::new(
            Arc::new(plonky_system),
            Default::default(),
            Default::default(),
        );
        
        let verification_manager = VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
            PlonkyConfig::default(),
        ).expect("Failed to create verification manager");
 
        let state = verification_manager.get_verification_state();
        assert!(state.is_ok());
        
        let state = state.unwrap();
        assert!(state.replication_state.is_valid());
        assert!(state.consistency_state.is_valid());
        assert!(state.distribution_state.is_valid());
    }
    
    #[tokio::test]
    async fn test_verification_error_handling() {
        let state_manager = StateManager::new().expect("Failed to create state manager");
        let plonky_system = Plonky2System::default();
        let mut consistency_validator = ConsistencyValidator::new(Arc::new(plonky_system.clone()));
        let distribution_manager = DistributionManager::new(
            Arc::new(plonky_system),
            Default::default(),
            Default::default(),
        );
 
        // Create manager with validator configured to fail
        consistency_validator.set_force_verification_error(true);
        
        let verification_manager = VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
            PlonkyConfig::default(),
        ).expect("Failed to create verification manager");
        
        let result = verification_manager.verify().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerificationError::ConsistencyError(_)));
    }
 
    #[tokio::test]
    async fn test_invalid_state() {
        let mut state_manager = StateManager::new().expect("Failed to create state manager");
        let plonky_system = Plonky2System::default();
        let consistency_validator = ConsistencyValidator::new(Arc::new(plonky_system.clone()));
        let distribution_manager = DistributionManager::new(
            Arc::new(plonky_system),
            Default::default(),
            Default::default(),
        );
 
        // Corrupt state manager
        state_manager.corrupt_state();
        
        let verification_manager = VerificationManager::new(
            state_manager,
            consistency_validator, 
            distribution_manager,
            PlonkyConfig::default(),
        ).expect("Failed to create verification manager");
 
        let result = verification_manager.verify().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerificationError::StateError(_)));
    }
 
    #[tokio::test]
    async fn test_distribution_verification() {
        let state_manager = StateManager::new().expect("Failed to create state manager");
        let plonky_system = Plonky2System::default();
        let consistency_validator = ConsistencyValidator::new(Arc::new(plonky_system.clone()));
        let mut distribution_manager = DistributionManager::new(
            Arc::new(plonky_system),
            Default::default(),
            Default::default(),
        );
 
        // Configure distribution manager to fail verification
        distribution_manager.set_force_verification_error(true);
        
        let verification_manager = VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
            PlonkyConfig::default(),
        ).expect("Failed to create verification manager");
 
        let result = verification_manager.verify().await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerificationError::DistributionError(_)));
    }
 }