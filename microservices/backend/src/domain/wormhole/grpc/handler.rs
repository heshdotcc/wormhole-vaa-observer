use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use serde_json::json;
use tracing::{info, error};
use tokio::time::timeout;
use std::time::Duration;
use uuid::Uuid;

use crate::AppState;
use crate::library::errors::AppError;
use crate::library::config::get_config;
use super::client::GrpcClient;

const DEFAULT_VAA_LIMIT: usize = 50;

pub fn wormhole_routes(state: Arc<AppState>) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/spy/vaas",
            get_with(get_spy_vaas, get_spy_vaas_docs),
        )
        .with_state(state)
}

async fn get_spy_vaas() -> impl IntoApiResponse {
    let result: Result<_, (StatusCode, Json<AppError>)> = async {
        info!("Starting VAA spy service with default limit...");
        
        let spy_addr = get_config().wormhole_spy_addr
            .clone()
            .ok_or_else(|| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AppError {
                        error: "Spy address not configured".to_string(),
                        error_id: Uuid::new_v4(),
                        status: StatusCode::INTERNAL_SERVER_ERROR,
                        error_details: None,
                    })
                )
            })?;

        let mut client = GrpcClient::connect(spy_addr).await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(AppError {
                        error: format!("Failed to connect to spy: {}", e),
                        error_id: Uuid::new_v4(),
                        status: StatusCode::INTERNAL_SERVER_ERROR,
                        error_details: None,
                    })
                )
            })?;

        match timeout(
            Duration::from_secs(get_config().wormhole_spy_timeout),
            client.subscribe_all_vaas(DEFAULT_VAA_LIMIT)
        ).await {
            Ok(result) => match result {
                Ok(count) => Ok(Json(json!({
                    "message": "Successfully processed VAA stream",
                    "processed_vaas": count,
                    "note": "Check server logs for details"
                }))),
                Err(e) => {
                    error!("Failed to subscribe: {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(AppError {
                            error: format!("Failed to subscribe: {}", e),
                            error_id: Uuid::new_v4(),
                            status: StatusCode::INTERNAL_SERVER_ERROR,
                            error_details: None,
                        })
                    ))
                }
            },
            Err(_) => {
                error!("Timeout waiting for spy response");
                Err((
                    StatusCode::GATEWAY_TIMEOUT,
                    Json(AppError {
                        error: "Timeout waiting for spy response".to_string(),
                        error_id: Uuid::new_v4(),
                        status: StatusCode::GATEWAY_TIMEOUT,
                        error_details: None,
                    })
                ))
            }
        }
    }.await;

    match result {
        Ok(json) => json.into_response(),
        Err((status, error)) => (status, error).into_response(),
    }
}

fn get_spy_vaas_docs(op: TransformOperation) -> TransformOperation {
    op.description("Stream VAAs from Wormhole Spy service")
        .tag("wormhole-spy")
        .response::<200, ()>()
        .response::<400, ()>()
        .response::<500, ()>()
        .response::<504, ()>()
} 