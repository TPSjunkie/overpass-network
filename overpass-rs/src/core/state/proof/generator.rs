// src/core/state/proof/generator.rs

use crate::core::proof::ProofError;
use crate::core::state::proof::plonky2_state::Plonky2Backend;
use crate::core::types::ovp_types::*;
use plonky2::plonk::proof::Proof;

/// Responsible for generating proofs for state transitions.
pub struct ProofGenerator {
    backend: Plonky2Backend,
}

impl ProofGenerator {
    /// Creates a new `ProofGenerator`.
    pub fn new() -> Self {
        ProofGenerator {
            backend: Plonky2Backend::new(),
        }
    }

    /// Generates a proof for a given state and transaction.
    pub fn generate_proof(
        &self,
        state: &State,
        transaction: &Transaction,
    ) -> Result<Proof, ProofError> {
        self.backend.generate(state, transaction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_proof() {
        let generator = ProofGenerator::new();
        let state = State::default();
        let transaction = Transaction::default();

        let result = generator.generate_proof(&state, &transaction);
        assert!(result.is_ok());
    }
}
