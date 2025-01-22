pub const SLOTS_PER_EPOCH: u64 = 32; // For mainnet
pub const SLOTS_PER_SYNC_COMMITTEE: u64 = 8192; // For mainnet
pub const TARGET_BATCH_SIZE: u64 = 32; // Defines how many epochs in one batch
pub const EPOCHS_PER_SYNC_COMMITTEE: u64 = 256; // For mainnet
pub const MAX_CONCURRENT_JOBS_IN_PROGRESS: u64 = 16; // Define the limit of how many jobs can be in state "in progress" concurrently
pub const MAX_CONCURRENT_PIE_GENERATIONS: usize = 3; // Define how many concurrent trace (pie file) generation jobs are allowed to not exhaust resources
pub const MAX_CONCURRENT_RPC_DATA_FETCH_JOBS: usize = 4; // Define how many data fetching jobs can be performed concurrently to not overload RPC
