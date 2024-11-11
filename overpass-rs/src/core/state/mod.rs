// ./src/core/state/mod.rs

// src/core/state/mod.rs
pub mod boc;
pub mod consistency_checker;
pub mod proof;
pub mod proof_orchestration;
pub mod state_machine;
pub mod state_sync;

// Re-exporting the modules
pub use consistency_checker::ConsistencyChecker;
pub use proof_orchestration::ProofOrchestration;
pub use state_sync::StateSync;

pub use state_machine::StateMachine;
