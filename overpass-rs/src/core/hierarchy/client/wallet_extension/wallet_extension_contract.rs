use js_sys::Uint8Array;

use serde::{Deserialize, Serialize};
use sha2::Digest;
use wasm_bindgen::prelude::*;

// Use statements for internal modules and imports for needed functionalities.

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
interface ByteArray32 {
    toArray(): Uint8Array;
}
"#;

// Implementing ByteArray32 structure with various methods and conversions.

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ByteArray32(#[wasm_bindgen(skip)] pub [u8; 32]);

#[wasm_bindgen]
impl ByteArray32 {
    #[wasm_bindgen(constructor)]
    pub fn new(array: &[u8]) -> Result<ByteArray32, JsValue> {
        if array.len() != 32 {
            return Err(JsValue::from_str("Array must be 32 bytes long"));
        }
        let mut result = [0u8; 32];
        result.copy_from_slice(array);
        Ok(ByteArray32(result))
    }

    #[wasm_bindgen(js_name = fromWasmAbi)]
    pub fn from_wasm_abi(val: JsValue) -> Result<ByteArray32, JsValue> {
        let array = Uint8Array::new(&val);
        let vec = array.to_vec();
        Self::new(&vec)
    }

    #[wasm_bindgen(js_name = toWasmAbi)]
    pub fn to_wasm_abi(&self) -> JsValue {
        let array = Uint8Array::new_with_length(32);
        array.copy_from(&self.0);
        array.into()
    }

    pub fn to_array(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_string(&self) -> String {
        hex::encode(self.0.to_vec())
    }
    pub fn from_string(val: &str) -> Result<ByteArray32, JsValue> {
        let array = hex::decode(val).map_err(|_| JsValue::from_str("Invalid hex string"))?;
        Self::new(&array)
    }
}

// Creating structures for proof verification and blockchain operations, including a wallet extension system.
#[wasm_bindgen]
#[derive(Serialize, Deserialize, Clone)]
pub struct ZkProof(Vec<u8>);
