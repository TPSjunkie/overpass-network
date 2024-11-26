// ./src/lib.rs
//! Main library module for the WASM-based Bitcoin functionality
//! 
//! Contains submodules for:
//! - bitcoin: Core Bitcoin protocol implementation
//! - common: Shared utilities and types
//! - core: Core business logic and data structures
//! - network: Networking and P2P functionality
//! - privacy: Privacy-enhancing features
//! - wasm: WebAssembly bindings and interfaces

pub mod bitcoin;
pub mod common;
pub mod core;
pub mod network;
pub mod privacy;
pub mod wasm;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
    
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(num: u32);
    
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    log(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn initialize() {
    log("Initializing WASM module...");
}

#[wasm_bindgen]
pub fn debug_log(message: &str) {
    log(message);
}

#[wasm_bindgen]
pub fn warn_log(message: &str) {
    warn(message);
}

#[wasm_bindgen]
pub fn error_log(message: &str) {
    error(message);
}

#[wasm_bindgen]
pub fn log_number(num: u32) {
    log_u32(num);
}

#[wasm_bindgen]
pub fn log_multiple(first: &str, second: &str) {
    log_many(first, second);
}

// Exported functions will be available for import in JavaScript
#[wasm_bindgen]
pub fn add(left: u32, right: u32) -> u32 {
    left + right
}

#[wasm_bindgen]
pub fn subtract(left: u32, right: u32) -> u32 {
    left - right
}

#[wasm_bindgen]
pub fn multiply(left: u32, right: u32) -> u32 {
    left * right
}

#[wasm_bindgen]
pub fn divide(left: u32, right: u32) -> u32 {
    left / right
}

#[wasm_bindgen]
pub fn power(base: u32, exponent: u32) -> u32 {
    base.pow(exponent)
}