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
use crate::state::AppState;
use super::{models::{VaaRequest, VaaResponse, VaaDoc, ResponsePagination}, client::WormholeClient};
use hex;

pub fn wormhole_routes(state: AppState) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/vaas/{chain_id}/{emitter}",
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

    let client = WormholeClient::new();
    info!("Fetching VAAs for chain {} and emitter {}", params.chain_id, params.emitter);
    
    match client.get_vaas(params.chain_id, &params.emitter).await {
        Ok(bytes) => {
            let raw_response = String::from_utf8_lossy(&bytes);
            // Only log first 200 chars of response at debug level
            debug!("Raw API response (truncated): {}...", &raw_response.chars().take(200).collect::<String>());
            
            if raw_response.contains("error") {
                let error_response: serde_json::Value = serde_json::from_slice(&bytes)
                    .unwrap_or_else(|_| json!({ "error": "Unknown error format" }));
                return (
                    StatusCode::BAD_REQUEST,
                    Json(error_response)
                ).into_response();
            }
            
            match serde_json::from_slice::<VaaResponse>(&bytes) {
                Ok(vaas) => {
                    info!("Successfully retrieved {} VAAs", vaas.data.len());
                    Json(vaas).into_response()
                },
                Err(e) => {
                    error!("Failed to parse VAA response: {}", e);
                    // Move full response to debug level
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
        .tag("wormhole")
        .description("Common chain IDs:\n\
            - 2: Ethereum\n\
            - 4: BSC\n\
            - 6: Avalanche\n\
            - 30: Optimism\n\
            \n\nSee https://docs.wormhole.com/wormhole/reference/constants for all chain IDs\n\n\
            Emitter address format: 32 bytes in hex (64 characters)\n\
            Example: 000000000000000000000000706f82e9bb5b0813501714ab5974216704980e31")
        .response_with::<200, Json<VaaResponse>, _>(|res| {
            res.description("List of VAAs for the given chain and emitter")
                .example(VaaResponse {
                    data: vec![VaaDoc {
                        sequence: 150289,
                        id: "30/000000000000000000000000706f82e9bb5b0813501714ab5974216704980e31/150289".to_string(),
                        version: 1,
                        emitter_chain: 30,
                        emitter_addr: "000000000000000000000000706f82e9bb5b0813501714ab5974216704980e31".to_string(),
                        emitter_native_addr: Some("0x706f82e9bb5b0813501714ab5974216704980e31".to_string()),
                        guardian_set_index: 4,
                        vaa: "base64_encoded_vaa_string".to_string(),
                        timestamp: "2025-01-25T05:03:33Z".to_string(),
                        updated_at: "2025-01-25T05:03:35.839692Z".to_string(),
                        indexed_at: "2025-01-25T05:03:35.839692Z".to_string(),
                        tx_hash: Some("fa9767935a3b9eeb14d4749089c1dfe718aaf09a18e9ad72d927b69ba9783e19".to_string()),
                        digest: Some("ed9bfa74384ea362a3e03f79234f642012d5d4e8cb08f96e2c7fcb62719226d4".to_string()),
                        is_duplicated: Some(false),
                    }],
                    pagination: Some(ResponsePagination {
                        next: None,
                    }),
                })
        })
        .response_with::<500, Json<serde_json::Value>, _>(|res| {
            res.description("Internal server error")
        })
}

/// Validates a Wormhole emitter address
/// 
/// # Arguments
/// * `emitter` - The emitter address as a hex string
/// 
/// # Returns
/// * `Ok(())` if valid
/// * `Err(String)` with error message if invalid
/// 
/// # Example
/// ```
/// let valid = "000000000000000000000000706f82e9bb5b0813501714ab5974216704980e31";
/// assert!(validate_emitter(valid).is_ok());
/// ```
fn validate_emitter(emitter: &str) -> Result<(), String> {
    // Length check: 32 bytes = 64 hex chars
    if emitter.len() != 64 {
        return Err("Invalid emitter address length".to_string());
    }

    // Hex validation
    hex::decode(emitter).map_err(|e| format!("Invalid hex: {}", e))?;
    
    Ok(())
} 