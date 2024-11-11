use crate::core::{
    error::errors::SystemErrorType, storage_node::storage_node_contract::StorageNodeConfig,
};
use std::sync::atomic::{AtomicU64, Ordering};
use web_sys::window;

#[derive(Clone, Default)]
pub struct BatteryConfig {
    pub initial_charge: u64,
    pub max_charge: u64,
    pub min_capacity: u64,
    pub cooldown_period: u64,
    pub usage_rate: u64,
    pub recharge_rate: u64,
    pub max_recharge_attempts: u64,
    pub recharge_wait_time: u64,
}

pub struct BatteryChargingSystem {
    pub battery_level: AtomicU64,
    config: BatteryConfig,
    last_charge_time: AtomicU64,
    pub reward_multiplier: AtomicU64,
    pub stake_amount: AtomicU64,
}

impl BatteryChargingSystem {
    pub fn new(stake_amount: u64) -> Self {
        let now = window().unwrap().performance().unwrap().now() as u64;
        let config = BatteryConfig {
            initial_charge: 100,
            max_charge: 100,
            min_capacity: 10,
            cooldown_period: 1000,
            usage_rate: 1,
            recharge_rate: 1,
            max_recharge_attempts: 10,
            recharge_wait_time: 1000,
        };

        Self {
            battery_level: AtomicU64::new(config.max_charge),
            config,
            last_charge_time: AtomicU64::new(now),
            reward_multiplier: AtomicU64::new(100),
            stake_amount: AtomicU64::new(stake_amount),
        }
    }

    pub async fn charge_for_processing(&self) -> Result<(), SystemErrorType> {
        let current_level = self.battery_level.load(Ordering::Acquire);
        if current_level < self.config.min_capacity {
            return Err(SystemErrorType::InsufficientBalance);
        }
        let now = window().unwrap().performance().unwrap().now() as u64;
        let last_charge = self.last_charge_time.load(Ordering::Acquire);
        if now - last_charge < self.config.cooldown_period {
            return Err(SystemErrorType::SpendingLimitExceeded);
        }
        self.battery_level
            .fetch_sub(self.config.usage_rate, Ordering::Release);
        self.update_reward_multiplier();
        Ok(())
    }

    pub async fn recharge(&self, synchronized_nodes: u64) -> Result<(), SystemErrorType> {
        let now = window().unwrap().performance().unwrap().now() as u64;

        let last_charge = self.last_charge_time.load(Ordering::Acquire);
        if now - last_charge < self.config.cooldown_period {
            return Err(SystemErrorType::SpendingLimitExceeded);
        }

        let current_level = self.battery_level.load(Ordering::Acquire);

        if current_level >= self.config.max_charge {
            return Ok(());
        }

        let charge_rate = self.config.recharge_rate * synchronized_nodes;
        let new_level = current_level.saturating_add(charge_rate);
        let capped_level = new_level.min(self.config.max_charge);

        self.battery_level.store(capped_level, Ordering::Release);
        self.last_charge_time.store(now, Ordering::Release);
        self.update_reward_multiplier();
        Ok(())
    }

    fn update_reward_multiplier(&self) {
        let battery_percentage = self.get_charge_percentage();
        let multiplier = if battery_percentage >= 98.0 {
            100
        } else if battery_percentage >= 80.0 {
            battery_percentage as u64
        } else {
            0
        };
        self.reward_multiplier.store(multiplier, Ordering::Release);
    }

    pub fn get_charge_percentage(&self) -> f64 {
        let current_level = self.battery_level.load(Ordering::Relaxed);
        (current_level as f64 / self.config.max_charge as f64) * 100.0
    }

    pub async fn wait_for_sufficient_charge(&self) -> Result<(), SystemErrorType> {
        let mut attempts = 0;

        while self.battery_level.load(Ordering::Acquire) < self.config.min_capacity {
            if attempts >= self.config.max_recharge_attempts {
                return Err(SystemErrorType::SpendingLimitExceeded);
            }
            if self.battery_level.load(Ordering::Acquire) == 0 {
                return Err(SystemErrorType::InsufficientBalance);
            }
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &resolve,
                        self.config.recharge_wait_time as i32,
                    )
                    .unwrap();
            });
            wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
            attempts += 1;
        }
        Ok(())
    }

    pub fn get_reward_multiplier(&self) -> u64 {
        self.reward_multiplier.load(Ordering::Relaxed)
    }

    pub fn get_stake_amount(&self) -> u64 {
        self.stake_amount.load(Ordering::Relaxed)
    }
}
