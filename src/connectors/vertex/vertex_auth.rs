use alloy_primitives::hex;
use alloy_sol_types::{Eip712Domain, SolStruct};
use ethsign::{Protected, SecretKey};

pub struct VertexAuth {
    private_key: SecretKey,
}

impl VertexAuth {
    pub fn new(arbitrum_private_key: &str) -> Self {
        let key_bytes = hex::decode(arbitrum_private_key).unwrap();
        let protected_key = Protected::new(key_bytes);
        let private_key = SecretKey::from_raw(protected_key.as_ref()).unwrap();
        VertexAuth { private_key }
    }

    pub fn generate_signature<P: SolStruct>(
        &self,
        payload: &P,
        domain: &Eip712Domain,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let signing_hash = payload.eip712_signing_hash(&domain);
        let signature = self.private_key.sign(signing_hash.as_slice()).unwrap();
        let (r, s, v) = (signature.r, signature.s, signature.v);
        let adjusted_v = v + 27;

        let final_sig = [r.as_ref(), s.as_ref(), &[adjusted_v]].concat();
        let final_sig_hex = hex::encode(final_sig);

        Ok(format!("0x{}", final_sig_hex))
    }

    // pub fn generate_digest(&self, signable_bytes: FixedBytes<32>) -> String {
    //     let mut hasher = Keccak256::new();
    //     hasher.update(&signable_bytes.as_slice());
    //     let hash = hasher.finalize();
    //     let hex_string = hex::encode(hash);

    //     format!("0x{}", hex_string)
    // }
}
