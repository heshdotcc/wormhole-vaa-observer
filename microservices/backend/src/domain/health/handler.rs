use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::{
    extract::State,
    Json,
    http::StatusCode,
    response::IntoResponse,
};
use serde::Serialize;
use schemars::JsonSchema;
use crate::state::AppState;

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

pub fn health_routes(state: AppState) -> ApiRouter {
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

// Basic health check - just returns if the service is running
async fn health_check() -> impl IntoApiResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        details: None,
    })
}

// Liveness probe - checks if the application is running and not deadlocked
async fn liveness_check() -> impl IntoApiResponse {
    // Here you could check:
    // - Memory usage
    // - CPU usage
    // - Thread deadlocks
    // - Critical background tasks
    
    // For now, just return healthy if we can respond
    Json(HealthResponse {
        status: "alive".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        details: None,
    })
}

// Readiness probe - checks if the app can accept traffic
async fn readiness_check(
    State(state): State<AppState>,
) -> impl IntoApiResponse {
    // Here you would check all external dependencies:
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
    // Implement actual database check
    true
}

async fn check_cache(_state: &AppState) -> bool {
    // Implement actual cache check
    true
}

async fn check_message_queue(_state: &AppState) -> bool {
    // Implement actual message queue check
    true
}

fn health_check_docs(op: TransformOperation) -> TransformOperation {
    op.description("Basic health check endpoint")
        .response_with::<200, Json<HealthResponse>, _>(|res| {
            res.description("Service is running")
                .example(HealthResponse {
                    status: "healthy".to_string(),
                    version: "0.1.0".to_string(),
                    details: None,
                })
        })
}

fn liveness_check_docs(op: TransformOperation) -> TransformOperation {
    op.description("Kubernetes liveness probe endpoint")
        .response_with::<200, Json<HealthResponse>, _>(|res| {
            res.description("Service is alive and functioning")
                .example(HealthResponse {
                    status: "alive".to_string(),
                    version: "0.1.0".to_string(),
                    details: None,
                })
        })
}

fn readiness_check_docs(op: TransformOperation) -> TransformOperation {
    op.description("Kubernetes readiness probe endpoint")
        .response_with::<200, Json<HealthResponse>, _>(|res| {
            res.description("Service is ready to accept traffic")
                .example(HealthResponse {
                    status: "ready".to_string(),
                    version: "0.1.0".to_string(),
                    details: Some(HealthDetails {
                        database: true,
                        cache: true,
                        message_queue: true,
                    }),
                })
        })
        .response_with::<503, Json<HealthResponse>, _>(|res| {
            res.description("Service is not ready to accept traffic")
        })
} 