// src/metrics/storage.rs

use async_trait::async_trait;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Stores metrics data in memory.
#[derive(Clone)]
pub struct MetricsStorage {
    data: Arc<Mutex<HashMap<String, u64>>>,
}

impl MetricsStorage {
    /// Creates a new `MetricsStorage`.
    pub fn new() -> Self {
        MetricsStorage {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Records a metric by its name and value.
    pub fn record_metric(&self, name: &str, value: u64) {
        let mut data = self.data.lock();
        *data.entry(name.to_string()).or_insert(0) += value;
    }

    /// Fetches the metric by name, if it exists.
    pub fn get_metric(&self, name: &str) -> Option<u64> {
        let data = self.data.lock();
        data.get(name).cloned()
    }

    /// Retrieves all metrics stored.
    pub fn get_all_metrics(&self) -> HashMap<String, u64> {
        let data = self.data.lock();
        data.clone()
    }
}

#[async_trait]
pub trait MetricsStorable {
    async fn save_metrics(&self) -> Result<(), String>;
}

#[async_trait]
impl MetricsStorable for MetricsStorage {
    /// Saves metrics data. In a real application, this would persist data to a database or file.
    async fn save_metrics(&self) -> Result<(), String> {
        // Implement persistence logic here if needed.
        Ok(())
    }
}
