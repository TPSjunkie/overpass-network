use wasm_bindgen::JsValue;
use web_sys::console;
use wasm_bindgen_futures::spawn_local;
use crate::core::error::errors::SystemError;
use crate::core::storage_node::storage_node_contract::StorageNode;
use std::sync::Arc;

pub trait Duration {
    fn as_millis(&self) -> u64;
}

pub struct ResponseManager {
    storage_node: Arc<StorageNode>,
    response_threshold: u64,
    response_interval: u64,
    is_verifying: bool,
}
impl ResponseManager {
    pub fn start_verification(&self) {
        let manager = self.clone();
        spawn_local(async move {
            let window = web_sys::window().expect("no global window exists");
            let performance = window
                .performance()
                .expect("performance should be available");

            loop {
                let start_time = performance.now();
                if let Err(e) = manager.check_response_verification().await {
                    console::error_1(&format!("Response verification error: {:?}", e).into());
                }

                let elapsed = performance.now() - start_time;
                if elapsed < manager.response_interval as f64 {
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                        &mut |resolve, _| {
                            window
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    &resolve,
                                    (manager.response_interval as f64 - elapsed) as i32,
                                )
                                .expect("failed to set timeout");
                        },
                    ))
                    .await
                    .expect("failed to await timeout");
                }
            }
        });
    }

    pub fn stop_verification(&self) {
        // Set a flag or signal to break the loop and stop the verification process
        // This would require adding a flag field to ResponseManager and checking it in the loop
        // For now, this is a placeholder as the actual implementation would need additional state management
    }
}
pub struct ResponseManagerWrapper(ResponseManager);

impl ResponseManagerWrapper {
    pub fn new(
        storage_node: JsValue,
        response_threshold: u64,
        response_interval: u64,
    ) -> Result<ResponseManagerWrapper, JsValue> {
        let storage_node: Arc<StorageNode> = Arc::new(StorageNode::from_js_value(storage_node)?);
        Ok(ResponseManagerWrapper(ResponseManager {
            storage_node,
            response_threshold,
            response_interval,
            is_verifying: false,
        }))
    }

    pub fn start_response_verification(&self) {
        self.0.start_verification();
    }

    pub fn stop_response_verification(&self) {
        self.0.stop_verification();
    }
}
impl ResponseManager {
    pub fn start_response_verification(&self) {
        let manager = self.clone();
        spawn_local(async move {
            let window = web_sys::window().expect("no global window exists");
            let performance = window
                .performance()
                .expect("performance should be available");

            loop {
                let start_time = performance.now();
                if let Err(e) = manager.check_response_verification().await {
                    console::error_1(&format!("Response verification error: {:?}", e).into());
                }

                // Simulate interval using performance.now()
                let elapsed = performance.now() - start_time;
                if elapsed < manager.response_interval as f64 {
                    wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(
                        &mut |resolve, _| {
                            window
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    &resolve,
                                    (manager.response_interval as f64 - elapsed) as i32,
                                )
                                .expect("should register timeout");
                        },
                    ))
                    .await
                    .expect("timeout should complete");
                }
            }
        });
    }

    async fn check_response_verification(&self) -> Result<(), SystemError> {
        let response_threshold = self.response_threshold;

        let boc = self.storage_node.retrieve_boc(&[0u8; 32]).await?;
        let response_count = boc.cells.len();

        if response_count >= response_threshold as usize {
            let responses: Vec<[u8; 32]> = boc
                .cells
                .into_iter()
                .filter_map(|item| {
                    if item.data.len() == 32 {
                        let mut array = [0u8; 32];
                        array.copy_from_slice(&item.data);
                        Some(array)
                    } else {
                        None
                    }
                })
                .collect();
            self.verify_responses(responses).await?;
        }

        Ok(())
    }
    async fn verify_responses(&self, responses: Vec<[u8; 32]>) -> Result<(), SystemError> {
        for proof in responses {
            match self.storage_node.retrieve_proof(&proof).await {
                Ok(zk_proof) => {
                    match zk_proof.verify() {
                        Ok(is_valid) => {
                            if !is_valid {
                                // Assuming there's no apply_penalty method, we'll just log the error
                                console::error_1(&format!("Invalid proof: {:?}", proof).into());
                                break;
                            }
                        }
                        Err(e) => {
                            console::error_1(&format!("Error verifying proof: {:?}", e).into());
                            break;
                        }
                    }
                }
                Err(e) => {
                    console::error_1(&format!("Error retrieving proof: {:?}", e).into());
                    break;
                }
            }
        }

        Ok(())
    }
}

impl Clone for ResponseManager {
    fn clone(&self) -> Self {
        Self {
            storage_node: Arc::clone(&self.storage_node),
            response_threshold: self.response_threshold,
            response_interval: self.response_interval,
            is_verifying: self.is_verifying,
        }
    }
}
