use crate::core::error::errors::{SystemError, SystemErrorType};
use crate::core::hierarchy::client::*;
use crate::core::storage_node::*;
use crate::core::tokens::*;
use crate::core::types::*;
use crate::core::zkps::proof::*;
use crate::core::zkps::zkp_interface::*;
use crate::core::zkps::{plonky2::*, *};
use js_sys::{Date, Promise, Uint8Array};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;
use web_sys::{console, window};

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
interface ByteArray32 {
    toArray(): Uint8Array;
}
"#;

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
}

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ByteArray20(#[wasm_bindgen(skip)] pub [u8; 20]);

#[wasm_bindgen]
impl ByteArray20 {
    #[wasm_bindgen(constructor)]
    pub fn new(array: &[u8]) -> Result<ByteArray20, JsValue> {
        if array.len() != 20 {
            return Err(JsValue::from_str("Array must be 20 bytes long"));
        }
        let mut result = [0u8; 20];
        result.copy_from_slice(array);
        Ok(ByteArray20(result))
    }

    #[wasm_bindgen(js_name = fromWasmAbi)]
    pub fn from_wasm_abi(val: JsValue) -> Result<ByteArray20, JsValue> {
        let array = Uint8Array::new(&val);
        let vec = array.to_vec();
        Self::new(&vec)
    }

    #[wasm_bindgen(js_name = toWasmAbi)]
    pub fn to_wasm_abi(&self) -> JsValue {
        let array = Uint8Array::new_with_length(20);
        array.copy_from(&self.0);
        array.into()
    }

    pub fn to_array(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct ByteArray8(#[wasm_bindgen(skip)] pub [u8; 8]);

#[wasm_bindgen]
impl ByteArray8 {
    #[wasm_bindgen(constructor)]
    pub fn new(array: &[u8]) -> Result<ByteArray8, JsValue> {
        if array.len() != 8 {
            return Err(JsValue::from_str("Array must be 8 bytes long"));
        }
        let mut result = [0u8; 8];
        result.copy_from_slice(array);
        Ok(ByteArray8(result))
    }

    #[wasm_bindgen(js_name = fromWasmAbi)]
    pub fn from_wasm_abi(val: JsValue) -> Result<ByteArray8, JsValue> {
        let array = Uint8Array::new(&val);
        let vec = array.to_vec();
        Self::new(&vec)
    }

    #[wasm_bindgen(js_name = toWasmAbi)]
    pub fn to_wasm_abi(&self) -> JsValue {
        let array = Uint8Array::new_with_length(8);
        array.copy_from(&self.0);
        array.into()
    }

    pub fn to_array(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

#[wasm_bindgen]
impl ByteArray32 {
    #[wasm_bindgen(js_name = fromHex)]
    pub fn from_hex(hex: &str) -> Result<ByteArray32, JsValue> {
        let bytes = hex::decode(hex).map_err(|_| JsValue::from_str("Invalid hex string"))?;
        Self::new(&bytes)
    }

    #[wasm_bindgen(js_name = toHex)]
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}
