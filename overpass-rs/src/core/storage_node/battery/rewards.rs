use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::storage_node::battery::charging::BatteryChargingSystem;
use crate::core::storage_node::epidemic::overlap::StorageOverlapManager;

// Reward tiers based on battery level and performance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardTier {
    Optimal,    // 98-100% battery, high overlap
    High,       // 80-97% battery, good overlap
    Base,       // 60-79% battery, minimal overlap
    Reduced,    // Below 60% battery
    None        // Suspended or insufficient metrics
}

// Reward multipliers for different activities
#[derive(Clone)]
pub struct RewardMultipliers {
    pub storage_multiplier: f64,      // For storing state/proofs
    pub verification_multiplier: f64,  // For verifying proofs
    pub propagation_multiplier: f64,   // For propagating messages
    pub overlap_multiplier: f64,       // Based on overlap score
    pub sync_multiplier: f64,         // Based on sync score
}

impl Default for RewardMultipliers {
    fn default() -> Self {
        Self {
            storage_multiplier: 1.0,
            verification_multiplier: 1.5,
            propagation_multiplier: 1.2,
            overlap_multiplier: 1.3,
            sync_multiplier: 1.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardMetrics {
    pub total_rewards: u64,
    pub storage_rewards: u64,
    pub verification_rewards: u64,
    pub propagation_rewards: u64,
    pub overlap_bonuses: u64,
    pub sync_bonuses: u64,
    pub current_tier: RewardTier,
    pub reward_rate: f64,
}

impl Default for RewardMetrics {
    fn default() -> Self {
        Self {
            total_rewards: 0,
            storage_rewards: 0,
            verification_rewards: 0,
            propagation_rewards: 0,
            overlap_bonuses: 0,
            sync_bonuses: 0,
            current_tier: RewardTier::Base,
            reward_rate: 1.0,
        }
    }
}

pub struct RewardDistributor {
    battery_system: Arc<BatteryChargingSystem>,
    overlap_manager: Arc<StorageOverlapManager>,
    multipliers: RewardMultipliers,
    metrics: Arc<RwLock<RewardMetrics>>,
    min_overlap_score: f64,
    min_sync_score: f64,
}

impl RewardDistributor {
    pub fn new(
        battery_system: Arc<BatteryChargingSystem>,
        overlap_manager: Arc<StorageOverlapManager>,
        multipliers: RewardMultipliers,
    ) -> Self {
        Self {
            battery_system,
            overlap_manager,
            multipliers,
            metrics: Arc::new(RwLock::new(RewardMetrics::default())),
            min_overlap_score: 0.8,  // 80% minimum overlap for bonuses
            min_sync_score: 0.7,     // 70% minimum sync for bonuses
        }
    }

    // Calculate reward tier based on battery level and performance
    pub fn calculate_reward_tier(&self) -> RewardTier {
        let battery_level = self.battery_system.get_charge_percentage();
        let overlap_score = self.overlap_manager.get_overlap_score();
        
        if self.battery_system.is_suspended() {
            return RewardTier::None;
        }

        match battery_level {
            l if l >= 98.0 && overlap_score >= self.min_overlap_score => RewardTier::Optimal,
            l if l >= 80.0 && overlap_score >= self.min_overlap_score * 0.9 => RewardTier::High,
            l if l >= 60.0 && overlap_score >= self.min_overlap_score * 0.8 => RewardTier::Base,
            l if l > 0.0 => RewardTier::Reduced,
            _ => RewardTier::None,
        }
    }

    // Calculate base reward multiplier based on tier
    fn get_tier_multiplier(tier: RewardTier) -> f64 {
        match tier {
            RewardTier::Optimal => 1.0,
            RewardTier::High => 0.8,
            RewardTier::Base => 0.6,
            RewardTier::Reduced => 0.3,
            RewardTier::None => 0.0,
        }
    }

    // Calculate rewards for storing data
    pub async fn calculate_storage_reward(
        &self,
        data_size: u64,
        duration: u64
    ) -> Result<u64, SystemError> {
        let tier = self.calculate_reward_tier();
        if tier == RewardTier::None {
            return Ok(0);
        }

        let base_reward = data_size
            .saturating_mul(duration)
            .saturating_div(1024); // Normalize by KB

        let tier_multiplier = Self::get_tier_multiplier(tier);
        let overlap_bonus = if self.overlap_manager.get_overlap_score() >= self.min_overlap_score {
            self.multipliers.overlap_multiplier
        } else {
            1.0
        };

        let final_reward = (base_reward as f64 
            * tier_multiplier 
            * self.multipliers.storage_multiplier 
            * overlap_bonus) as u64;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.storage_rewards = metrics.storage_rewards.saturating_add(final_reward);
        metrics.total_rewards = metrics.total_rewards.saturating_add(final_reward);
        metrics.current_tier = tier;

        Ok(final_reward)
    }

    // Calculate rewards for verification work
    pub async fn calculate_verification_reward(
        &self,
        proof_complexity: u64
    ) -> Result<u64, SystemError> {
        let tier = self.calculate_reward_tier();
        if tier == RewardTier::None {
            return Ok(0);
        }

        let base_reward = proof_complexity.saturating_mul(100); // Base reward per complexity unit
        let tier_multiplier = Self::get_tier_multiplier(tier);
        
        let final_reward = (base_reward as f64 
            * tier_multiplier 
            * self.multipliers.verification_multiplier) as u64;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.verification_rewards = metrics.verification_rewards.saturating_add(final_reward);
        metrics.total_rewards = metrics.total_rewards.saturating_add(final_reward);
        metrics.current_tier = tier;

        Ok(final_reward)
    }

    // Calculate rewards for message propagation
    pub async fn calculate_propagation_reward(
        &self,
        message_count: u64,
        priority_level: u8
    ) -> Result<u64, SystemError> {
        let tier = self.calculate_reward_tier();
        if tier == RewardTier::None {
            return Ok(0);
        }

        let base_reward = message_count.saturating_mul(10); // Base reward per message
        let tier_multiplier = Self::get_tier_multiplier(tier);
        let priority_multiplier = 1.0 + (priority_level as f64 / 10.0); // Higher priority = higher reward

        let sync_bonus = if self.overlap_manager.get_sync_score() >= self.min_sync_score {
            self.multipliers.sync_multiplier
        } else {
            1.0
        };

        let final_reward = (base_reward as f64 
            * tier_multiplier 
            * self.multipliers.propagation_multiplier 
            * priority_multiplier
            * sync_bonus) as u64;

        // Update metrics
        let mut metrics = self.metrics.write();
        metrics.propagation_rewards = metrics.propagation_rewards.saturating_add(final_reward);
        metrics.total_rewards = metrics.total_rewards.saturating_add(final_reward);
        metrics.current_tier = tier;

        Ok(final_reward)
    }

    // Update reward rate based on recent performance
    pub fn update_reward_rate(&self) {
        let mut metrics = self.metrics.write();
        let tier = self.calculate_reward_tier();
        let tier_multiplier = Self::get_tier_multiplier(tier);
        
        let overlap_score = self.overlap_manager.get_overlap_score();
        let sync_score = self.overlap_manager.get_sync_score();
        
        // Calculate performance factors
        let overlap_factor = if overlap_score >= self.min_overlap_score {
            overlap_score
        } else {
            overlap_score * 0.8
        };

        let sync_factor = if sync_score >= self.min_sync_score {
            sync_score
        } else {
            sync_score * 0.8
        };

        // Update rate based on all factors
        metrics.reward_rate = tier_multiplier * overlap_factor * sync_factor;
        metrics.current_tier = tier;
    }

    // Get current metrics
    pub fn get_metrics(&self) -> RewardMetrics {
        self.metrics.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    async fn setup_distributor() -> RewardDistributor {
        let battery_system = Arc::new(BatteryChargingSystem::new(Default::default()));
        let overlap_manager = Arc::new(StorageOverlapManager::new(0.8, 3));
        let multipliers = RewardMultipliers::default();
        
        RewardDistributor::new(battery_system, overlap_manager, multipliers)
    }

    #[wasm_bindgen_test]
    async fn test_reward_tiers() {
        let distributor = setup_distributor().await;
        
        // Test optimal conditions
        assert_eq!(distributor.calculate_reward_tier(), RewardTier::Optimal);
        
        // Test reduced battery
        distributor.battery_system.consume_charge(50).await.unwrap();
        assert_eq!(distributor.calculate_reward_tier(), RewardTier::Reduced);
    }

    #[wasm_bindgen_test]
    async fn test_storage_rewards() {
        let distributor = setup_distributor().await;
        
        // Test reward calculation
        let reward = distributor.calculate_storage_reward(1024, 3600).await.unwrap();
        assert!(reward > 0);
        
        // Verify metrics
        let metrics = distributor.get_metrics();
        assert_eq!(metrics.storage_rewards, reward);
    }

    #[wasm_bindgen_test]
    async fn test_verification_rewards() {
        let distributor = setup_distributor().await;
        
        // Test reward calculation
        let reward = distributor.calculate_verification_reward(10).await.unwrap();
        assert!(reward > 0);
        
        // Verify metrics
        let metrics = distributor.get_metrics();
        assert_eq!(metrics.verification_rewards, reward);
    }

    #[wasm_bindgen_test]
    async fn test_propagation_rewards() {
        let distributor = setup_distributor().await;
        
        // Test reward calculation
        let reward = distributor.calculate_propagation_reward(5, 2).await.unwrap();
        assert!(reward > 0);
        
        // Verify metrics
        let metrics = distributor.get_metrics();
        assert_eq!(metrics.propagation_rewards, reward);
    }
}