use crate::core::error::errors::SystemError;
use crate::core::storage_node::battery::charging::BatteryChargingSystem;
use crate::core::storage_node::storage_node_contract::StorageNode;
use log;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub trait BatteryInterface {
    fn get_min_battery(&self) -> u64;
    fn get_peers(&self) -> Vec<[u8; 32]>;
    fn send_low_battery_alert(&self, peer_id: [u8; 32]) -> Result<(), SystemError>;
    fn send_suspension_notice(&self) -> Result<(), SystemError>;
    fn get_suspension_period(&self) -> Duration;
    fn send_resume_notice(&self) -> Result<(), SystemError>;
}

pub struct BatteryMonitor<RootTree, IntermediateTreeManager> {
    battery_system: Arc<RwLock<BatteryChargingSystem>>,
    storage_node: Arc<StorageNode<RootTree, IntermediateTreeManager>>,
    monitoring_interval: Duration,
    _phantom: std::marker::PhantomData<IntermediateTreeManager>,
}

impl<RootTree, IntermediateTreeManager> BatteryMonitor<RootTree, IntermediateTreeManager>
where
    StorageNode<RootTree, IntermediateTreeManager>: BatteryInterface,
{
    pub fn new(
        battery_system: Arc<RwLock<BatteryChargingSystem>>,
        storage_node: Arc<StorageNode<RootTree, IntermediateTreeManager>>,
        monitoring_interval: Duration,
    ) -> Self {
        Self {
            battery_system,
            storage_node,
            monitoring_interval,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn start_monitoring(&self) {
        loop {
            if let Err(e) = self.check_battery_status().await {
                log::error!("Battery monitoring error: {:?}", e);
            }
            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &resolve,
                        self.monitoring_interval.as_millis() as i32,
                    )
                    .unwrap();
            }))
            .await
            .unwrap();
        }
    }

    async fn check_battery_status(&self) -> Result<(), SystemError> {
        let battery_system = self.battery_system.read().unwrap();
        let charge_percentage = battery_system.get_charge_percentage();
        let current_level = battery_system
            .battery_level
            .load(std::sync::atomic::Ordering::Relaxed);

        if current_level < self.storage_node.get_min_battery() {
            for peer_id in self.storage_node.get_peers() {
                self.storage_node.send_low_battery_alert(peer_id)?;
            }
        }

        if current_level == 0 {
            self.storage_node.send_suspension_notice()?;

            wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &resolve,
                        self.storage_node.get_suspension_period().as_millis() as i32,
                    )
                    .unwrap();
            }))
            .await
            .unwrap();

            let synchronized_nodes = self.storage_node.get_peers().len() as u64;
            battery_system.recharge(synchronized_nodes).await;

            if battery_system
                .battery_level
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
            {
                self.storage_node.send_resume_notice()?;
            }
        }

        log::info!(
            "Battery Status - Level: {}%, Current: {}, Reward Multiplier: {}",
            charge_percentage,
            current_level,
            battery_system.get_reward_multiplier()
        );

        Ok(())
    }
    pub fn get_monitoring_metrics(&self) -> BatteryMetrics {
        let battery_system = self.battery_system.read().unwrap();
        BatteryMetrics {
            charge_percentage: battery_system.get_charge_percentage(),
            current_level: battery_system
                .battery_level
                .load(std::sync::atomic::Ordering::Relaxed),
            reward_multiplier: battery_system.get_reward_multiplier(),
            stake_amount: battery_system.get_stake_amount(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatteryMetrics {
    pub charge_percentage: f64,
    pub current_level: u64,
    pub reward_multiplier: u64,
    pub stake_amount: u64,
    pub last_check: u64,
}
