// TODO: Scope models for each wormhole client (grpc, rest)

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;

use crate::storage::{ReadModel, HasId};


#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct VaaRequest {
    pub chain_id: u16,
    pub emitter: String,
}

// Updated to match Wormhole API response format
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct VaaResponse {
    pub data: Vec<VaaDoc>,
    pub pagination: Option<ResponsePagination>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ResponsePagination {
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct VaaDoc {
    pub sequence: u64,
    pub id: String,
    pub version: u8,
    #[serde(rename = "emitterChain")]
    pub emitter_chain: u16,
    #[serde(rename = "emitterAddr")]
    pub emitter_addr: String,
    #[serde(rename = "emitterNativeAddr")]
    pub emitter_native_addr: Option<String>,
    #[serde(rename = "guardianSetIndex")]
    pub guardian_set_index: u32,
    pub vaa: String,
    pub timestamp: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "indexedAt")]
    pub indexed_at: String,
    #[serde(rename = "txHash")]
    pub tx_hash: Option<String>,
    pub digest: Option<String>,
    #[serde(rename = "isDuplicated")]
    pub is_duplicated: Option<bool>,
}

impl HasId for VaaRequest {
    fn id(&self) -> Uuid {
        // Generate a deterministic UUID based on chain_id and emitter
        Uuid::new_v5(&Uuid::NAMESPACE_URL, format!("{}-{}", self.chain_id, self.emitter).as_bytes())
    }
}

impl ReadModel for VaaResponse {
    type WriteModel = VaaRequest;

    fn from_write_model(_model: &Self::WriteModel) -> Self { // Leave underscore to supress unused param warning
        Self {
            data: Vec::new(),
            pagination: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaaRecord {
    pub id: Uuid,
    pub hash: String,
    pub bytes: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl HasId for VaaRecord {
    fn id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VaaRecordView {
    pub id: Uuid,
    pub hash: String,
    pub base64_bytes: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ReadModel for VaaRecordView {
    type WriteModel = VaaRecord;

    fn from_write_model(model: &Self::WriteModel) -> Self {
        Self {
            id: model.id,
            hash: model.hash.clone(),
            base64_bytes: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &model.bytes),
            timestamp: model.timestamp,
        }
    }
}