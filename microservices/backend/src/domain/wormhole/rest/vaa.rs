use std::collections::HashMap;
use super::super::models::{VaaResponse, VaaMetadata, SequenceGap};

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
