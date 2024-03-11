use alloy_primitives::FixedBytes;

// Helper to convert HEX to FIXEDBytes<32> specifically used for solidity EIP712
pub fn hex_to_fixed_bytes32(hex_str: &str) -> Result<FixedBytes<32>, Box<dyn std::error::Error>> {
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

pub fn vec_to_fixed_bytes32(vec: Vec<u8>) -> Result<FixedBytes<32>, &'static str> {
    if vec.len() != 32 {
        Err("Vector must be 32 bytes long")
    } else {
        let mut bytes = FixedBytes::default(); // Assuming FixedBytes::default() gives you a zero-initialized array
        bytes[..].copy_from_slice(&vec);
        Ok(bytes)
    }
}

pub fn pad_to_fixed_bytes32(input: &Vec<u8>) -> Result<FixedBytes<32>, Box<dyn std::error::Error>> {
    // Ensure the input length is at most 32 bytes
    if input.len() > 32 {
        return Err("Input must be at most 32 bytes long".into());
    }

    // Create a zero-initialized array of 32 bytes
    let mut bytes = [0u8; 32];

    // Copy the input bytes into the array, starting at the beginning
    bytes[..input.len()].copy_from_slice(&input);

    // Convert the array to FixedBytes
    Ok(FixedBytes::try_from(bytes)?)
}

pub fn fixed_bytes_to_hex(bytes: &FixedBytes<32>) -> String {
    hex::encode(bytes)
}
