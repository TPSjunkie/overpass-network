use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::types::boc::BOC;

use super::state::StateManager;

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
        }
    }

    /// Validate consistency of a state update using proofs
    pub async fn validate_consistency(
        &self,
        state_hash: [u8; 32],
        proof: &[u8],
        expected_state: &BOC,
    ) -> Result<bool, SystemError> {
        let start_time = std::time::Instant::now();

        // Verify state existence
        if !self.state_manager.verify_state(state_hash) {
            self.metrics.write().state_mismatches += 1;
            return Err(SystemError::new(
                SystemErrorType::InvalidState,
                "State hash does not exist in state manager".to_owned(),
            ));
        }

        // Generate proof inputs
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

    /// Generate a consistency report
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

    /// Get consistency metrics
    pub fn get_metrics(&self) -> ConsistencyMetrics {
        self.metrics.read().clone()
    }
}

pub trait VerifyProof {
    fn verify_proof(&self, proof: &[u8], public_inputs: &[Vec<u8>]) -> Result<bool, SystemError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn create_test_boc() -> BOC {
        BOC {
            cells: vec![],
            references: vec![],
            roots: vec![],
        }
    }

    struct MockProofManager;

    impl MockProofManager {
        fn new() -> Self {
            MockProofManager
        }

        fn generate_proof(&self, _state: &BOC, _public_inputs: &[Vec<u8>]) -> Result<Vec<u8>, SystemError> {
            Ok(vec![])
        }
    }

    impl VerifyProof for MockProofManager {
        fn verify_proof(&self, _proof: &[u8], _public_inputs: &[Vec<u8>]) -> Result<bool, SystemError> {
            Ok(true)
        }
    }

    #[wasm_bindgen_test]
    async fn test_consistency_validation() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::new(state_manager.clone(), proof_manager.clone());

        let state = create_test_boc();
        let state_hash = state_manager.update_wallet_state([1u8; 32], state.clone()).await.unwrap();
        let public_inputs = vec![state_hash.to_vec()];

        let proof = proof_manager.generate_proof(&state, &public_inputs).unwrap();

        let is_valid = validator
            .validate_consistency(state_hash, &proof, &state)
            .await
            .unwrap();
        assert!(is_valid);

        let metrics = validator.get_metrics();
        assert_eq!(metrics.total_consistency_checks, 1);
    }

    #[wasm_bindgen_test]
    async fn test_invalid_consistency() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::new(state_manager.clone(), proof_manager.clone());

        let state = create_test_boc();
        let invalid_state = create_test_boc();

        let state_hash = state_manager.update_wallet_state([2u8; 32], state.clone()).await.unwrap();
        let public_inputs = vec![state_hash.to_vec()];

        let proof = proof_manager.generate_proof(&state, &public_inputs).unwrap();

        // Validate with incorrect state
        let is_valid = validator
            .validate_consistency(state_hash, &proof, &invalid_state)
            .await
            .unwrap();
        assert!(!is_valid);

        let metrics = validator.get_metrics();
        assert_eq!(metrics.proof_failures, 1);
    }

    #[wasm_bindgen_test]
    async fn test_consistency_report() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::new(state_manager.clone(), proof_manager.clone());

        let state = create_test_boc();
        let state_hash = state_manager.update_wallet_state([3u8; 32], state.clone()).await.unwrap();
        let public_inputs = vec![state_hash.to_vec()];

        let proof = proof_manager.generate_proof(&state, &public_inputs).unwrap();
        validator
            .validate_consistency(state_hash, &proof, &state)
            .await
            .unwrap();

        let report = validator.generate_consistency_report();
        assert!(report.contains("Consistency Validation Report:"));
    }
}
