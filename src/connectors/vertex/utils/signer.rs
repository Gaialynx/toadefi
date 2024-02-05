use super::stream_authentication::{SerializableStreamAuthentication, StreamAuthentication};
use crate::connectors::vertex::vertex_auth::VertexAuth;
use crate::utils::type_conv;
use alloy_sol_types::Eip712Domain;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Signer<'a> {
    sender_address: String,
    vertex_auth: &'a VertexAuth,
    domain: &'a Eip712Domain,
}

impl<'a> Signer<'a> {
    pub fn new(
        sender_address: String,
        vertex_auth: &'a VertexAuth,
        domain: &'a Eip712Domain,
    ) -> Self {
        Signer {
            vertex_auth,
            sender_address,
            domain,
        }
    }

    pub fn construct_ws_auth_payload(&self) -> Result<String, Box<dyn std::error::Error>> {
        let sender_bytes32 = type_conv::hex_to_bytes32(&self.sender_address)?;

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let expiration = current_time + 60_000;

        let tx_data = StreamAuthentication {
            sender: sender_bytes32,
            expiration,
        };

        let stream_auth = StreamAuthentication::from(tx_data);
        let signature = self
            .vertex_auth
            .generate_signature(&stream_auth, self.domain)?;

        let serializable_auth = SerializableStreamAuthentication::from(&stream_auth);
        let serialized_auth = serde_json::to_string(&serializable_auth)?;

        let serialized_auth_value: serde_json::Value = serde_json::from_str(&serialized_auth)?;
        let payload = serde_json::json!({
            "method": "authenticate",
            "id": 0,
            "tx": serialized_auth_value,
            "signature": signature
        })
        .to_string();

        Ok(payload)
    }
}
