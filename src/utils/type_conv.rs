use alloy_primitives::FixedBytes;

// Helper to convert HEX to FIXEDBytes<32> specifically used for solidity EIP712
pub fn hex_to_bytes32(hex_str: &str) -> Result<FixedBytes<32>, Box<dyn std::error::Error>> {
    let mut bytes = [0u8; 32];
    // Decode the hex string and get a slice of bytes
    let decoded_bytes = hex::decode(hex_str.trim_start_matches("0x"))?;
    // Ensure the decoded bytes are not longer than 32 bytes
    if decoded_bytes.len() > 32 {
        return Err("Hex string too long".into());
    }
    // Place the decoded bytes at the beginning of the 32-byte array
    bytes[..decoded_bytes.len()].copy_from_slice(&decoded_bytes);
    Ok(FixedBytes::from(bytes))
}
