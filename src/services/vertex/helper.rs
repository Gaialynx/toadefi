use tonic::Status;

use crate::shared::errors::connect_error::ConnectError;

use super::client::VertexClient;

pub trait VertexHelper {
    fn construct_query_message<T: serde::Serialize>(&self, request: &T) -> Result<String, Status>;
    async fn send_message_to_gateway(&self, query_message: String) -> Result<String, ConnectError>;
}

impl VertexHelper for VertexClient {
    fn construct_query_message<T: serde::Serialize>(&self, request: &T) -> Result<String, Status> {
        serde_json::to_string(request)
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))
    }

    async fn send_message_to_gateway(&self, query_message: String) -> Result<String, ConnectError> {
        self.gateway_client.send_message(query_message).await
    }
}
