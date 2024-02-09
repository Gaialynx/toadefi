// src/api/mod.rs

mod handlers;

use crate::services::vertex::client::VertexClient;
use axum::{routing::post, Extension, Router};
use std::sync::Arc;

pub fn router(trading_service: Arc<VertexClient>) -> Router {
    Router::new()
        .route(
            "/initiate_connection",
            post(handlers::initiate_connection_handler),
        )
        .route("/query/status", post(handlers::query_status_handler))
        // Add more routes here for other gRPC methods
        .layer(Extension(trading_service))
}
