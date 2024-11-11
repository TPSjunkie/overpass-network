// src/core/state/proof/mod.rs

pub mod generator;
pub mod plonky2_state;
pub mod verification;

// Re-export common types
pub use self::generator::ProofGenerator;
pub use self::plonky2_state::Plonky2Backend;
pub use self::verification::ProofVerifier;
