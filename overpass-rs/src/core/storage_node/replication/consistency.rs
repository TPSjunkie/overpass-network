use std::sync::Arc;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Serialize, Deserialize};
use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::types::boc::BOC;
use crate::core::storage_node::replication::state::StateManager;
use crate::core::zkps::circuit_builder::Circuit;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyMetrics {
    pub total_consistency_checks: u64,
    pub failed_consistency_checks: u64,
    pub state_mismatches: u64,
    pub proof_failures: u64,
    pub average_consistency_time: f64,
}

impl Default for ConsistencyMetrics {
    fn default() -> Self {
        Self {
            total_consistency_checks: 0,
            failed_consistency_checks: 0,
            state_mismatches: 0,
            proof_failures: 0,
            average_consistency_time: 0.0,
        }
    }
}

pub struct ConsistencyValidator<ProofManager> {
    state_manager: Arc<StateManager>,
    proof_manager: Arc<ProofManager>,
    metrics: RwLock<ConsistencyMetrics>,
    force_verification_error: AtomicBool,
}

impl<ProofManager> ConsistencyValidator<ProofManager>
where
    ProofManager: VerifyProof,
{
    pub fn new(state_manager: Arc<StateManager>, proof_manager: Arc<ProofManager>) -> Self {
        Self {
            state_manager,
            proof_manager,
            metrics: RwLock::new(ConsistencyMetrics::default()),
            force_verification_error: AtomicBool::new(false),
        }
    }

    pub fn set_force_verification_error(&self, value: bool) {
        self.force_verification_error.store(value, Ordering::SeqCst);
    }

    pub async fn validate_consistency(
        &self,
        state_hash: [u8; 32],
        proof: &[u8],
        expected_state: &BOC,
    ) -> Result<bool, SystemError> {
        if self.force_verification_error.load(Ordering::SeqCst) {
            return Err(SystemError::new(
                SystemErrorType::VerificationError,
                "Forced verification error".to_string()
            ));
        }

        let start_time = std::time::Instant::now();

        // Verify state existence
        if !self.state_manager.verify_state(state_hash) {
            self.metrics.write().state_mismatches += 1;
            return Err(SystemError::new(
                SystemErrorType::InvalidState,
                "State hash does not exist in state manager".to_owned(),
            ));
        }

        // Compare with expected state
        let stored_state = self.state_manager.get_state().map_err(|e| {
            SystemError::new(
                SystemErrorType::StorageError,
                format!("Failed to get state: {}", e)
            )
        })?;

        if stored_state != *expected_state {
            self.metrics.write().state_mismatches += 1;
            return Ok(false);
        }

        // Generate proof inputs and verify
        let public_inputs = vec![state_hash.to_vec()];
        let proof_result = self.proof_manager.verify_proof(proof, &public_inputs);

        match proof_result {
            Ok(is_valid) => {
                if is_valid {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let mut metrics = self.metrics.write();
                    metrics.total_consistency_checks += 1;
                    metrics.average_consistency_time = 
                        (metrics.average_consistency_time * (metrics.total_consistency_checks as f64 - 1.0) + elapsed)
                        / metrics.total_consistency_checks as f64;

                    Ok(true)
                } else {
                    self.metrics.write().proof_failures += 1;
                    Ok(false)
                }
            }
            Err(e) => {
                self.metrics.write().proof_failures += 1;
                Err(e)
            }
        }
    }

    pub async fn verify_consistency(&self, boc: &BOC) -> Result<bool, SystemError> {
        if self.force_verification_error.load(Ordering::SeqCst) {
            return Err(SystemError::new(
                SystemErrorType::VerificationError,
                "Forced verification error".to_string()
            ));
        }

        let state_hash = self.state_manager.get_hash(boc).map_err(|e| {
            SystemError::new(
                SystemErrorType::HashError,
                format!("Failed to get hash: {}", e)
            )
        })?;

        let proof = self.state_manager.generate_proof(state_hash)?;
        
        self.validate_consistency(state_hash, &proof, boc).await
    }

    pub async fn verify_proof(&self, circuit: &Circuit<Fnn{ , } D>) -> Result<(), SystemError> {
        if self.force_verification_error.load(Ordering::SeqCst) {
            return Err(SystemError::new(
                SystemErrorType::VerificationError, 
                "Forced verification error".to_string()
            ));
        }

        let start_time = std::time::Instant::now();
        let proof = self.proof_manager.generate_circuit_proof(circuit)?;
        
        let result = self.proof_manager.verify_circuit_proof(circuit, &proof)?;
        
        if result {
            let elapsed = start_time.elapsed().as_secs_f64();
            let mut metrics = self.metrics.write();
            metrics.total_consistency_checks += 1;
            metrics.average_consistency_time = 
                (metrics.average_consistency_time * (metrics.total_consistency_checks as f64 - 1.0) + elapsed)
                / metrics.total_consistency_checks as f64;
            Ok(())
        } else {
            self.metrics.write().proof_failures += 1;
            Err(SystemError::new(
                SystemErrorType::VerificationError,
                "Circuit proof verification failed".to_string()
            ))
        }
    }
    pub fn get_state(&self) -> Result<ConsistencyState, SystemError> {
        Ok(ConsistencyState {
            total_checks: self.metrics.read().total_consistency_checks,
            failed_checks: self.metrics.read().failed_consistency_checks,
            state_mismatches: self.metrics.read().state_mismatches,
            proof_failures: self.metrics.read().proof_failures,
            is_valid: self.metrics.read().failed_consistency_checks == 0
        })
    }

    pub fn generate_consistency_report(&self) -> String {
        let metrics = self.metrics.read();
        format!(
            "Consistency Validation Report:\n\
            Total Checks: {}\n\
            Failed Checks: {}\n\
            State Mismatches: {}\n\
            Proof Failures: {}\n\
            Average Validation Time: {:.2} seconds",
            metrics.total_consistency_checks,
            metrics.failed_consistency_checks,
            metrics.state_mismatches,
            metrics.proof_failures,
            metrics.average_consistency_time
        )
    }

    pub fn get_metrics(&self) -> ConsistencyMetrics {
        self.metrics.read().clone()
    }
}

pub trait VerifyProof {
    fn verify_proof(&self, proof: &[u8], public_inputs: &[Vec<u8>]) -> Result<bool, SystemError>;
    fn generate_circuit_proof(&self, circuit: &Circuit) -> Result<Vec<u8>, SystemError>;
    fn verify_circuit_proof(&self, circuit: &Circuit, proof: &[u8]) -> Result<bool, SystemError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyState {
    pub total_checks: u64,
    pub failed_checks: u64,
    pub state_mismatches: u64,
    pub proof_failures: u64,
    pub is_valid: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::zkps::circuit_builder::Circuit;
    use tokio::test;

    fn create_test_boc() -> BOC {
        BOC {
            cells: vec![],
            references: vec![],
            roots: vec![],
            hash: todo!(),
        }
    }

    struct MockProofManager;

    impl MockProofManager {
        fn new() -> Self {
            MockProofManager
        }
    }

    impl VerifyProof for MockProofManager {
        fn verify_proof(&self, _proof: &[u8], _public_inputs: &[Vec<u8>]) -> Result<bool, SystemError> {
            Ok(true)
        }

        fn generate_circuit_proof(&self, _circuit: &Circuit) -> Result<Vec<u8>, SystemError> {
            Ok(vec![])
        }

        fn verify_circuit_proof(&self, _circuit: &Circuit, _proof: &[u8]) -> Result<bool, SystemError> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_consistency_validation() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::new(state_manager.clone(), proof_manager);

        let state = create_test_boc();
        let state_hash = state_manager.update_wallet_state([1u8; 32], state.clone(), vec![]).unwrap();
        let public_inputs = vec![state_hash.to_vec()];
        
        let is_valid = validator
            .validate_consistency(state_hash, &[], &state)
            .await
            .unwrap();
        assert!(is_valid);

        let metrics = validator.get_metrics();
        assert_eq!(metrics.total_consistency_checks, 1);
    }

    #[tokio::test]
    async fn test_invalid_consistency() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::new(state_manager.clone(), proof_manager);

        validator.set_force_verification_error(true);

        let state = create_test_boc();
        let result = validator.verify_consistency(&state).await;
        assert!(result.is_err());

        let metrics = validator.get_metrics();
        assert_eq!(metrics.failed_consistency_checks, 0);
    }

    #[tokio::test]
    async fn test_consistency_report() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::new(state_manager, proof_manager);

        let report = validator.generate_consistency_report();
        assert!(report.contains("Consistency Validation Report:"));
        assert!(report.contains("Total Checks: 0"));
    }

    #[tokio::test]
    async fn test_verification_state() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::new(state_manager, proof_manager);

        let state = validator.get_state().unwrap();
        assert!(state.is_valid);
        assert_eq!(state.total_checks, 0);
    }
}