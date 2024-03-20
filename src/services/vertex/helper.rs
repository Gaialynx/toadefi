use std::{
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};

use tonic::Status;

use crate::{shared::errors::connect_error::ConnectError, vertex_query::ContractsResponse};

use super::client::VertexClient;

pub trait VertexHelper {
    fn generate_expiration_time(&self, seconds_from_now: u64, order_type: u8) -> u64;
    fn construct_query_message<T: serde::Serialize>(&self, request: &T) -> Result<String, Status>;
    async fn send_message_to_gateway(&self, query_message: String) -> Result<String, ConnectError>;
    fn generate_nonce(&self) -> u64;
    async fn get_contract_addr(&self, product_id: u32) -> Option<String>;
}

impl VertexHelper for VertexClient {
    fn construct_query_message<T: serde::Serialize>(&self, request: &T) -> Result<String, Status> {
        serde_json::to_string(request)
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))
    }

    async fn send_message_to_gateway(&self, query_message: String) -> Result<String, ConnectError> {
        self.gateway_client.send_message(query_message).await
    }

    // todo add 50 to unixepoch to see the results
    fn generate_nonce(&self) -> u64 {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            + 5000;

        let random_part = 1000; // TODO: Ensure this part is randomly generated in production

        ((now_ms << 20) as u64) + random_part // Shift left by 20 bits and add the random part
    }

    // order types (0 for default, 1 for IOC, 2 for FOK, and 3 for post-only)
    fn generate_expiration_time(&self, seconds_from_now: u64, order_type: u8) -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let now_secs = since_the_epoch.as_secs();

        let expiration = now_secs + seconds_from_now;
        let order_type_bits: u64 = u64::from(order_type) << 62; // Shift order_type into the most significant 2 bits

        expiration | order_type_bits // Combine expiration with order_type_bits
    }

    // get verifying contract order address for signing place order
    async fn get_contract_addr(&self, product_id: u32) -> Option<String> {
        const MSG: &str = "{
            \"type\":\"contracts\"
        }";

        let response_data = self
            .send_message_to_gateway(MSG.to_string())
            .await
            .expect("Failed to send message to gateway");

        let contracts: ContractsResponse = serde_json::from_str(&response_data).ok()?; // Convert Result to Option, discarding the error

        contracts
            .data
            .map(|d| d.book_addrs[product_id as usize].clone())
    }
}
