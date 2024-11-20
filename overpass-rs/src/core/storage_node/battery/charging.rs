use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use web_sys::window;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::core::error::errors::SystemErrorType;

#[derive(Clone)]
pub struct BatteryConfig {
    // Core battery parameters
    pub max_charge: u64,            // 100%
    pub optimal_threshold: u64,     // 98% - for maximum rewards
    pub high_threshold: u64,        // 80% - for partial rewards
    pub min_capacity: u64,          // Minimum required for operation
    pub suspension_threshold: u64,   // 0% - node gets suspended
    
    // Charging mechanics
    pub base_charging_rate: u64,    // Base rate for charge increase
    pub overlap_multiplier: u64,    // Multiplier based on overlap score
    pub discharge_rate: u64,        // Rate at which battery depletes
    pub sync_boost_factor: u64,     // Additional charge from synchronization
    
    // Timing parameters
    pub charging_cooldown: u64,     // Minimum time between charges
    pub suspension_period: u64,     // How long node stays suspended
}

impl Default for BatteryConfig {
    fn default() -> Self {
        Self {
            max_charge: 100,
            optimal_threshold: 98,
            high_threshold: 80,
            min_capacity: 10,
            suspension_threshold: 0,
            base_charging_rate: 1,
            overlap_multiplier: 2,
            discharge_rate: 1,
            sync_boost_factor: 2,
            charging_cooldown: 1000, // 1 second
            suspension_period: 3600000, // 1 hour
        }
    }
}

pub struct BatteryChargingSystem {
    // Core state
    battery_level: AtomicU64,
    config: BatteryConfig,
    
    // Timing tracking
    last_charge_time: AtomicU64,
    last_suspension_time: Option<AtomicU64>,
    
    // Performance metrics
    overlap_score: AtomicU64,
    sync_score: AtomicU64,
    
    // Current status
    is_suspended: AtomicBool,
}

impl BatteryChargingSystem {
    pub fn new(config: BatteryConfig) -> Self {
        let now = window().unwrap().performance().unwrap().now() as u64;
        
        Self {
            battery_level: AtomicU64::new(config.max_charge),
            config,
            last_charge_time: AtomicU64::new(now),
            last_suspension_time: None,
            overlap_score: AtomicU64::new(0),
            sync_score: AtomicU64::new(0),
            is_suspended: AtomicBool::new(false),
        }
    }

    // Core charging functionality
    pub async fn charge(&self) -> Result<(), SystemErrorType> {
        if self.is_suspended.load(Ordering::Acquire) {
            return Err(SystemErrorType::NodeSuspended);
        }

        let now = window().unwrap().performance().unwrap().now() as u64;
        let last_charge = self.last_charge_time.load(Ordering::Acquire);
        
        // Check cooldown period
        if now - last_charge < self.config.charging_cooldown {
            return Err(SystemErrorType::CooldownPeriod);
        }

        let current_level = self.battery_level.load(Ordering::Acquire);
        if current_level >= self.config.max_charge {
            return Ok(());
        }

        // Calculate charge amount based on overlap and sync scores
        let overlap_bonus = self.overlap_score.load(Ordering::Relaxed) * self.config.overlap_multiplier;
        let sync_bonus = self.sync_score.load(Ordering::Relaxed) * self.config.sync_boost_factor;
        let total_charge = self.config.base_charging_rate + overlap_bonus + sync_bonus;

        // Apply charge
        let new_level = current_level.saturating_add(total_charge);
        let capped_level = new_level.min(self.config.max_charge);
        
        self.battery_level.store(capped_level, Ordering::Release);
        self.last_charge_time.store(now, Ordering::Release);
        
        Ok(())
    }

    // Battery consumption for operations
    pub async fn consume_charge(&mut self, amount: u64) -> Result<(), SystemErrorType> {
        if self.is_suspended.load(Ordering::Acquire) {
            return Err(SystemErrorType::NodeSuspended);
        }

        let current_level = self.battery_level.load(Ordering::Acquire);
        if current_level < self.config.min_capacity {
            return Err(SystemErrorType::InsufficientCharge);
        }

        let new_level = current_level.saturating_sub(amount);
        self.battery_level.store(new_level, Ordering::Release);

        // Check if we need to suspend
        if new_level <= self.config.suspension_threshold {
            self.suspend().await?;
        }

        Ok(())
    }

    // Suspension handling
    async fn suspend(&mut self) -> Result<(), SystemErrorType> {
        self.is_suspended.store(true, Ordering::Release);
        let now = window().unwrap().performance().unwrap().now() as u64;
        self.last_suspension_time = Some(AtomicU64::new(now));
        Ok(())
    }


    pub async fn check_suspension(&self) -> bool {
        if let Some(last_suspension) = &self.last_suspension_time {
            let now = window().unwrap().performance().unwrap().now() as u64;
            if now - last_suspension.load(Ordering::Acquire) >= self.config.suspension_period {
                self.is_suspended.store(false, Ordering::Release);
                return false;
            }
            return true;
        }
        false
    }

    // Metrics updates
    pub fn update_overlap_score(&self, score: u64) {
        self.overlap_score.store(score, Ordering::Release);
    }

    pub fn update_sync_score(&self, score: u64) {
        self.sync_score.store(score, Ordering::Release);
    }

    // Status checks
    pub fn get_charge_percentage(&self) -> f64 {
        let current_level = self.battery_level.load(Ordering::Relaxed);
        (current_level as f64 / self.config.max_charge as f64) * 100.0
    }

    pub fn is_optimal(&self) -> bool {
        self.get_charge_percentage() >= self.config.optimal_threshold as f64
    }

    pub fn is_high(&self) -> bool {
        let percentage = self.get_charge_percentage();
        percentage >= self.config.high_threshold as f64 && percentage < self.config.optimal_threshold as f64
    }

    pub fn is_suspended(&self) -> bool {
        self.is_suspended.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libc::system;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_charging_system() {
        let config = BatteryConfig::default();
        let mut system = BatteryChargingSystem::new(config);
        
        // Test initial state
        assert_eq!(system.get_charge_percentage(), 100.0);
        assert!(system.is_optimal());
        assert!(!system.is_suspended());

        // Test consumption
        system.consume_charge(20).await.unwrap();
        assert_eq!(system.get_charge_percentage(), 80.0);
        assert!(system.is_high());

        // Test charging
        system.update_overlap_score(10);
        system.update_sync_score(5);
        system.charge().await.unwrap();
        
        // Verify charge increased
        assert!(system.get_charge_percentage() > 80.0);
    }
    #[wasm_bindgen_test]
    async fn test_suspension() {
        let mut config = BatteryConfig::default();
        config.suspension_threshold = 5;
        let mut system = BatteryChargingSystem::new(config);

        // Consume until suspension
        system.consume_charge(95).await.unwrap();
        assert!(system.is_suspended());
        
        // Verify suspended operations fail
        assert!(system.charge().await.is_err());
        assert!(system.consume_charge(1).await.is_err());
    }
}