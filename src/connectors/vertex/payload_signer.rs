use crate::domain::models::vertex::serializable_structs::SerializableOrder;
use crate::domain::models::vertex::sol_structs::Order;
use crate::domain::models::vertex::{
    serializable_structs::SerializableStreamAuthentication, sol_structs::StreamAuthentication,
};
use crate::shared::utils::eth_signer::EthSigner;
use crate::shared::utils::type_conv;
use alloy_sol_types::Eip712Domain;
use std::time::{SystemTime, UNIX_EPOCH};

// Signable trait defination
pub trait Signable {
    fn serialize_for_signing(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

impl Signable for StreamAuthentication {
    fn serialize_for_signing(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Serialize StreamAuthentication according to your requirements
        let serializable = SerializableStreamAuthentication::from(self);
        serde_json::to_vec(&serializable).map_err(Into::into)
    }
}

impl Signable for Order {
    fn serialize_for_signing(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Serialize Order according to your requirements
        let serializable = SerializableOrder::from(self);
        serde_json::to_vec(&serializable).map_err(Into::into)
    }
}

pub struct Signer<'a> {
    sender_address: String,
    eth_signer: &'a EthSigner,
    domain: &'a Eip712Domain,
}

impl<'a> Signer<'a> {
    pub fn new(
        sender_address: String,
        eth_signer: &'a EthSigner,
        domain: &'a Eip712Domain,
    ) -> Self {
        Signer {
            eth_signer,
            sender_address,
            domain,
        }
    }

    pub fn construct_ws_auth_payload(&self) -> Result<String, Box<dyn std::error::Error>> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let expiration = current_time + 60_000; // Expiration time set to 1 minute from now

        // Initialize StreamAuthentication using the abstracted method for generating sender bytes
        let tx_data = StreamAuthentication {
            sender: type_conv::hex_to_bytes32(&self.sender_address)?,
            expiration,
        };

        let serializable_auth = SerializableStreamAuthentication::from(&tx_data);
        let serialized_auth = serde_json::to_string(&serializable_auth)?;
        let serialized_auth_value: serde_json::Value = serde_json::from_str(&serialized_auth)?;

        // Leverage the generic sign method to sign the StreamAuthentication data
        let signature = self.sign(&tx_data)?;

        let payload = serde_json::json!({
            "method": "authenticate",
            "id": 0,
            "tx": serialized_auth_value,
            "signature": signature
        })
        .to_string();

        Ok(payload)
    }

    // Method to construct payload for placing an order
    pub fn construct_order_payload(
        &self,
        order: &Order,
        product_id: i32,
        id: i32,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Serialize the Order for signing
        let serialized_order = serde_json::to_string(&SerializableOrder::from(order))?;

        // Generate a signature for the serialized Order
        let signature = self.sign(order)?;

        // Construct the payload with the serialized Order and its signature
        let payload = serde_json::json!({
            "method": "placeOrder",
            "order": serde_json::from_str::<serde_json::Value>(&serialized_order)?,
            "signature": signature
        })
        .to_string();

        Ok(payload)
    }

    // Generic signing method for any Signable struct
    pub fn sign<T: Signable>(&self, signable: &T) -> Result<String, Box<dyn std::error::Error>> {
        let serialized_signable = signable.serialize_for_signing()?;
        let signature = self
            .eth_signer
            .generate_signature(&serialized_signable, self.domain)?;
        Ok(signature)
    }
}
