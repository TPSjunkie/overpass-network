// ./src/core/storage_node/replication/distribution.rs

use crate::core::error::SystemError;
use crate::core::storage_node::storage_node_contract::StorageNode;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::console;

#[wasm_bindgen]
pub struct DistributionManager {
    storage_node: JsValue,
    replication_threshold: u64,
    replication_interval: u32,
}

#[wasm_bindgen]
impl DistributionManager {
    #[wasm_bindgen(constructor)]
    pub fn new(
        storage_node: JsValue,
        replication_threshold: u64,
        replication_interval: u32,
    ) -> Self {
        Self {
            storage_node,
            replication_threshold,
            replication_interval,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn replication_interval(&self) -> u32 {
        self.replication_interval
    }

    #[wasm_bindgen(getter)]
    pub fn replication_threshold(&self) -> u64 {
        self.replication_threshold
    }
    #[wasm_bindgen]
    pub fn start_replication_distribution(&self) {
        let storage_node = self.storage_node.clone();
        let replication_threshold = self.replication_threshold;
        let interval_ms = self.replication_interval;

        let f = async move {
            loop {
                let promise = js_sys::Promise::new(&mut |resolve, _| {
                    web_sys::window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            interval_ms.try_into().unwrap(),
                        )
                        .unwrap();
                });
                JsFuture::from(promise).await.unwrap();

                if let Err(e) =
                    check_replication_distribution(&storage_node, replication_threshold).await
                {
                    console::error_1(&format!("Replication distribution error: {:?}", e).into());
                }
            }
        };

        spawn_local(f);
    }
}

async fn check_replication_distribution(
    storage_node: &JsValue,
    replication_threshold: u64,
) -> Result<(), JsValue> {
    let replications = js_sys::Reflect::get(storage_node, &JsValue::from_str("stored_proofs"))?;
    let replication_count = js_sys::Reflect::get(&replications, &JsValue::from_str("length"))?
        .as_f64()
        .unwrap() as u64;

    if replication_count >= replication_threshold {
        distribute_replications(&replications).await?;
    }

    Ok(())
}

async fn distribute_replications(_replications: &JsValue) -> Result<(), JsValue> {
    // Distribution logic here
    Ok(())
}
