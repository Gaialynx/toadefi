use std::{sync::Arc, time::Duration};
use tonic::{Request, Response, Status};

use crate::{
    connectors::vertex::{gateway_client::GatewayClient, subscription_client::SubscriptionClient},
    trading_service::{
        self, query_response, trading_service_server::TradingService,
        vertex_query_service_server::VertexQueryService, ConnectionRequest, ConnectionResponse,
        ContractsResponse, ProductsResponse, QueryRequest, QueryResponse, StatusResponse,
    },
};

#[derive(Debug, Default)]
pub struct MyTradingService {
    // You might want to include shared state here
    pub subscription_client: Arc<SubscriptionClient>,
    pub gateway_client: Arc<GatewayClient>,
}

#[tonic::async_trait]
impl TradingService for MyTradingService {
    async fn initiate_connection(
        &self,
        request: Request<ConnectionRequest>,
    ) -> Result<Response<ConnectionResponse>, Status> {
        println!("InitiateConnection request: {:?}", request);
        // Call start_subscription here
        match self.subscription_client.start_subscription().await {
            Ok(_) => {
                // Upon successful subscription, spawn the check_and_reconnect task
                let subscription_client_clone = self.subscription_client.clone();
                tokio::spawn(async move {
                    MyTradingService::check_and_reconnect(subscription_client_clone).await;
                });

                let response = ConnectionResponse {
                    success: true,
                    message: "Connection initiated and subscription started".into(),
                };
                Ok(Response::new(response))
            }
            Err(e) => {
                // Handle subscription failure
                let response = ConnectionResponse {
                    success: false,
                    message: format!("Failed to start subscription: {}", e),
                };
                Ok(Response::new(response))
            }
        }
    }
}

#[tonic::async_trait]
impl VertexQueryService for MyTradingService {
    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        println!("Hello {:?}", request);
        let query_request = request.into_inner();
        let query_type = determine_query_type(&query_request);
        println!("Creating message for: getting {}", query_type);

        // create query message based on the type of request
        /*
        Creating message for: getting get_status, get_all_products
         */
        let query_message = serde_json::json!({ "type": query_type }).to_string();

        let response_data = self
            .gateway_client
            .send_query_with_type(query_message)
            .await
            .map_err(|e| Status::internal(format!("Query failed: {}", e)))?;

        // get response based on query type

        match query_type {
            "status" => {
                let json: StatusResponse = serde_json::from_str(&response_data)
                    .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

                let response = QueryResponse {
                    data: Some(query_response::Data::Status(json)),
                };

                Ok(Response::new(response))
            }
            "contracts" => {
                let json: ContractsResponse = serde_json::from_str(&response_data)
                    .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

                let response = QueryResponse {
                    data: Some(query_response::Data::Contracts(json)),
                };

                Ok(Response::new(response))
            }
            "all_products" => {
                // Parse the JSON response and return it
                let json: ProductsResponse = serde_json::from_str(&response_data)
                    .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

                println!("{:?}", json);
                let response = QueryResponse {
                    data: Some(query_response::Data::Products(json)),
                };

                Ok(Response::new(response))
            }
            // Handle other query types...
            _ => Err(Status::internal("Unsupported query type")),
        }
    }
}
// Helper function to determine the query type
fn determine_query_type(query_request: &QueryRequest) -> &str {
    println!("InitiateConnection request: {:?}", query_request);
    match query_request.query {
        Some(trading_service::query_request::Query::StatusRequest(_)) => "status",
        Some(trading_service::query_request::Query::ContractsRequest(_)) => "contracts",
        Some(trading_service::query_request::Query::ProductsRequest(_)) => "all_products",
        // Add other cases as needed
        None => "unknown",
    }
}

impl MyTradingService {
    async fn check_and_reconnect(vertex_client: Arc<SubscriptionClient>) {
        // Continuously check and reconnect in another task
        tokio::spawn(async move {
            loop {
                vertex_client.check_and_reconnect().await;
                tokio::time::sleep(Duration::from_secs(5)).await; // Adjust the timing as ye see fit
            }
        });
    }
}
