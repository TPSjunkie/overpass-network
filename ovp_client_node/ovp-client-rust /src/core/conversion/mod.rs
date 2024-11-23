/*
Conversion Module - Consolidated
==============================
- Merged from:
  1. Original core/conversion/conversion_client.rs
  2. WASM-specific conversions from wasm/conversion_wasm.rs
- Handles all data conversion needs for the client
- Includes WASM type conversions
- Removed server-specific conversion logic
*/
// ./src/core/conversion/mod.rs

pub mod conversion_client;
