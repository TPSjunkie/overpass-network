// src/core/storage_node/battery/mod.rs
pub mod charging;
//pub mod monitoring;
pub mod rewards;

// re-exporting the modules
pub use charging::BatteryChargingSystem;

pub use rewards::RewardDistributor;
