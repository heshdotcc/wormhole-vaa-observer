use std::collections::HashSet;
use tracing::{debug, info};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};

use crate::domain::wormhole::grpc::proto::spy::v1::SubscribeSignedVaaResponse;

pub struct VaaProcessor {
    seen_hashes: HashSet<String>,
    count: usize,
    limit: usize,
}

impl VaaProcessor {
    pub fn new(limit: usize) -> Self {
        Self {
            seen_hashes: HashSet::new(),
            count: 0,
            limit,
        }
    }

    pub fn process_vaa(&mut self, vaa: SubscribeSignedVaaResponse) -> bool {
        // Use first 32 bytes as hash for deduplication
        let hash = hex::encode(&vaa.vaa_bytes[0..32]);
        
        if !self.seen_hashes.contains(&hash) {
            self.seen_hashes.insert(hash.clone());
            self.count += 1;
            
            info!(
                "VAA #{}: {} bytes, hash: {}",
                self.count,
                vaa.vaa_bytes.len(),
                hash
            );
            debug!("Full VAA: {}", BASE64_STANDARD.encode(&vaa.vaa_bytes));

            if self.count >= self.limit {
                debug!("Reached VAA limit of {}", self.limit);
                return false;
            }
            // TODO: Add to missing sequence anomaly detection logic
            true  
        } else {
            debug!("Skipping duplicate VAA with hash: {}", hash);
            // TODO: Add to duplicated sequence anomaly detection logic
            true
        }
    }

    pub fn processed_count(&self) -> usize {
        self.count
    }

    pub fn unique_vaas(&self) -> usize {
        self.seen_hashes.len()
    }
} 