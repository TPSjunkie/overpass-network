// src/core/state/proof/verification.rs

use crate::core::proof::ProofError;
use crate::core::state::proof::plonky2_state::Plonky2Backend;
use crate::core::types::ovp_types::{State, Transaction};
use plonky2::plonk::proof::Proof;

/// Handles the verification of wallet extention contract state proofs using the Plonky2 backend.
pub struct ProofVerifier {
    backend: Plonky2Backend,
}

impl ProofVerifier {
    /// Creates a new `ProofVerifier`.
    pub fn new() -> Self {
        ProofVerifier {
            backend: Plonky2Backend::new(),
        }
    }

    /// Verifies a proof given the current state and transaction.
    pub fn verify_proof(
        &self,
        proof: &Proof,
        state: &State,
        transaction: &Transaction,
    ) -> Result<(), ProofError> {
        self.backend.verify(proof, state, transaction)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::types::ProofType;

    use super::*;
    #[test]
    fn test_verify_proof() {
        let verifier = ProofVerifier::new();
        let state = State::default();
        let transaction = Transaction::default();
        let proof = Proof::new(ProofType::StateTransition, vec![], vec![]);

        let result = verifier.verify_proof(&proof, &state, &transaction);
        assert!(result.is_ok());
    }
}
