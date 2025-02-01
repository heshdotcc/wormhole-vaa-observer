use std::collections::HashMap;
use std::io::{Cursor, Read};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use byteorder::{BigEndian, ReadBytesExt};
use serde::{Serialize, Deserialize};
use super::super::models::{VaaResponse, VaaMetadata, SequenceGap};
use schemars::JsonSchema;
use aide::OperationOutput;

pub fn analyze_sequences(response: &mut VaaResponse) {
    if response.data.is_empty() {
        return;
    }

    // Sort VAAs by sequence number
    response.data.sort_by_key(|vaa| vaa.sequence);

    let mut sequence_counts: HashMap<u64, usize> = HashMap::new();
    let mut duplicated_sequences = Vec::new();
    let mut sequence_gaps = Vec::new();

    // Ocurrences of each sequence number
    for vaa in &response.data {
        *sequence_counts.entry(vaa.sequence).or_insert(0) += 1;
    }

    // Duplicates
    for (&sequence, &count) in &sequence_counts {
        if count > 1 {
            duplicated_sequences.push(sequence);
        }
    }

    // Gaps
    let lowest_sequence = sequence_counts.keys().min().copied();
    let highest_sequence = sequence_counts.keys().max().copied();

    if let (Some(lowest), Some(highest)) = (lowest_sequence, highest_sequence) {
        let mut current = lowest;
        while current < highest {
            if !sequence_counts.contains_key(&current) {
                let mut gap_end = current + 1;
                while gap_end < highest && !sequence_counts.contains_key(&gap_end) {
                    gap_end += 1;
                }
                
                sequence_gaps.push(SequenceGap {
                    from: current,
                    to: gap_end - 1,
                    size: gap_end - current,
                });
                
                current = gap_end;
            } else {
                current += 1;
            }
        }
    }

    // Totals (before moving the vectors)
    let total_items = response.data.len();
    let total_duplicates = duplicated_sequences.len();
    let total_gaps = sequence_gaps.len();

    response.metadata = VaaMetadata {
        total_items,
        total_duplicates,
        duplicated_sequences,
        lowest_sequence,
        highest_sequence,
        total_gaps,
        sequence_gaps,
    };
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct DecodedVaa {
    // Header
    pub version: u8,
    pub guardian_set_index: u32,
    pub signatures: Vec<GuardianSignature>,
    // Body (Envelope)
    pub timestamp: u32,
    pub nonce: u32,
    pub emitter_chain: u16,
    pub emitter_address: String,
    pub sequence: u64,
    pub consistency_level: u8,
    pub payload: Option<String>,
}

impl OperationOutput for DecodedVaa {
    type Inner = Self;
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct GuardianSignature {
    pub index: u8,
    pub signature: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DecodeVaaRequest {
    pub vaa: String,
}

pub fn decode_vaa(base64_vaa: &str) -> Result<DecodedVaa, Box<dyn std::error::Error>> {
    let vaa_bytes = STANDARD.decode(base64_vaa)?;
    let mut cursor = Cursor::new(&vaa_bytes);

    // 1. Parse Header
    let version = cursor.read_u8()?;
    let guardian_set_index = cursor.read_u32::<BigEndian>()?;
    let num_signatures = cursor.read_u8()?;

    // 2. Parse Signatures
    let mut signatures = Vec::with_capacity(num_signatures as usize);
    for _ in 0..num_signatures {
        let guardian_index = cursor.read_u8()?;
        let mut signature = [0u8; 65];
        cursor.read_exact(&mut signature)?;
        
        signatures.push(GuardianSignature {
            index: guardian_index,
            signature: hex::encode(signature),
        });
    }

    // 3. Parse Body (Envelope)
    let timestamp = cursor.read_u32::<BigEndian>()?;  // 4 bytes per spec
    let nonce = cursor.read_u32::<BigEndian>()?;
    let emitter_chain = cursor.read_u16::<BigEndian>()?;
    
    let mut emitter_address = [0u8; 32];
    cursor.read_exact(&mut emitter_address)?;
    
    let sequence = cursor.read_u64::<BigEndian>()?;
    let consistency_level = cursor.read_u8()?;

    // 4. Get remaining bytes as payload (if any)
    let mut payload = Vec::new();
    cursor.read_to_end(&mut payload)?;
    let payload = if !payload.is_empty() {
        Some(hex::encode(payload))
    } else {
        None
    };

    // Format emitter_address as hex, preserving all 32 bytes
    let emitter_address = format!("0x{}", hex::encode(emitter_address));

    Ok(DecodedVaa {
        version,
        guardian_set_index,
        signatures,
        timestamp,
        nonce,
        emitter_chain,
        emitter_address,
        sequence,
        consistency_level,
        payload,
    })
}
