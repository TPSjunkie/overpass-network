use std::sync::Arc;
use parking_lot::RwLock;
use web_sys::{console, window};
use wasm_bindgen_futures::spawn_local;
use std::time::Duration;
use serde::{Serialize, Deserialize};

use crate::core::error::errors::SystemError;
use crate::core::storage_node::battery::charging::BatteryChargingSystem;

// Monitoring configuration
#[derive(Clone, Debug)]
pub struct MonitoringConfig {
    pub check_interval: Duration,          // How often to check battery status
    pub alert_threshold: u64,              // Battery level to start alerting
    pub critical_threshold: u64,           // Battery level for critical alerts
    pub min_peers_for_charging: u64,       // Minimum peers needed to charge
    pub max_concurrent_operations: u64,    // Maximum operations while in low battery
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(1),
            alert_threshold: 20,           // Alert at 20% battery
            critical_threshold: 10,        // Critical at 10% battery
            min_peers_for_charging: 3,     // Need at least 3 peers to charge
            max_concurrent_operations: 5,   // Max 5 ops when battery is low
        }
    }
}

// Metrics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryMetrics {
    pub current_level: f64,
    pub charge_rate: f64,
    pub discharge_rate: f64,
    pub time_since_charge: u64,
    pub overlap_score: u64,
    pub sync_score: u64,
    pub connected_peers: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub suspension_count: u64,
    pub last_suspension_duration: Option<u64>,
}

impl Default for BatteryMetrics {
    fn default() -> Self {
        Self {
            current_level: 100.0,
            charge_rate: 0.0,
            discharge_rate: 0.0,
            time_since_charge: 0,
            overlap_score: 0,
            sync_score: 0,
            connected_peers: 0,
            successful_operations: 0,
            failed_operations: 0,
            suspension_count: 0,
            last_suspension_duration: None,
        }
    }
}

pub struct BatteryMonitor {
    battery_system: Arc<BatteryChargingSystem>,
    config: MonitoringConfig,
    metrics: Arc<RwLock<BatteryMetrics>>,
    operation_semaphore: Arc<RwLock<u64>>,
    is_monitoring: Arc<RwLock<bool>>,
}

impl BatteryMonitor {
    pub fn new(
        battery_system: Arc<BatteryChargingSystem>,
        config: MonitoringConfig,
    ) -> Self {
        Self {
            battery_system,
            config,
            metrics: Arc::new(RwLock::new(BatteryMetrics::default())),
            operation_semaphore: Arc::new(RwLock::new(0)),
            is_monitoring: Arc::new(RwLock::new(false)),
        }
    }

    // Start the monitoring process
    pub async fn start_monitoring(&self) -> Result<(), SystemError> {
        let mut is_monitoring = self.is_monitoring.write();
        if *is_monitoring {
            return Ok(());
        }
        *is_monitoring = true;
        drop(is_monitoring);

        let monitor = self.clone();
        spawn_local(async move {
            while *monitor.is_monitoring.read() {
                monitor.check_battery_status().await;
                monitor.update_metrics().await;
                monitor.handle_low_battery().await;
                
                // Wait for next check interval
                wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, _| {
                    window().unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            monitor.config.check_interval.as_millis() as i32,
                        )
                        .unwrap();
                }))
                .await
                .unwrap();
            }
        });

        Ok(())
    }

    // Stop monitoring
    pub fn stop_monitoring(&self) {
        *self.is_monitoring.write() = false;
    }

    // Check battery status and handle state changes
    async fn check_battery_status(&self) {
        let current_level = self.battery_system.get_charge_percentage();
        let metrics = self.metrics.read();

        // Check if we need to trigger alerts
        if current_level <= self.config.alert_threshold as f64 {
            self.handle_low_battery_alert(current_level).await;
        }

        // Check if we need to limit operations
        if current_level <= self.config.critical_threshold as f64 {
            self.limit_operations().await;
        }

        // Check if we can charge
        if metrics.connected_peers >= self.config.min_peers_for_charging {
            if let Err(e) = self.battery_system.charge().await {
                console::error_1(&format!("Charging error: {:?}", e).into());
            }
        }
    }

    // Handle low battery conditions
    async fn handle_low_battery_alert(&self, level: f64) {
        console::warn_1(&format!("Low battery alert: {}%", level).into());
        
        // Update metrics
        let mut metrics = self.metrics.write();
        if level <= self.config.critical_threshold as f64 {
            metrics.failed_operations += 1;
        }
    }

    // Limit operations during low battery
    async fn limit_operations(&self) {
        let current_ops = *self.operation_semaphore.read();
        if current_ops >= self.config.max_concurrent_operations {
            console::warn_1(&"Operations limited due to low battery".into());
        }
    }

    // Update monitoring metrics
    async fn update_metrics(&self) {
        let mut metrics = self.metrics.write();
        
        metrics.current_level = self.battery_system.get_charge_percentage();
        metrics.time_since_charge = self.get_time_since_last_charge().await;
        
        // Update suspension metrics if necessary
        if self.battery_system.is_suspended() {
            metrics.suspension_count += 1;
            metrics.last_suspension_duration = Some(self.get_current_suspension_duration().await);
        }
    }

    // Utility functions
    async fn get_time_since_last_charge(&self) -> u64 {
        let now = window().unwrap().performance().unwrap().now() as u64;
        now - self.battery_system.last_charge_time()
    }

    async fn get_current_suspension_duration(&self) -> u64 {
        let now = window().unwrap().performance().unwrap().now() as u64;
        match self.battery_system.last_suspension_time() {
            Some(time) => now - time,
            None => 0,
        }
    }

    // Public interface for metrics
    pub fn get_metrics(&self) -> BatteryMetrics {
        self.metrics.read().clone()
    }

    // Operation management
    pub async fn request_operation(&self) -> Result<(), SystemError> {
        let mut ops = self.operation_semaphore.write();
        let current_level = self.battery_system.get_charge_percentage();

        if current_level <= self.config.critical_threshold as f64 
            && *ops >= self.config.max_concurrent_operations {
            return Err(SystemError::battery_error("Too many operations for current battery level"));
        }

        *ops += 1;
        Ok(())
    }

    pub async fn complete_operation(&self, success: bool) {
        let mut ops = self.operation_semaphore.write();
        *ops = ops.saturating_sub(1);

        let mut metrics = self.metrics.write();
        if success {
            metrics.successful_operations += 1;
        } else {
            metrics.failed_operations += 1;
        }
    }
}

impl Clone for BatteryMonitor {
    fn clone(&self) -> Self {
        Self {
            battery_system: Arc::clone(&self.battery_system),
            config: self.config.clone(),
            metrics: Arc::clone(&self.metrics),
            operation_semaphore: Arc::clone(&self.operation_semaphore),
            is_monitoring: Arc::clone(&self.is_monitoring),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);

    async fn setup_monitor() -> BatteryMonitor {
        let battery_system = Arc::new(BatteryChargingSystem::new(Default::default()));
        let config = MonitoringConfig::default();
        BatteryMonitor::new(battery_system, config)
    }

    #[wasm_bindgen_test]
    async fn test_monitor_start_stop() {
        let monitor = setup_monitor().await;
        
        // Test starting
        monitor.start_monitoring().await.unwrap();
        assert!(*monitor.is_monitoring.read());
        
        // Test stopping
        monitor.stop_monitoring();
        assert!(!*monitor.is_monitoring.read());
    }

    #[wasm_bindgen_test]
    async fn test_operation_management() {
        let monitor = setup_monitor().await;
        
        // Test successful operation
        monitor.request_operation().await.unwrap();
        monitor.complete_operation(true).await;
        
        let metrics = monitor.get_metrics();
        assert_eq!(metrics.successful_operations, 1);
        assert_eq!(metrics.failed_operations, 0);
    }

    #[wasm_bindgen_test]
    async fn test_low_battery_behavior() {
        let monitor = setup_monitor().await;
        
        // Drain battery
        monitor.battery_system.consume_charge(90).await.unwrap();
        
        // Test operation limiting
        for _ in 0..monitor.config.max_concurrent_operations {
            monitor.request_operation().await.unwrap();
        }
        
        // Next operation should fail
        assert!(monitor.request_operation().await.is_err());
    }
}