use super::{client::VertexClient, helper::VertexHelper};
use crate::{
    vertex_products::ProductsResponse,
    vertex_query::{
        vertex_query_service_server::VertexQueryService, ContractsRequest, ContractsResponse,
        ProductsRequest, StatusRequest, StatusResponse,
    },
    vertex_symbols::{SymbolsRequest, SymbolsResponse},
};
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl VertexQueryService for VertexClient {
    async fn status(
        &self,
        request: Request<StatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let query_message = self.construct_query_message(&request.into_inner())?;

        let response_data = self
            .send_message_to_gateway(query_message)
            .await
            .expect("Failed to send message to gateway");

        let json: StatusResponse = serde_json::from_str(&response_data)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        Ok(Response::new(json))
    }

    async fn contracts(
        &self,
        request: Request<ContractsRequest>,
    ) -> Result<Response<ContractsResponse>, Status> {
        let query_message = self.construct_query_message(&request.into_inner())?;
        println!("query_message: {:?}", query_message);

        let response_data = self
            .send_message_to_gateway(query_message)
            .await
            .expect("Failed to send message to gateway");
        let json: ContractsResponse = serde_json::from_str(&response_data)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        Ok(Response::new(json))
    }

    async fn products(
        &self,
        request: Request<ProductsRequest>,
    ) -> Result<Response<ProductsResponse>, Status> {
        let query_message = self.construct_query_message(&request.into_inner())?;
        let response_data = self
            .send_message_to_gateway(query_message)
            .await
            .expect("Failed to send message to gateway");
        let json: ProductsResponse = serde_json::from_str(&response_data)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        Ok(Response::new(json))
    }

    async fn symbols(
        &self,
        request: Request<SymbolsRequest>,
    ) -> Result<Response<SymbolsResponse>, Status> {
        let query_message = self.construct_query_message(&request.into_inner())?;

        let response_data = self
            .send_message_to_gateway(query_message)
            .await
            .expect("Failed to send message to gateway");

        let json: SymbolsResponse = serde_json::from_str(&response_data)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        Ok(Response::new(json))
    }
}
