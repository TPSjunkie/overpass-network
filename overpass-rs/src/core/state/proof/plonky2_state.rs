use merkle::Proof;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::PoseidonGoldilocksConfig;
use plonky2_field::goldilocks_field::GoldilocksField;

use crate::core::circuit_builder::Circuit;
use crate::core::proof::ProofError;
use crate::core::types::ovp_types::{State, Transaction};

type F = GoldilocksField;
type C = PoseidonGoldilocksConfig;
type H = PoseidonHash;
/// Backend that interfaces with the Plonky2 library for proof generation and verification.
pub struct Plonky2Backend;

impl Plonky2Backend {
    pub fn new() -> Self {
        Plonky2Backend
    }

    pub fn generate(&self, state: &State, transaction: &Transaction) -> Result<Proof, ProofError> {
        let circuit = self.build_circuit(state, transaction)?;
        let witness = self.create_witness(state, transaction)?;
        let plonky2_proof = circuit
            .prove(&witness)
            .map_err(|e| ProofError::GenerationError(e.to_string()))?;
        Ok(Proof::from_plonky2_proof(plonky2_proof))
    }

    pub fn verify(
        &self,
        proof: &Proof,
        state: &State,
        transaction: &Transaction,
    ) -> Result<(), ProofError> {
        let circuit = self.build_circuit(state, transaction)?;
        let plonky2_proof = proof.to_plonky2_proof();
        circuit
            .verify(&plonky2_proof)
            .map_err(|e| ProofError::VerificationError(e.to_string()))
    }

    fn build_circuit(
        &self,
        state: &State,
        transaction: &Transaction,
    ) -> Result<Circuit<F, C, H>, ProofError> {
        let config = CircuitConfig::standard_recursion_config();
        let mut builder = CircuitBuilder::<F, C>::new(config);

        let state_root = builder.add_virtual_target();
        let state_balance = builder.add_virtual_target();
        let state_nonce = builder.add_virtual_target();

        let transaction_signature = builder.add_virtual_target();
        let transaction_amount = builder.add_virtual_target();
        let transaction_nonce = builder.add_virtual_target();

        builder.connect(state_root, F::from_canonical_u64(state.root()));
        builder.connect(state_balance, F::from_canonical_u64(state.balance()));
        builder.connect(state_nonce, F::from_canonical_u64(state.nonce()));

        builder.connect(
            transaction_signature,
            F::from_canonical_u64(transaction.signature()),
        );
        builder.connect(
            transaction_amount,
            F::from_canonical_u64(transaction.amount()),
        );
        builder.connect(
            transaction_nonce,
            F::from_canonical_u64(transaction.nonce()),
        );

        builder.add_constraint(
            "balance_sufficient",
            state_balance - transaction_amount,
            F::ZERO,
            F::from_canonical_u64(state.balance()),
        );

        builder.add_constraint(
            "nonce_valid",
            state_nonce - transaction_nonce,
            F::ZERO,
            F::ZERO,
        );

        // Simplified signature validation (replace with actual logic)
        builder.add_constraint(
            "signature_valid",
            transaction_signature,
            F::ZERO,
            F::from_canonical_u64(transaction.signature()),
        );

        let new_balance = builder.sub(state_balance, transaction_amount);
        let new_nonce = builder.add(state_nonce, F::ONE);

        builder.add_constraint(
            "new_balance",
            new_balance,
            F::ZERO,
            F::from_canonical_u64(state.balance() - transaction.amount()),
        );

        builder.add_constraint(
            "new_nonce",
            new_nonce,
            F::ZERO,
            F::from_canonical_u64(state.nonce() + 1),
        );

        let circuit = builder.build::<H>();

        Ok(circuit)
    }
    fn create_witness(
        &self,
        state: &State,
        transaction: &Transaction,
    ) -> Result<Vec<F>, ProofError> {
        let mut witness = Vec::new();

        witness.push(F::from_canonical_u64(state.root()));
        witness.push(F::from_canonical_u64(state.balance()));
        witness.push(F::from_canonical_u64(state.nonce()));

        witness.push(F::from_canonical_u64(transaction.signature()));
        witness.push(F::from_canonical_u64(transaction.amount()));
        witness.push(F::from_canonical_u64(transaction.nonce()));

        let new_balance = state.balance() - transaction.amount();
        let new_nonce = state.nonce() + 1;
        witness.push(F::from_canonical_u64(new_balance));
        witness.push(F::from_canonical_u64(new_nonce));

        Ok(witness)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_proof() {
        let backend = Plonky2Backend::new();
        let state = State::default();
        let transaction = Transaction::default();

        let proof = backend.generate(&state, &transaction).unwrap();
        let result = backend.verify(&proof, &state, &transaction);
        assert!(result.is_ok());
    }
}
