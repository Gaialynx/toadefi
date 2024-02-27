use std::time::{SystemTime, UNIX_EPOCH};

use tonic::Status;

use crate::shared::errors::connect_error::ConnectError;

use super::client::VertexClient;

pub trait VertexHelper {
    fn construct_query_message<T: serde::Serialize>(&self, request: &T) -> Result<String, Status>;
    async fn send_message_to_gateway(&self, query_message: String) -> Result<String, ConnectError>;
    fn generate_nonce(&self) -> u64;
    fn generate_expiration_time(&self,seconds_from_now: u64) -> u64;
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
        // Get the current time in milliseconds
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        let now_ms = since_the_epoch.as_millis();
    
        // Add 50 ms to the current time for the recv_time
        let recv_time = now_ms + 50;
    
        // Generate a random integer for the least significant 20 bits
        // Note: Make sure to use a proper random function for production code
        let random_int = 1000; // Example random integer
    
        // Combine recv_time and random integer into the nonce
        ((recv_time << 20) + random_int as u128) as u64
    }

    fn generate_expiration_time(&self,seconds_from_now: u64) -> u64 {
        // Get the current time
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let now_secs = since_the_epoch.as_secs();
    
        // Add the specified number of seconds to the current time
        now_secs + seconds_from_now
    }
}
