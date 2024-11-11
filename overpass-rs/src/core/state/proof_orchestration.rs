// src/core/state/proof_orchestration.rs

use crate::core::proof::ProofError;
use crate::core::state::proof::generator::ProofGenerator;
use crate::core::state::proof::verification::ProofVerifier;
use crate::core::types::ovp_types::*;
use merkle::Proof;

/// Manages the orchestration of proof generation and verification.
pub struct ProofOrchestration {
    generator: ProofGenerator,
    verifier: ProofVerifier,
}

impl ProofOrchestration {
    /// Creates a new `ProofOrchestration`.
    pub fn new() -> Self {
        Self {
            generator: ProofGenerator::new(),
            verifier: ProofVerifier::new(),
        }
    }

    /// Generates a proof for a given state and transaction.
    pub fn generate_proof(
        &self,
        state: &State,
        transaction: &Transaction,
    ) -> Result<Proof, ProofError> {
        self.generator.generate_proof(state, transaction)
    }

    /// Verifies a proof against the given state and transaction.
    pub fn verify_proof(
        &self,
        proof: &Proof,
        state: &State,
        transaction: &Transaction,
    ) -> Result<(), ProofError> {
        self.verifier.verify_proof(proof, state, transaction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_orchestration() {
        let orchestration = ProofOrchestration::new();
        let state = State::default();
        let transaction = Transaction::default();

        let proof = orchestration.generate_proof(&state, &transaction).unwrap();
        let result = orchestration.verify_proof(&proof, &state, &transaction);
        assert!(result.is_ok());
    }
}
