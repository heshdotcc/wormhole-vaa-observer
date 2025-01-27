use std::sync::Arc;
use aide::{
    axum::{routing::get_with, ApiRouter, IntoApiResponse},
    transform::TransformOperation,
};
use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use tracing::{info, error, debug};

use crate::AppState;
use super::client::RestClient;
use crate::domain::wormhole::models::{VaaRequest, VaaResponse};


pub fn wormhole_routes(state: Arc<AppState>) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/scan/vaas/{chain_id}/{emitter}",
            get_with(get_vaas, get_vaas_docs),
        )
        .with_state(state)
}

async fn get_vaas(
    Path(params): Path<VaaRequest>,
) -> impl IntoApiResponse {
    // Validate emitter address format
    if params.emitter.len() != 64 {  // 32 bytes in hex = 64 chars
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid emitter address length. Expected 64 hex characters (32 bytes)",
                "example": "000000000000000000000000706f82e9bb5b0813501714ab5974216704980e31"
            }))
        ).into_response();
    }

    // Validate it's a valid hex string
    if hex::decode(&params.emitter).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid emitter address format. Expected hex string",
                "example": "000000000000000000000000706f82e9bb5b0813501714ab5974216704980e31"
            }))
        ).into_response();
    }

    let client = RestClient::new();
    info!("Fetching VAAs for chain {} and emitter {}", params.chain_id, params.emitter);
    
    match client.get_vaas(params.chain_id, &params.emitter).await {
        Ok(bytes) => {
            let raw_response = String::from_utf8_lossy(&bytes);
            debug!("Raw API response (truncated): {}...", &raw_response.chars().take(200).collect::<String>());
            
            match serde_json::from_slice::<VaaResponse>(&bytes) {
                Ok(vaas) => {
                    info!("Successfully retrieved {} VAAs", vaas.data.len());
                    Json(vaas).into_response()
                },
                Err(e) => {
                    error!("Failed to parse VAA response: {}", e);
                    debug!("Raw response: {}", raw_response);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": format!("Failed to parse response: {}", e) }))
                    ).into_response()
                },
            }
        },
        Err(e) => {
            error!("Failed to fetch VAAs: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to fetch VAAs: {}", e) }))
            ).into_response()
        },
    }
}

fn get_vaas_docs(op: TransformOperation) -> TransformOperation {
    op.description("Get VAAs for a specific chain and emitter")
        .tag("wormhole-scan")
        .description("Common chain IDs:\n\
            - 2: Ethereum\n\
            - 4: BSC\n\
            - 6: Avalanche\n\
            - 30: Optimism\n\
            \n\nSee https://docs.wormhole.com/wormhole/reference/constants for all chain IDs\n\n\
            Emitter address format: 32 bytes in hex (64 characters)\n\
            Example: 000000000000000000000000706f82e9bb5b0813501714ab5974216704980e31")
        .response::<200, ()>()
        .response::<400, ()>()
        .response::<500, ()>()
} 