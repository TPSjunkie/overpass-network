// ./src/core/storage_node/battery/rewards.rs

use crate::core::error::errors::SystemError;
use crate::core::storage_node::battery::charging::BatteryChargingSystem;
use std::sync::Arc;
use std::sync::RwLock;
pub struct RewardDistributor {
    battery_system: Arc<RwLock<BatteryChargingSystem>>,
}

impl RewardDistributor {
    pub fn new(battery_system: Arc<RwLock<BatteryChargingSystem>>) -> Self {
        Self { battery_system }
    }

    pub fn calculate_reward(&self, transaction_fees: u64) -> Result<u64, SystemError> {
        let battery_system = self.battery_system.read().unwrap();
        let battery_percentage = battery_system.get_charge_percentage();

        let reward = if battery_percentage >= 98.0 {
            // Maximum reward: 10% of transaction fees
            transaction_fees.saturating_mul(10) / 100
        } else if battery_percentage >= 80.0 {
            // Proportional reward based on battery percentage
            let proportion = battery_percentage / 100.0;
            let base_reward = transaction_fees.saturating_mul(10) / 100;
            (base_reward as f64 * proportion) as u64
        } else {
            // No reward for battery below 80%
            0
        };

        Ok(reward)
    }

    pub async fn distribute_rewards(&self, transaction_fees: u64) -> Result<u64, SystemError> {
        let reward = self.calculate_reward(transaction_fees)?;

        if reward > 0 {
            // Here we would implement the actual transfer logic
            // For now just return the calculated reward
            Ok(reward)
        } else {
            Ok(0)
        }
    }

    pub fn get_reward_tier(&self) -> RewardTier {
        let battery_system = self.battery_system.read().unwrap();
        let battery_percentage = battery_system.get_charge_percentage();

        if battery_percentage >= 98.0 {
            RewardTier::Maximum
        } else if battery_percentage >= 80.0 {
            RewardTier::Proportional
        } else {
            RewardTier::None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RewardTier {
    Maximum,
    Proportional,
    None,
}

impl RewardTier {
    pub fn get_multiplier(&self) -> f64 {
        match self {
            RewardTier::Maximum => 1.0,
            RewardTier::Proportional => 0.8,
            RewardTier::None => 0.0,
        }
    }
}
