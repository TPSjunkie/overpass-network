// ./src/core/storage_node/replication/verification.rs

/// Verification of the replication process
///
/// This module contains the logic for verifying the replication process.
/// It includes the logic for verifying the consistency of the data and the
/// logic for verifying the distribution of the data.

use crate::core::storage_node::replication::state::StateManager;
use crate::core::storage_node::replication::consistency::ConsistencyValidator;
use crate::core::storage_node::replication::distribution::DistributionManager;
use crate::core::zkps::zkp_interface::ZkpInterface;
use crate::core::storage_node::replication::state::ReplicationState;



/// The verification of the replication process
///
/// This struct represents the verification of the replication process.
/// It contains the logic for verifying the consistency of the data and the
/// logic for verifying the distribution of the data.
pub struct VerificationManager {
    state_manager: StateManager,
    consistency_validator: ConsistencyValidator,
    distribution_manager: DistributionManager,
}

impl VerificationManager {
    /// Creates a new instance of the verification manager
    ///
    /// This function creates a new instance of the verification manager.
    /// It takes a reference to the state manager, the consistency validator,
    /// and the distribution manager as arguments.
    ///
    /// # Arguments
    ///
    /// * `state_manager` - A reference to the state manager.
    /// * `consistency_validator` - A reference to the consistency validator.
    /// * `distribution_manager` - A reference to the distribution manager.
    ///
    /// # Returns
    ///
    /// A new instance of the verification manager.
    pub fn new(
        state_manager: &StateManager,
        consistency_validator: &ConsistencyValidator,
        distribution_manager: &DistributionManager,
    ) -> Self {
        Self {
            state_manager: state_manager.clone(),
            consistency_validator: consistency_validator.clone(),
            distribution_manager: distribution_manager.clone(),
        }
    }

    /// Verifies the replication process
    ///
    /// This function verifies the replication process by checking the consistency
    /// of the data and the distribution of the data.
    ///
    /// # Arguments
    ///
    /// * `zkp_interface` - A reference to the ZKP interface.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the replication process is verified or not.
    pub fn verify(&self, zkp_interface: &ZkpInterface) -> bool {
        let state = self.state_manager.get_state();
        let is_consistent = self.consistency_validator.is_consistent(&state);
        let is_distributed = self.distribution_manager.is_distributed(&state);
        if is_consistent && is_distributed {
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage_node::replication::state::ReplicationState;
    use crate::core::storage_node::replication::consistency::ConsistencyValidator;
    use crate::core::storage_node::replication::distribution::DistributionManager;
    use crate::core::zkps::zkp_interface::ZkpInterface;

    #[test]
    fn test_verify() {
        let state_manager = StateManager::new();
        let consistency_validator = ConsistencyValidator::new();
        let distribution_manager = DistributionManager::new();
        let verification_manager = VerificationManager::new(
            &state_manager,
            &consistency_validator,
            &distribution_manager,
        );
        let zkp_interface = ZkpInterface::new();
        let state = ReplicationState::new();
        state_manager.set_state(state.clone());
        consistency_validator.set_state(state.clone());
        distribution_manager.set_state(state.clone());
        assert!(verification_manager.verify(&zkp_interface));
    }
}