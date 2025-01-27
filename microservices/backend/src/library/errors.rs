use axum::{http::StatusCode, response::IntoResponse};
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

/// A default error response for most API errors.
#[derive(Debug, Serialize, JsonSchema)]
pub struct AppError {
    pub error: String,
    pub error_id: Uuid,
    #[serde(skip)]
    pub status: StatusCode,
    /// Optional Additional error details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<Value>,
}

impl AppError {
    pub fn new(error: &str) -> Self {
        Self {
            error: error.to_string(),
            error_id: Uuid::new_v4(),
            status: StatusCode::BAD_REQUEST,
            error_details: None,
        }
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.error_details = Some(details);
        self
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = self.status;
        let mut res = axum::Json(self).into_response();
        *res.status_mut() = status;
        res
    }
}

#[derive(Debug)]
pub enum Error {
    Connection(String),
    Request(String),
    External(String),
    Parsing(String),
    Subscription(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Connection(msg) => write!(f, "Connection error: {}", msg),
            Error::Request(msg) => write!(f, "Request error: {}", msg),
            Error::External(msg) => write!(f, "External error: {}", msg),
            Error::Parsing(msg) => write!(f, "Parsing error: {}", msg),
            Error::Subscription(msg) => write!(f, "Subscription error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}