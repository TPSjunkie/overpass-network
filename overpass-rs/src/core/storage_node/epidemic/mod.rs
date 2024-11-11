// ./src/core/storage_node/epidemic/mod.rs

pub mod overlap;
pub mod propagation;
//pub mod sync;

// Re-export for convenience
pub use overlap::StorageOverlapManager;
pub use propagation::BatteryPropagation;
pub use sync::SynchronizationManager;
