use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use tonic::Status;

// Define an error type for your application
#[derive(Debug)]
pub struct ApiError {
    code: StatusCode,
    message: String,
}

impl ApiError {
    fn new(code: StatusCode, message: &str) -> Self {
        ApiError {
            code,
            message: message.to_string(),
        }
    }
}

impl From<tonic::Status> for ApiError {
    fn from(status: Status) -> Self {
        // Map tonic::Status to an appropriate HTTP status code and message
        let code = match status.code() {
            tonic::Code::NotFound => StatusCode::NOT_FOUND,
            tonic::Code::InvalidArgument => StatusCode::BAD_REQUEST,
            // Add more mappings as necessary
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        ApiError::new(code, status.message())
    }
}

// Implement IntoResponse for ApiError to convert it to an Axum response
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(serde_json::json!({ "error": self.message }));
        (self.code, body).into_response()
    }
}
