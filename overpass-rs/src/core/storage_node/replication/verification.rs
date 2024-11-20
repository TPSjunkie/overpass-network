// ./src/core/storage_node/replication/verification.rs

use std::sync::RwLock;
use sp_std::sync::RwLock;
use std::sync::Arc;

use crate::core::storage_node::replication::state::StateManager;
use crate::core::storage_node::replication::consistency::ConsistencyValidator;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::zkps::circuit_builder::ZkCircuitBuilder;
use crate::core::types::boc::BOC;

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

pub struct VerificationManager {
    state_manager: Arc<RwLock<StateManager>>,
    consistency_validator: Arc<ConsistencyValidator>,
    distribution_manager: Arc<DistributionManager>,
    circuit_builder: ZkCircuitBuilder,
}

impl VerificationManager {
    pub fn new(
        state_manager: StateManager,
        consistency_validator: ConsistencyValidator,
        distribution_manager: DistributionManager,
    ) -> Self {
        let circuit_builder = ZkCircuitBuilder::new(());  
        Self {
            state_manager: Arc::new(RwLock::new(state_manager)),
            consistency_validator: Arc::new(consistency_validator),
            distribution_manager: Arc::new(distribution_manager),
            circuit_builder,
        }
    }
    pub async fn verify_consistency(
        &self,
        circuit: &ZkCircuitBuilder,
        zkp_interface: &ZkpInterface,
    ) -> VerificationResult<bool> {
        let consistency_result = self
            .consistency_validator
            .verify_consistency(circuit, zkp_interface)
            .await
            .map_err(|e| VerificationError::ConsistencyError(e.to_string()))?;
        Ok(consistency_result)
    }


    pub async fn verify(&self, zkp_interface: &ZkpInterface) -> VerificationResult<bool> {
        // First verify the current state
        let state = self.state_manager.read().await;
        if !state.is_valid() {
            return Err(VerificationError::StateError(
                "Invalid replication state".to_string(),
            ));
        }

        // Build verification circuit
        let circuit = self.build_verification_circuit().await?;

        // Verify consistency
        let consistency_result = self
            .verify_consistency(&circuit, zkp_interface)
            .await
            .map_err(|e| VerificationError::ConsistencyError(e.to_string()))?;

        // Verify distribution
        let distribution_result = self
            .verify_distribution(&circuit, zkp_interface)
            .await
            .map_err(|e| VerificationError::DistributionError(e.to_string()))?;

        // Verify ZKP
        self.verify_zkp(&circuit, zkp_interface).await?;

        Ok(consistency_result && distribution_result)
    }

    async fn build_verification_circuit(&self) -> VerificationResult<BOC> {
        let state = self.state_manager.read().await;
        let circuit = self
            .circuit_builder
            .build_verification_circuit(&state)
            .map_err(|e| VerificationError::StateError(e.to_string()))?;
        Ok(circuit)
    }

    async fn verify_consistency(
        &self,
        circuit: &BOC,
        zkp_interface: &ZkpInterface,
    ) -> VerificationResult<bool> {
        let consistency_state = self.consistency_validator.get_state().await;
        let result = self
            .consistency_validator
            .verify_with_circuit(circuit, zkp_interface, &consistency_state)
            .await
            .map_err(|e| VerificationError::ConsistencyError(e.to_string()))?;
        Ok(result)
    }

    async fn verify_distribution(
        &self,
        circuit: &BOC,
        zkp_interface: &ZkpInterface,
    ) -> VerificationResult<bool> {
        let distribution_state = self.distribution_manager.get_state().await;
        let result = self
            .distribution_manager
            .verify_with_circuit(circuit, zkp_interface, &distribution_state)
            .await
            .map_err(|e| VerificationError::DistributionError(e.to_string()))?;
        Ok(result)
    }

    async fn verify_zkp(&self, circuit: &BOC, zkp_interface: &ZkpInterface) -> VerificationResult<()> {
        zkp_interface
            .verify_proof(circuit)
            .await
            .map_err(|e| VerificationError::ZkpError(e.to_string()))
    }

    pub async fn get_verification_state(&self) -> VerificationResult<VerificationState> {
        let state = self.state_manager.read().await;
        let consistency_state = self.consistency_validator.get_state().await;
        let distribution_state = self.distribution_manager.get_state().await;

        Ok(VerificationState {
            replication_state: state.clone(),
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
    use crate::core::zkps::plonky2::Plonky2System;

    #[tokio::test]
    async fn test_verification_manager() {
        let state_manager = StateManager::new();
        let consistency_validator = ConsistencyValidator::new(Default::default(), Default::default());
        let distribution_manager = DistributionManager::new(Default::default(), Default::default(), Default::default());
        
        let verification_manager = VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
        );

        let zkp_system = Plonky2System::new(/* Add required arguments */);
        let zkp_interface = ZkpInterface::new(zkp_system);

        let result = verification_manager.verify(&zkp_interface).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
    #[tokio::test]
    async fn test_verification_state() {
        let state_manager = StateManager::new();
        let consistency_validator = ConsistencyValidator::new(Default::default(), Default::default());
        let distribution_manager = DistributionManager::new(Default::default(), Default::default(), Default::default());
        
        let verification_manager = VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
        );

        let state = verification_manager.get_verification_state().await;
        assert!(state.is_ok());
        
        let state = state.unwrap();
        assert!(state.replication_state.is_valid());
        assert!(state.consistency_state.is_valid());
        assert!(state.distribution_state.is_valid());
    }
    #[tokio::test]
    async fn test_verification_error_handling() {
        let state_manager = StateManager::new();
        let consistency_validator = ConsistencyValidator::new(Default::default(), Default::default());
        let distribution_manager = DistributionManager::new(Default::default(), Default::default(), Default::default());
        
        let verification_manager = VerificationManager::new(
            state_manager,
            consistency_validator,
            distribution_manager,
        );

        let zkp_system = Plonky2System::new(/* Add required arguments */);
        let mut zkp_interface = ZkpInterface::new(zkp_system);
        
        // Simulate an error condition
        zkp_interface.inject_error(/* Add required arguments */);
        
        let result = verification_manager.verify(&zkp_interface).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), VerificationError::ZkpError(_)));
    }}