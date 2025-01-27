use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
    OperationOutput,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use schemars::JsonSchema;
use std::sync::Arc;

use crate::AppState;


#[derive(Serialize, JsonSchema)]
pub struct HealthResponse {
    status: String,
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<HealthDetails>,
}

#[derive(Serialize, JsonSchema)]
pub struct HealthDetails {
    database: bool,
    cache: bool,
    message_queue: bool,
}

impl OperationOutput for HealthResponse {
  type Inner = Self;
}


pub fn health_routes(state: Arc<AppState>) -> ApiRouter {
    ApiRouter::new()
        .api_route_with(
            "/healthz",
            get_with(health_check, health_check_docs),
            |p| p.tag("health"),
        )
        .api_route_with(
            "/livez",
            get_with(liveness_check, liveness_check_docs),
            |p| p.tag("health"),
        )
        .api_route_with(
            "/readyz",
            get_with(readiness_check, readiness_check_docs),
            |p| p.tag("health"),
        )
        .with_state(state)
}

// Health check - checks if the app is healthy
async fn health_check() -> impl IntoApiResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        details: None,
    }).into_response()
}

// Readiness probe - checks if the app can accept traffic
async fn liveness_check() -> impl IntoApiResponse {
    Json(HealthResponse {
        status: "alive".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        details: None,
    }).into_response()
}

// Readiness probe - checks if the app can accept traffic
async fn readiness_check(
    State(state): State<Arc<AppState>>,
) -> impl IntoApiResponse {
    let details = HealthDetails {
        database: check_database(&state).await,
        cache: check_cache(&state).await,
        message_queue: check_message_queue(&state).await,
    };

    let is_ready = details.database && details.cache && details.message_queue;

    let response = HealthResponse {
        status: if is_ready { "ready".to_string() } else { "not ready".to_string() },
        version: env!("CARGO_PKG_VERSION").to_string(),
        details: Some(details),
    };

    if is_ready {
        (StatusCode::OK, Json(response)).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(response)).into_response()
    }
}

async fn check_database(_state: &AppState) -> bool {
    // TODO: Implement actual database check
    true
}

async fn check_cache(_state: &AppState) -> bool {
    // TODO: Implement actual cache check
    true
}

async fn check_message_queue(_state: &Arc<AppState>) -> bool {
    // TODO: Implement queue check
    true
}

fn health_check_docs(op: TransformOperation) -> TransformOperation {
    op.description("Basic health check endpoint")
        .response_with::<200, HealthResponse, _>(|res| {
            res.description("Service is running")
                .example(HealthResponse {
                    status: "healthy".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    details: None,
                })
        })
}

fn liveness_check_docs(op: TransformOperation) -> TransformOperation {
    op.description("Kubernetes liveness probe endpoint")
        .response_with::<200, HealthResponse, _>(|res| {
            res.description("Service is alive and functioning")
                .example(HealthResponse {
                    status: "alive".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    details: None,
                })
        })
}

fn readiness_check_docs(op: TransformOperation) -> TransformOperation {
    op.description("Kubernetes readiness probe endpoint")
        .response_with::<200, HealthResponse, _>(|res| {
            res.description("Service is ready to accept traffic")
                .example(HealthResponse {
                    status: "ready".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    details: Some(HealthDetails {
                        database: true,
                        cache: true,
                        message_queue: true,
                    }),
                })
        })
        .response_with::<503, HealthResponse, _>(|res| {
            res.description("Service is not ready to accept traffic")
        })
} 