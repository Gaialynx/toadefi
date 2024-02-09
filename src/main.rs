mod api;
mod config;
mod connectors;
mod domain;
mod services;
mod utils;

// Include the generated protobuf code
// Include the generated protobuf modules
pub mod trading_service {
    tonic::include_proto!("trading");
}
pub mod vertex_products {
    tonic::include_proto!("vertex_products");
}
pub mod vertex_symbols {
    tonic::include_proto!("vertex_symbols");
}
pub mod vertex_query {
    tonic::include_proto!("vertex_query");
}
pub mod vertex_execute {
    tonic::include_proto!("vertex_execute");
}

use crate::api::router as api_router;
use config::Config;
use connectors::vertex::{gateway_client::GatewayClient, subscription_client::SubscriptionClient};
use std::sync::Arc;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};

use crate::services::vertex::client::VertexClient;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    let subscriber = FmtSubscriber::builder().finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Create a new instance of the SubscriptionClient
    let config = Config::new();
    let subscription_client = Arc::new(SubscriptionClient::new(config.clone()));

    // Create a new instance of the GatewayClient
    let gateway_client = Arc::new(GatewayClient::new(config.clone()));
    let trading_service = VertexClient {
        subscription_client: Arc::clone(&subscription_client),
        gateway_client: Arc::clone(&gateway_client),
    };

    // Create a new instance of the VertexQueryService
    let vertex_query_service = VertexClient {
        subscription_client: Arc::clone(&subscription_client),
        gateway_client: Arc::clone(&gateway_client),
    };
    let vertex_query_service_arc = Arc::new(vertex_query_service);

    let cors = CorsLayer::new().allow_origin(Any);
    let addr = "[::1]:1321".parse()?;

    // Start the gRPC server
    let grpc_server = tokio::spawn(async move {
        Server::builder()
            .accept_http1(true)
            .layer(GrpcWebLayer::new())
            .layer(cors)
            .add_service(tonic_web::enable(
                trading_service::trading_service_server::TradingServiceServer::new(trading_service),
            ))
            .add_service(tonic_web::enable(
                vertex_query::vertex_query_service_server::VertexQueryServiceServer::new(
                    VertexClient {
                        subscription_client: Arc::clone(&subscription_client),
                        gateway_client: Arc::clone(&gateway_client),
                    },
                ),
            ))
            .serve(addr)
            .await
            .expect("gRPC server failed to start");
    });

    // Start the HTTP server
    let http_app = api_router(vertex_query_service_arc.clone());
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1322").await.unwrap();
    let http_server = tokio::spawn(async move {
        axum::serve(listener, http_app)
            .await
            .expect("HTTP server failed to start");
    });

    println!("TradingServer GRPC listening on {}", addr);
    println!("TradingServer HTTP listening on {}", "[::1]:1322");
    let _ = tokio::try_join!(grpc_server, http_server);

    Ok(())
}
