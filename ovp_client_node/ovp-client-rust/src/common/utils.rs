// ./src/common/utils.rs

/// Converts a byte array to a hex string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex_string = String::new();
    for byte in bytes {
        hex_string.push_str(&format!("{:02x}", byte));
    }
    hex_string
}

/// Converts a hex string to a byte array.
pub fn hex_to_bytes(hex_string: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    for hex in hex_string.split_whitespace() {
        let byte = u8::from_str_radix(hex, 16).map_err(|_| "Invalid hex string".to_string())?;
        bytes.push(byte);
    }
    Ok(bytes)
}   