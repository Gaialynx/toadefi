// src/api/handlers.rs

use crate::{
    services::trading_service::MyTradingService,
    trading_service::{
        trading_service_server::TradingService, vertex_query_service_server::VertexQueryService,
        ConnectionRequest, ConnectionResponse, QueryRequest, QueryResponse,
    },
};

use crate::utils::errors::api_error::ApiError;
use axum::{Extension, Json};
use http::StatusCode;
use std::sync::Arc;
use tonic::Request;
use tracing::{error, info};

#[axum::debug_handler]
pub async fn initiate_connection_handler(
    Extension(trading_service): Extension<Arc<MyTradingService>>,
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
    Extension(trading_service): Extension<Arc<MyTradingService>>,
    Json(payload): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    match trading_service.as_ref().query(Request::new(payload)).await {
        Ok(response) => Ok(Json(response.into_inner())),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
