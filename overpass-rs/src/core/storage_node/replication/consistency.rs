use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::replication::state::StateManager;
use crate::core::types::boc::BOC;
use crate::core::zkps::circuit_builder::Circuit;
use parking_lot::RwLock;
use plonky2::hash::hash_types::RichField;
use plonky2_field::extension::Extendable;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::instrument::Instrument;

const D: usize = 2; // Extension degree

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

pub struct ConsistencyValidator<F: RichField + Extendable<D>, ProofManager> {
    state_manager: Arc<StateManager>,
    proof_manager: Arc<ProofManager>,
    metrics: RwLock<ConsistencyMetrics>,
    force_verification_error: AtomicBool,
    _phantom: std::marker::PhantomData<F>,
}

impl<F: RichField + Extendable<D>, ProofManager> ConsistencyValidator<F, ProofManager>
where
    ProofManager: VerifyProof<F>,
{
    pub fn new(state_manager: Arc<StateManager>, proof_manager: Arc<ProofManager>) -> Self {
        Self {
            state_manager,
            proof_manager,
            metrics: RwLock::new(ConsistencyMetrics::default()),
            force_verification_error: AtomicBool::new(false),
            _phantom: std::marker::PhantomData,
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
                "Forced verification error".to_string(),
            ));
        }

        let start_time = std::time::Instant::now();

        if !self.state_manager.verify_state(state_hash) {
            self.metrics.write().state_mismatches += 1;
            return Err(SystemError::new(
                SystemErrorType::InvalidState,
                "State hash does not exist in state manager".to_owned(),
            ));
        }

        let stored_state = self.state_manager.in_current_span().await.map_err(|e| {
            SystemError::new(
                SystemErrorType::StorageError,
                format!("Failed to get state: {}", e),
            )
        })?;

        if stored_state != *expected_state {
            self.metrics.write().state_mismatches += 1;
            return Ok(false);
        }

        let proof_result = self
            .proof_manager
            .verify_proof(proof, &[state_hash.to_vec()]);

        match proof_result {
            Ok(is_valid) => {
                if is_valid {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let mut metrics = self.metrics.write();
                    metrics.total_consistency_checks += 1;
                    metrics.average_consistency_time = (metrics.average_consistency_time
                        * (metrics.total_consistency_checks as f64 - 1.0)
                        + elapsed)
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
                "Forced verification error".to_string(),
            ));
        }

        let state_hash = self.state_manager.get_state_type(boc).ok_or_else(|| {
            SystemError::new(
                SystemErrorType::InvalidState,
                "Failed to get state hash".to_string(),
            )
        })?;

        let proof = self.state_manager.generate_proof(&state_hash)?;

        self.validate_consistency(state_hash, &proof, boc).await
    }
    pub async fn verify_proof(&self, circuit: &Circuit<F, D>) -> Result<(), SystemError> {
        if self.force_verification_error.load(Ordering::SeqCst) {
            return Err(SystemError::new(
                SystemErrorType::VerificationError,
                "Forced verification error".to_string(),
            ));
        }

        let start_time = std::time::Instant::now();
        let proof = self.proof_manager.generate_circuit_proof(circuit)?;

        let result = self.proof_manager.verify_circuit_proof(circuit, &proof)?;

        if result {
            let elapsed = start_time.elapsed().as_secs_f64();
            let mut metrics = self.metrics.write();
            metrics.total_consistency_checks += 1;
            metrics.average_consistency_time = (metrics.average_consistency_time
                * (metrics.total_consistency_checks as f64 - 1.0)
                + elapsed)
                / metrics.total_consistency_checks as f64;
            Ok(())
        } else {
            self.metrics.write().proof_failures += 1;
            Err(SystemError::new(
                SystemErrorType::VerificationError,
                "Circuit proof verification failed".to_string(),
            ))
        }
    }
    pub fn get_state(&self) -> Result<ConsistencyState, SystemError> {
        Ok(ConsistencyState {
            total_checks: self.metrics.read().total_consistency_checks,
            failed_checks: self.metrics.read().failed_consistency_checks,
            state_mismatches: self.metrics.read().state_mismatches,
            proof_failures: self.metrics.read().proof_failures,
            is_valid: self.metrics.read().failed_consistency_checks == 0,
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

pub trait VerifyProof<F: RichField + Extendable<D>> {
    fn verify_proof(&self, proof: &[u8], public_inputs: &[Vec<u8>]) -> Result<bool, SystemError>;
    fn generate_circuit_proof(&self, circuit: &Circuit<F, D>) -> Result<Vec<u8>, SystemError>;
    fn verify_circuit_proof(
        &self,
        circuit: &Circuit<F, D>,
        proof: &[u8],
    ) -> Result<bool, SystemError>;
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
    use plonky2::field::goldilocks_field::GoldilocksField;
    use wasm_bindgen_test::*;

    type F = GoldilocksField;

    fn create_test_boc() -> BOC {
        BOC::new()
    }

    struct MockProofManager;

    impl MockProofManager {
        fn new() -> Self {
            MockProofManager
        }
    }

    impl VerifyProof<F> for MockProofManager {
        fn verify_proof(
            &self,
            _proof: &[u8],
            _public_inputs: &[Vec<u8>],
        ) -> Result<bool, SystemError> {
            Ok(true)
        }

        fn generate_circuit_proof(&self, _circuit: &Circuit<F, D>) -> Result<Vec<u8>, SystemError> {
            Ok(vec![])
        }

        fn verify_circuit_proof(
            &self,
            _circuit: &Circuit<F, D>,
            _proof: &[u8],
        ) -> Result<bool, SystemError> {
            Ok(true)
        }
    }

    #[wasm_bindgen_test]
    async fn test_consistency_validation() {
        let state_manager = Arc::new(StateManager::default());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::<F, _>::new(state_manager.clone(), proof_manager);

        let state = create_test_boc();
        let state_hash = state_manager
            .update_wallet_state([1u8; 32], state.clone(), vec![])
            .unwrap();

        let is_valid = validator
            .validate_consistency(state_hash, &[], &state)
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
        let validator = ConsistencyValidator::<F, _>::new(state_manager.clone(), proof_manager);

        validator.set_force_verification_error(true);

        let state = create_test_boc();
        let result = validator.verify_consistency(&state).await;
        assert!(result.is_err());

        let metrics = validator.get_metrics();
        assert_eq!(metrics.failed_consistency_checks, 0);
    }

    #[wasm_bindgen_test]
    async fn test_consistency_report() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::<F, _>::new(state_manager, proof_manager);

        let report = validator.generate_consistency_report();
        assert!(report.contains("Consistency Validation Report:"));
        assert!(report.contains("Total Checks: 0"));
    }

    #[wasm_bindgen_test]
    async fn test_verification_state() {
        let state_manager = Arc::new(StateManager::new().unwrap());
        let proof_manager = Arc::new(MockProofManager::new());
        let validator = ConsistencyValidator::<F, _>::new(state_manager, proof_manager);

        let state = validator.get_state().unwrap();
        assert!(state.is_valid);
        assert_eq!(state.total_checks, 0);
    }
}
