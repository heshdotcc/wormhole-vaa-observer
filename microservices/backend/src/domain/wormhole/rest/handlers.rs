use std::sync::Arc;
use aide::{
    axum::{routing::{get_with, post_with}, ApiRouter, IntoApiResponse},
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
use serde::Deserialize;

use crate::AppState;
use super::client::RestClient;
use crate::domain::wormhole::models::{
    VaaRequest, 
    VaaResponse, 
    VaaDoc, 
    VaaMetadata,
    ResponsePagination,
};
use super::vaa::{analyze_sequences, decode_vaa, DecodeVaaRequest};

#[derive(Debug, Deserialize)]
struct ExternalVaaResponse {
    data: Vec<VaaDoc>,
    pagination: Option<ResponsePagination>,
}

pub fn wormhole_routes(state: Arc<AppState>) -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/scan/vaas/{chain_id}/{emitter}",
            get_with(get_vaas, get_vaas_docs),
        )
        .api_route(
            "/observer/vaas/decode",
            post_with(decode_vaa_handler, decode_vaa_docs),
        )
        .with_state(state)
}

async fn get_vaas(
    Path(params): Path<VaaRequest>,
) -> impl IntoApiResponse {
    if params.emitter.len() != 64 {  // 32 bytes in hex = 64 chars
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid emitter address length. Expected 64 hex characters (32 bytes)",
            }))
        ).into_response();
    }

    if hex::decode(&params.emitter).is_err() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid emitter address format. Expected hex string",
            }))
        ).into_response();
    }

    let client = RestClient::new();
    info!("Fetching VAAs for chain {} and emitter {}", params.chain_id, params.emitter);
    
    match client.get_vaas(params.chain_id, &params.emitter).await {
        Ok(bytes) => {
            let raw_response = String::from_utf8_lossy(&bytes);
            debug!("Raw API response (truncated): {}...", &raw_response.chars().take(200).collect::<String>());
            
            match serde_json::from_slice::<ExternalVaaResponse>(&bytes) {
                Ok(external_response) => {
                    let total_items = external_response.data.len();
                    info!("Successfully retrieved {} VAAs via REST", total_items);
                    let mut response = VaaResponse {
                        metadata: VaaMetadata {
                            total_items,
                            total_duplicates: 0,
                            duplicated_sequences: Vec::new(),
                            lowest_sequence: None,
                            highest_sequence: None,
                            sequence_gaps: Vec::new(),
                            total_gaps: 0,
                        },
                        data: external_response.data,
                        pagination: external_response.pagination,
                    };
                    
                    analyze_sequences(&mut response);
                    Json(response).into_response()
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

pub async fn decode_vaa_handler(
    Json(request): Json<DecodeVaaRequest>,
) -> impl IntoApiResponse {
    match decode_vaa(&request.vaa) {
        Ok(decoded) => Json(decoded).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Failed to decode VAA: {}", e)
            }))
        ).into_response(),
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

fn decode_vaa_docs(op: TransformOperation) -> TransformOperation {
    op.description("Decode a base64-encoded VAA")
        .tag("wormhole-observer")
        .response::<200, ()>()
        .response::<400, ()>()
        .description("Submit a base64/binary-encoded VAA to decode and analyze its contents.")
}