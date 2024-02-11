// src/api/handlers.rs

use crate::services::vertex::client::VertexClient;
use crate::trading_service::trading_service_server::TradingService;
use crate::trading_service::{ConnectionRequest, ConnectionResponse};
use crate::vertex_query::vertex_query_service_server::VertexQueryService;
use crate::vertex_query::{StatusRequest, StatusResponse};

use crate::shared::errors::api_error::ApiError;
use axum::{Extension, Json};
use http::StatusCode;
use std::sync::Arc;
use tonic::Request;
use tracing::{error, info};

#[axum::debug_handler]
pub async fn initiate_connection_handler(
    Extension(trading_service): Extension<Arc<VertexClient>>,
    Json(payload): Json<ConnectionRequest>,
) -> Result<Json<ConnectionResponse>, ApiError> {
    info!("Received initiate_connection request: {:?}", payload);

    // Now returns a Result with ApiError for errors
    let grpc_request = tonic::Request::new(payload);

    match trading_service
        .as_ref()
        .initiate_connection(grpc_request)
        .await
    {
        Ok(grpc_response) => Ok({
            info!("Successfully initiated connection");
            Json(grpc_response.into_inner())
        }),
        Err(e) => Err({
            error!("Error initiating connection: {:?}", e);
            e.into()
        }), // Convert tonic::Status to ApiError
    }
}

#[axum::debug_handler]
pub async fn query_status_handler(
    Extension(vertex_client): Extension<Arc<VertexClient>>,
    Json(payload): Json<StatusRequest>,
) -> Result<Json<StatusResponse>, StatusCode> {
    match vertex_client.as_ref().status(Request::new(payload)).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
