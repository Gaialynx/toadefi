use crate::domain::models::vertex::serializable_structs::SerializableOrder;
use crate::domain::models::vertex::sol_structs::Order;
use crate::domain::models::vertex::{
    serializable_structs::SerializableStreamAuthentication, sol_structs::StreamAuthentication,
};
use crate::shared::utils::eth_signer::EthSigner;
use crate::shared::utils::type_conv;
use alloy_sol_types::Eip712Domain;
use regex::Regex;
use serde_json::{json, Value};
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
    sender_address: &'a String,
    eth_signer: &'a EthSigner,
    domain: &'a Eip712Domain,
}

impl<'a> Signer<'a> {
    pub fn new(
        sender_address: &'a String,
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
            sender: type_conv::hex_to_fixed_bytes32(self.sender_address)?,
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
        let serialized_order_raw = serde_json::to_string(&SerializableOrder::from(order))
            .map_err(|e| format!("Error serializing order: {}", e))?;
    
            let serialized_order = adjust_field_names(&serialized_order_raw)?;
        println!("Serialized {:?}", serialized_order);
 
        // Generate a signature for the serialized Order
        let signature = self.sign(order)
            .map_err(|e| format!("Error generating signature: {}", e))?;
    
        if signature.is_empty() {
            return Err("Signature is empty, signing failed".into());
        }
    
        let order_value = serde_json::from_str::<Value>(&serialized_order)?;
        
        // Construct the nested 'place_order' payload with the order details, signature, and other required fields
        let payload = json!({
            "place_order": {
                "product_id": product_id,
                "order": order_value,
                "signature": signature,
                "id": id,
            }
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


pub fn adjust_field_names(serialized: &str) -> Result<String, regex::Error> {
    let re = Regex::new(r#""price_x18":"#)?; // Match the exact field name in quotes
    Ok(re.replace_all(serialized, r#""priceX18":"#).into_owned()) // Replace with the desired field name
}