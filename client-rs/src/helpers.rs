use crate::{
    constants::{SLOTS_PER_EPOCH, SLOTS_PER_SYNC_COMMITTEE, TARGET_BATCH_SIZE},
    Error,
};
use tracing::info;

pub fn slot_to_epoch_id(slot: u64) -> u64 {
    slot / SLOTS_PER_EPOCH
}

pub fn slot_to_sync_committee_id(slot: u64) -> u64 {
    slot / SLOTS_PER_SYNC_COMMITTEE
}

pub fn calculate_slots_range_for_batch(first_slot: u64) -> (u64, u64) {
    let start_slot = (u64::try_from(first_slot).unwrap() / 32) * 32 + 32;
    let term = start_slot / 0x2000;
    let mut end_slot = (term + 1) * 0x2000 - 32;

    info!("Slots in Term: Start {}, End {}", start_slot, end_slot);
    let epoch_gap = (end_slot - start_slot) / SLOTS_PER_EPOCH;
    info!(
        "Available Epochs in this Sync Committee period: {}",
        epoch_gap
    );

    // if the gap is smaller then x2 the target size, use the entire gap
    if epoch_gap >= TARGET_BATCH_SIZE * 2 {
        end_slot = start_slot + TARGET_BATCH_SIZE * SLOTS_PER_EPOCH;
    }

    info!("Selected Slots: Start {}, End {}", start_slot, end_slot);
    info!("Epoch Count: {}", (end_slot - start_slot) / SLOTS_PER_EPOCH);

    (start_slot, end_slot)
}

// Computes the slot numbers for term of specified slot
pub async fn calculate_batching_range_for_slot(slot: u64) -> Result<(u64, u64), Error> {
    let next_epoch_slot = (u64::try_from(slot).unwrap() / 32) * 32 + 32;
    let term = next_epoch_slot / 0x2000;
    let terms_last_epoch_slot = (term + 1) * 0x2000 - 32;
    Ok((next_epoch_slot, terms_last_epoch_slot))
}
