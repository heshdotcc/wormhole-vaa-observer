use std::collections::{HashSet, HashMap};
use tracing::{debug, info, error};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::Serialize;
use schemars::JsonSchema;
use crate::library::metrics::record_vaa_metrics;

use crate::domain::wormhole::grpc::proto::spy::v1::SubscribeSignedVaaResponse;

#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct VaaMetadata {
    pub total_processed: usize,
    pub unique_count: usize,
    pub duplicate_count: usize,
    // Potential missing VAAs
    pub sequence_gaps: usize,
    pub duplicated_hashes: Vec<String>,
}

pub struct VaaProcessor {
    seen_hashes: HashSet<String>,
    duplicated_hashes: HashSet<String>, 
    // emitter -> sequence numbers
    sequences: HashMap<String, Vec<u64>>,
    count: usize,
    limit: usize,
    metadata: VaaMetadata,
}

impl VaaProcessor {
    pub fn new(limit: usize) -> Self {
        Self {
            seen_hashes: HashSet::new(),
            duplicated_hashes: HashSet::new(),
            sequences: HashMap::new(),
            count: 0,
            limit,
            metadata: VaaMetadata::default(),
        }
    }

    pub fn process_vaa(&mut self, vaa: SubscribeSignedVaaResponse) -> bool {
        // Stop processing if we've reached the limit
        if self.count >= self.limit {
            return false;
        }

        self.metadata.total_processed += 1;
        // Use first 32 bytes as hash for deduplication
        let hash = hex::encode(&vaa.vaa_bytes[0..32]);
        
        if !self.seen_hashes.contains(&hash) {
            self.seen_hashes.insert(hash.clone());
            self.count += 1;
            self.metadata.unique_count += 1;
            
            // TODO: Extract emitter and sequence from VAA bytes (those are just placeholders)
            let emitter = "default";
            let sequence = self.count as u64;
            
            // Track sequence numbers per emitter
            let sequences = self.sequences
                .entry(emitter.to_string())
                .or_insert_with(Vec::new);
            sequences.push(sequence);
            
            // Check for gaps in sequence
            if sequences.len() > 1 {
                let last = sequences[sequences.len() - 2];
                if sequence > last + 1 {
                    self.metadata.sequence_gaps += 1;
                }
            }

            info!(
                "VAA #{}: {} bytes, hash: {}",
                self.count,
                vaa.vaa_bytes.len(),
                hash
            );
            debug!("Full VAA: {}", BASE64_STANDARD.encode(&vaa.vaa_bytes));

            // TODO: Add to missing sequence anomaly detection logic
            record_vaa_metrics(
                self.metadata.total_processed,
                self.metadata.unique_count,
                self.metadata.duplicate_count,
                self.metadata.sequence_gaps
            );

            true
        } else {
            debug!("Found duplicate VAA with hash: {}", hash);
            self.metadata.duplicate_count += 1;
            self.duplicated_hashes.insert(hash.clone());

            // TODO: Make this more DRY or compat auto-instrumentation
            record_vaa_metrics(
                self.metadata.total_processed,
                self.metadata.unique_count,
                self.metadata.duplicate_count,
                self.metadata.sequence_gaps
            );

            true
        }
    }

    pub fn finalize_metadata(&mut self) {
        // Convert duplicated_hashes to Vec for the response
        self.metadata.duplicated_hashes = self.duplicated_hashes.iter().cloned().collect();
    }

    pub fn processed_count(&self) -> usize {
        self.count
    }

    pub fn get_metadata(&self) -> &VaaMetadata {
        &self.metadata
    }

    // Add a helper method to verify counts
    pub fn verify_counts(&self) -> bool {
        let expected_total = self.metadata.unique_count + self.metadata.duplicate_count;
        if expected_total != self.metadata.total_processed {
            error!(
                "Count mismatch - Total: {}, Unique: {}, Duplicates: {}", 
                self.metadata.total_processed,
                self.metadata.unique_count,
                self.metadata.duplicate_count
            );
            return false;
        }
        true
    }
} 