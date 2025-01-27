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

use crate::AppState;
use crate::library::env::get_config;
use super::client::GrpcClient;

pub fn wormhole_routes(state: Arc<AppState>) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/spy/vaas",
            get_with(get_spy_vaas, get_spy_vaas_docs),
        )
        .with_state(state)
}

async fn get_spy_vaas() -> impl IntoApiResponse {
    info!("Starting VAA spy service...");
    
    let spy_addr = match &get_config().wormhole_spy_addr {
        Some(addr) => addr.clone(),
        None => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Wormhole spy address not configured" }))
        ).into_response(),
    };

    let mut client = match GrpcClient::connect(spy_addr).await {
        Ok(client) => client,
        Err(e) => return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to connect to spy: {}", e) }))
        ).into_response(),
    };

    match timeout(
        Duration::from_secs(10),
        client.subscribe_all_vaas()
    ).await {
        Ok(result) => match result {
            Ok(_) => Json(json!({
                "message": "Successfully started VAA stream",
                "note": "Check server logs for all incoming VAAs"
            })).into_response(),
            Err(e) => {
                error!("Failed to subscribe: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Failed to subscribe: {}", e) }))
                ).into_response()
            }
        },
        Err(_) => {
            error!("Timeout waiting for spy response");
            (
                StatusCode::GATEWAY_TIMEOUT,
                Json(json!({ "error": "Timeout waiting for spy response" }))
            ).into_response()
        }
    }
}

fn get_spy_vaas_docs(op: TransformOperation) -> TransformOperation {
    op.description("Get VAAs from Wormhole Spy for a specific chain and emitter")
        .tag("wormhole-spy")
        .response::<200, ()>()
        .response::<400, ()>()
        .response::<500, ()>()
        .response::<504, ()>()
} 