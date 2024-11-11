// ./src/metrics/reporting.rs

use crate::metrics::collection::MetricsCollection;
use std::time::Duration;

pub struct MetricsReporter {
    collection: MetricsCollection,
}

impl MetricsReporter {
    pub fn new(collection: MetricsCollection) -> Self {
        MetricsReporter { collection }
    }

    /// Starts the reporting task, which periodically logs or sends metrics data.
    #[cfg(target_arch = "wasm32")]
    pub async fn start_reporting(&self) {
        let collection = self.collection.clone();
        spawn_local(async move {
            let delay = Duration::from_secs(10);
            loop {
                gloo_timers::future::sleep(delay).await;
                if let Err(e) = collection.report_metrics().await {
                    web_sys::console::error_1(&e.into());
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_reporting(&self) {
        let collection = self.collection.clone();
        std::thread::spawn(move || {
            let delay = Duration::from_secs(10);
            loop {
                std::thread::sleep(delay);
                if let Err(e) = collection.report_metrics() {
                    eprintln!("Error reporting metrics: {:?}", e);
                }
            }
        });
    }
}
