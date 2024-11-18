use crate::core::error::errors::SystemError;
use crate::core::storage_node::battery::charging::BatteryChargingSystem;
use crate::core::storage_node::storage_node_contract::StorageNode;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;

pub trait BatteryInterface {
    fn get_min_battery(&self) -> u64;
    fn get_peers(&self) -> Vec<[u8; 32]>;
    fn send_low_battery_alert(&self, peer_id: [u8; 32]) -> Result<(), SystemError>;
    fn send_suspension_notice(&self) -> Result<(), SystemError>;
    fn get_suspension_period(&self) -> Duration;
    fn send_resume_notice(&self) -> Result<(), SystemError>;
}

pub struct BatteryMonitor {
    battery_system: Arc<RwLock<BatteryChargingSystem>>,
    storage_node: Arc<dyn BatteryInterface>,
    monitoring_interval: Duration,
}

impl BatteryMonitor {
    pub fn new(
        battery_system: Arc<RwLock<BatteryChargingSystem>>,
        storage_node: Arc<dyn BatteryInterface>,
        monitoring_interval: Duration,
    ) -> Self {
        Self {
            battery_system,
            storage_node,
            monitoring_interval,
        }
    }

    pub fn start_monitoring(&self) {
        let _battery_system = self.battery_system.clone();
        let _storage_node = self.storage_node.clone();
        let monitoring_interval = self.monitoring_interval;
        thread::spawn(move || {
            loop {
                // Removed BatteryMetrics::new and update_metrics calls
                // as they were not defined in the provided code
                thread::sleep(monitoring_interval);
            }
        });
    }
}

impl BatteryInterface for BatteryMonitor {
    fn get_min_battery(&self) -> u64 {
        self.storage_node.get_min_battery()
    }

    fn get_peers(&self) -> Vec<[u8; 32]> {
        self.storage_node.get_peers()
    }

    fn send_low_battery_alert(&self, peer_id: [u8; 32]) -> Result<(), SystemError> {
        self.storage_node.send_low_battery_alert(peer_id)
    }

    fn send_suspension_notice(&self) -> Result<(), SystemError> {
        let peers = self.get_peers();
        for peer_id in peers {
            self.send_low_battery_alert(peer_id)?;
        }
        Ok(())
    }

    fn get_suspension_period(&self) -> Duration {
        self.storage_node.get_suspension_period()
    }

    fn send_resume_notice(&self) -> Result<(), SystemError> {
        self.storage_node.send_resume_notice()
    }
}