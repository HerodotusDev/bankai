pub const SLOTS_PER_EPOCH: u64 = 32; // For mainnet
pub const SLOTS_PER_SYNC_COMMITTEE: u64 = 8192; // For mainnet
pub const TARGET_BATCH_SIZE: u64 = 32; // Defines how many epochs in one batch
pub const EPOCHS_PER_SYNC_COMMITTEE: u64 = 256; // For mainnet
pub const MAX_CONCURRENT_JOBS_IN_PROGRESS: u64 = 16; // Define the limit of how many jobs can be in state "in progress" concurrently
pub const MAX_CONCURRENT_PIE_GENERATIONS: usize = 1; // Define how many concurrent trace (pie file) generation jobs are allowed to not exhaust resources
pub const MAX_CONCURRENT_RPC_DATA_FETCH_JOBS: usize = 1; // Define how many data fetching jobs can be performed concurrently to not overload RPC
pub const STARKNET_SEPOLIA: &str = "0x534e5f5345504f4c4941";
pub const STARKNET_MAINNET: &str = "0x534e5f4d41494e";
pub const USE_TRANSACTOR: bool = false;
pub const MAX_JOB_RETRIES_COUNT: u64 = 7;
pub const BEACON_CHAIN_LISTENER_ENABLED: bool = true;
pub const JOBS_RETRY_ENABLED: bool = true;
pub const JOBS_RESUME_ENABLED: bool = true;
pub const JOB_RETRY_DELAY_MS: u64 = 300_0000;
pub const MAX_SKIPPED_SLOTS_RETRY_ATTEMPTS: u64 = 5; // How many skipped slots in row can be on Beacon Chain before we throw error
pub const JOBS_RETRY_CHECK_INTERVAL: u64 = 600; // In seconds
pub const STARKNET_TX_CONFIRMATION_MAX_RETRIES: u64 = 20;
pub const STARKNET_TX_CONFIRMATION_CHECK_DELAY: u64 = 5; // In seconds
