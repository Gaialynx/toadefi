use tonic::{Response, Status};

use crate::vertex_execute::{
    vertex_execute_service_server::VertexExecuteService, PlaceOrderRequest, PlaceOrderResponse,
};

use super::{client::VertexClient, helper::VertexHelper};

#[tonic::async_trait]
impl VertexExecuteService for VertexClient {
    async fn place_order(
        &self,
        request: tonic::Request<PlaceOrderRequest>,
    ) -> Result<tonic::Response<PlaceOrderResponse>, tonic::Status> {
        let order_request = request.into_inner();
        let execute_message = self.construct_query_message(&order_request)?;

        let response_data = self
            .send_message_to_gateway(execute_message)
            .await
            .expect("Failed to send message to gateway");

        let json: PlaceOrderResponse = serde_json::from_str(&response_data)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        Ok(Response::new(json))
    }
}
