use std::{sync::Arc, time::Duration};
use tonic::{Request, Response, Status};

use crate::{
    connectors::vertex::{gateway_client::GatewayClient, subscription_client::SubscriptionClient},
    trading_service::{
        trading_service_server::TradingService, ConnectionRequest, ConnectionResponse,
    },
};

#[derive(Debug)]
pub struct VertexClient {
    // You might want to include shared state here
    pub subscription_client: Arc<SubscriptionClient>,
    pub gateway_client: Arc<GatewayClient>,
}

impl VertexClient {
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

#[tonic::async_trait]
impl TradingService for VertexClient {
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
                    VertexClient::check_and_reconnect(subscription_client_clone).await;
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
