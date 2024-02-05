// src/api/mod.rs

mod handlers;

use crate::services::trading_service::MyTradingService;
use axum::{routing::post, Extension, Router};
use std::sync::Arc;

pub fn router(trading_service: Arc<MyTradingService>) -> Router {
    Router::new()
        .route(
            "/initiate_connection",
            post(handlers::initiate_connection_handler),
        )
        .route("/query/status", post(handlers::query_status_handler))
        // Add more routes here for other gRPC methods
        .layer(Extension(trading_service))
}
