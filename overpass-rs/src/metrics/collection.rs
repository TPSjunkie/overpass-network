// src/metrics/collection.rs

use crate::metrics::storage::{MetricsStorable, MetricsStorage};
use std::sync::Arc;

#[derive(Clone)]
pub struct MetricsCollection {
    storage: Arc<MetricsStorage>,
}

impl MetricsCollection {
    /// Creates a new `MetricsCollection`.
    pub fn new(storage: Arc<MetricsStorage>) -> Self {
        MetricsCollection { storage }
    }

    /// Collects and records a metric.
    pub fn collect(&self, name: &str, value: u64) {
        self.storage.record_metric(name, value);
    }

    /// Aggregates and reports all metrics.
    pub async fn report_metrics(&self) -> Result<(), String> {
        let metrics = self.storage.get_all_metrics();
        for (name, value) in metrics {
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(&format!("Metric: {} - Value: {}", name, value).into());

            #[cfg(not(target_arch = "wasm32"))]
            println!("Metric: {} - Value: {}", name, value);
        }
        self.storage.save_metrics().await
    }
}
