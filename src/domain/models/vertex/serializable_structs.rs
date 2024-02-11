use alloy_primitives::hex;
use serde::Serialize;

use super::sol_structs::{Order, StreamAuthentication};

#[derive(Serialize)]
pub struct SerializableStreamAuthentication {
    sender: String,
    expiration: String,
}

// Convert StreamAuthentication to SerializableStreamAuthentication
impl From<&StreamAuthentication> for SerializableStreamAuthentication {
    fn from(auth: &StreamAuthentication) -> Self {
        let sender = hex::encode(auth.sender.as_slice());
        let expiration = auth.expiration.to_string();

        SerializableStreamAuthentication { sender, expiration }
    }
}

#[derive(Serialize)]
pub struct SerializableOrder {
    sender: String,
    price_x18: String,
    amount: String,
    expiration: String,
    nonce: String,
}

// Convert Order to SerializableOrder
impl From<&Order> for SerializableOrder {
    fn from(order: &Order) -> Self {
        let sender = hex::encode(order.sender.as_slice());
        let price_x18 = order.priceX18.to_string();
        let amount = order.amount.to_string();
        let expiration = order.expiration.to_string();
        let nonce = order.nonce.to_string();

        SerializableOrder {
            sender,
            price_x18,
            amount,
            expiration,
            nonce,
        }
    }
}
