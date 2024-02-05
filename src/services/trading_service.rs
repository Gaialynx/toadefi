use std::{sync::Arc, time::Duration};
use tonic::{Request, Response, Status};

use crate::{
    connectors::vertex::{gateway_client::GatewayClient, subscription_client::SubscriptionClient},
    trading_service::{
        self, query_response, trading_service_server::TradingService,
        vertex_query_service_server::VertexQueryService, ConnectionRequest, ConnectionResponse,
        QueryRequest, QueryResponse, StatusResponse,
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
        print!("{:?}", request);
        let query_request = request.into_inner();
        let query_type = determine_query_type(&query_request);
        let response_data = self
            .gateway_client
            .send_query(query_type)
            .await
            .map_err(|e| Status::internal(format!("Query failed: {}", e)))?;

        let json: StatusResponse = serde_json::from_str(&response_data)
            .map_err(|e| Status::internal(format!("Failed to parse JSON: {}", e)))?;

        let response = StatusResponse {
            status: json.status.into(),
            data: json.data.into(),
            request_type: json.request_type.into(),
        };

        println!("{:?}", response);

        let response = QueryResponse {
            data: Some(query_response::Data::Status(response)),
        };

        Ok(Response::new(response))
    }
}
// Helper function to determine the query type
fn determine_query_type(query_request: &QueryRequest) -> &str {
    println!("InitiateConnection request: {:?}", query_request);
    match query_request.query {
        Some(trading_service::query_request::Query::StatusRequest(_)) => "status",
        Some(trading_service::query_request::Query::ContractsRequest(_)) => "contracts",
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
