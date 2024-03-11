use std::time::{SystemTime, UNIX_EPOCH};

use tonic::Status;

use crate::shared::errors::connect_error::ConnectError;

use super::client::VertexClient;

pub trait VertexHelper {
    fn generate_expiration_time(&self, seconds_from_now: u64, order_type: u8) -> u64;
    fn construct_query_message<T: serde::Serialize>(&self, request: &T) -> Result<String, Status>;
    async fn send_message_to_gateway(&self, query_message: String) -> Result<String, ConnectError>;
    fn generate_nonce(&self) -> u64;
}

impl VertexHelper for VertexClient {
    fn construct_query_message<T: serde::Serialize>(&self, request: &T) -> Result<String, Status> {
        serde_json::to_string(request)
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))
    }

    async fn send_message_to_gateway(&self, query_message: String) -> Result<String, ConnectError> {
        self.gateway_client.send_message(query_message).await
    }

    fn generate_nonce(&self) -> u64 {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let random_part = 1000; // Ensure this part is randomly generated in production

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
        let order_type_bits = u64::from(order_type) << 62; // Shift order_type into the most significant 2 bits

        expiration | order_type_bits // Combine expiration with order_type_bits
    }
}
