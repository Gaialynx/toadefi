use alloy_primitives::{hex, FixedBytes, Keccak256};
use ethsign::{Protected, SecretKey};

#[derive(Debug)]
pub struct EthSigner {
    private_key: SecretKey,
}

impl EthSigner {
    pub fn new(arbitrum_private_key: &str) -> Self {
        let key_bytes = hex::decode(arbitrum_private_key).unwrap();
        let protected_key = Protected::new(key_bytes);
        let private_key = SecretKey::from_raw(protected_key.as_ref()).unwrap();
        EthSigner { private_key }
    }

    pub fn generate_signature(
        &self,
        signing_hash: &[u8],
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Sign the hash
        let signature = self.private_key.sign(signing_hash).unwrap(); // Consider handling this Result instead of unwrapping
        let (r, s, v) = (signature.r, signature.s, signature.v);

        // Adjust v for Ethereum signature compatibility (if necessary)
        let adjusted_v = v + 27;

        // Concatenate r, s, and v to form the final signature
        let final_sig = [r.as_ref(), s.as_ref(), &[adjusted_v]].concat();
        let final_sig_hex = hex::encode(final_sig);

        Ok(format!("0x{}", final_sig_hex))
    }

    pub fn generate_digest(&self, signable_bytes: FixedBytes<32>) -> String {
        let mut hasher = Keccak256::new();
        hasher.update(&signable_bytes.as_slice());
        let hash = hasher.finalize();
        let hex_string = hex::encode(hash);

        format!("0x{}", hex_string)
    }
}
