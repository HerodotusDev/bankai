use std::sync::Arc;

use crate::db::manager::DatabaseManager;
use crate::types::job::JobStatus;
use crate::types::proofs::epoch_batch::EpochUpdateBatch;
use crate::types::proofs::epoch_update::{EpochUpdate, G1Point, G2Point};
use crate::types::proofs::sync_committee::SyncCommitteeUpdate;
use crate::utils::config::BankaiConfig;
use bankai_runner::committee_update::{CircuitInput, CircuitOutput, CommitteeUpdateCircuit};
use bankai_runner::epoch_batch::{
    EpochUpdateBatchCircuit, EpochUpdateBatchCircuitInputs, ExpectedEpochUpdateBatchCircuitOutputs,
};
use bankai_runner::epoch_update::{
    BeaconHeaderCircuit, EpochCircuitInputs, EpochUpdateCircuit, ExecutionHeaderCircuitProof,
    ExecutionPayloadHeaderCircuit, ExpectedEpochUpdateCircuitOutputs,
};
use bankai_runner::run_epoch_batch;
use bankai_runner::run_epoch_update;
use bankai_runner::types::{Felt, G1CircuitPoint, G2CircuitPoint, UInt384, Uint256, Uint256Bits32};
pub use bankai_runner::{error::Error as CairoRunnerError, run_committee_update};
use cairo_vm::vm::runners::cairo_pie::CairoPie;
use cairo_vm::Felt252;
use num_bigint::BigUint;
use tracing::info;
use uuid::Uuid;

pub async fn generate_epoch_batch_pie(
    input: EpochUpdateBatch,
    config: &BankaiConfig,
    db_manager: Option<Arc<DatabaseManager>>,
    job_id: Option<Uuid>,
) -> Result<CairoPie, CairoError> {
    let _permit = config
        .pie_generation_semaphore
        .clone()
        .acquire_owned()
        .await
        .unwrap();

    match db_manager {
        None => {}
        Some(db) => {
            let _ = db
                .update_job_status(job_id.unwrap(), JobStatus::StartedTraceGeneration)
                .await;
        }
    }

    info!("Generating trace...");
    let start_time = std::time::Instant::now();

    let pie = run_epoch_batch(config.epoch_batch_circuit_path.as_str(), input.into())?;
    let duration = start_time.elapsed();

    info!("Trace generated successfully in {:.2?}!", duration);

    Ok(pie)
}

pub async fn generate_epoch_update_pie(
    input: EpochUpdate,
    config: &BankaiConfig,
    db_manager: Option<Arc<DatabaseManager>>,
    job_id: Option<Uuid>,
) -> Result<CairoPie, CairoError> {
    let _permit = config
        .pie_generation_semaphore
        .clone()
        .acquire_owned()
        .await
        .unwrap();

    match db_manager {
        None => {}
        Some(db) => {
            let _ = db
                .update_job_status(job_id.unwrap(), JobStatus::StartedTraceGeneration)
                .await;
        }
    }
    info!("Generating trace...");
    let start_time = std::time::Instant::now();

    let pie = run_epoch_update(config.epoch_circuit_path.as_str(), input.into())?;
    let duration = start_time.elapsed();

    info!("Trace generated successfully in {:.2?}!", duration);

    Ok(pie)
}

pub async fn generate_committee_update_pie(
    input: SyncCommitteeUpdate,
    config: &BankaiConfig,
    db_manager: Option<Arc<DatabaseManager>>,
    job_id: Option<Uuid>,
) -> Result<CairoPie, CairoError> {
    let _permit = config
        .pie_generation_semaphore
        .clone()
        .acquire_owned()
        .await
        .unwrap();

    match db_manager {
        None => {}
        Some(db) => {
            let _ = db
                .update_job_status(job_id.unwrap(), JobStatus::StartedTraceGeneration)
                .await;
        }
    }
    info!("Generating trace...");
    let start_time = std::time::Instant::now();

    let pie = run_committee_update(config.committee_circuit_path.as_str(), input.into())?;
    let duration = start_time.elapsed();

    info!("Trace generated successfully in {:.2?}!", duration);

    Ok(pie)
}

impl From<SyncCommitteeUpdate> for CommitteeUpdateCircuit {
    fn from(val: SyncCommitteeUpdate) -> Self {
        let branch = val
            .circuit_inputs
            .next_sync_committee_branch
            .iter()
            .map(|b| Uint256Bits32(BigUint::from_bytes_be(b.as_slice())))
            .collect::<Vec<Uint256Bits32>>();
        let circuit_input = CircuitInput {
            beacon_slot: Felt(Felt252::from(val.circuit_inputs.beacon_slot)),
            next_sync_committee_branch: branch,
            next_aggregate_sync_committee: UInt384(BigUint::from_bytes_be(
                val.circuit_inputs.next_aggregate_sync_committee.as_slice(),
            )),
            committee_keys_root: Uint256Bits32(BigUint::from_bytes_be(
                val.circuit_inputs.committee_keys_root.as_slice(),
            )),
        };
        let circuit_output = CircuitOutput {
            state_root: Uint256(BigUint::from_bytes_be(
                val.expected_circuit_outputs.state_root.as_slice(),
            )),
            slot: Felt(Felt252::from(val.expected_circuit_outputs.slot)),
            committee_hash: Uint256(BigUint::from_bytes_be(
                val.expected_circuit_outputs.committee_hash.as_slice(),
            )),
        };
        CommitteeUpdateCircuit {
            circuit_inputs: circuit_input,
            expected_circuit_outputs: circuit_output,
        }
    }
}

impl From<EpochUpdate> for EpochUpdateCircuit {
    fn from(val: EpochUpdate) -> Self {
        let beacon_header = BeaconHeaderCircuit {
            slot: Uint256(BigUint::from(val.circuit_inputs.header.slot)),
            proposer_index: Uint256(BigUint::from(val.circuit_inputs.header.proposer_index)),
            parent_root: Uint256(BigUint::from_bytes_be(
                val.circuit_inputs.header.parent_root.as_slice(),
            )),
            state_root: Uint256(BigUint::from_bytes_be(
                val.circuit_inputs.header.state_root.as_slice(),
            )),
            body_root: Uint256(BigUint::from_bytes_be(
                val.circuit_inputs.header.body_root.as_slice(),
            )),
        };
        let execution_header_proof: ExecutionHeaderCircuitProof = ExecutionHeaderCircuitProof {
            root: Uint256(BigUint::from_bytes_be(
                val.circuit_inputs.execution_header_proof.root.as_slice(),
            )),
            path: val
                .circuit_inputs
                .execution_header_proof
                .path
                .iter()
                .map(|p| Uint256Bits32(BigUint::from_bytes_be(p.as_slice())))
                .collect::<Vec<Uint256Bits32>>(),
            leaf: Uint256(BigUint::from_bytes_be(
                val.circuit_inputs.execution_header_proof.leaf.as_slice(),
            )),
            index: Felt(Felt252::from(
                val.circuit_inputs.execution_header_proof.index,
            )),
            execution_payload_header: ExecutionPayloadHeaderCircuit(
                val.circuit_inputs
                    .execution_header_proof
                    .execution_payload_header,
            )
            .to_field_roots(),
        };
        let inputs = EpochCircuitInputs {
            header: beacon_header,
            signature_point: val.circuit_inputs.signature_point.into(),
            aggregate_pub: val.circuit_inputs.aggregate_pub.into(),
            non_signers: val
                .circuit_inputs
                .non_signers
                .iter()
                .map(|n| n.clone().into())
                .collect::<Vec<G1CircuitPoint>>(),
            execution_header_proof,
        };
        let expected_outputs = ExpectedEpochUpdateCircuitOutputs {
            beacon_header_root: Uint256(BigUint::from_bytes_be(
                val.expected_circuit_outputs.beacon_header_root.as_slice(),
            )),
            beacon_state_root: Uint256(BigUint::from_bytes_be(
                val.expected_circuit_outputs.beacon_state_root.as_slice(),
            )),
            committee_hash: Uint256(BigUint::from_bytes_be(
                val.expected_circuit_outputs.committee_hash.as_slice(),
            )),
            n_signers: Felt(Felt252::from(val.expected_circuit_outputs.n_signers)),
            slot: Felt(Felt252::from(val.expected_circuit_outputs.slot)),
            execution_header_hash: Uint256(BigUint::from_bytes_be(
                val.expected_circuit_outputs
                    .execution_header_hash
                    .as_slice(),
            )),
            execution_header_height: Felt(Felt252::from(
                val.expected_circuit_outputs.execution_header_height,
            )),
        };
        EpochUpdateCircuit {
            circuit_inputs: inputs,
            expected_circuit_outputs: expected_outputs,
        }
    }
}

impl From<G1Point> for G1CircuitPoint {
    fn from(val: G1Point) -> Self {
        let json = serde_json::to_string(&val).unwrap();
        let parsed: G1CircuitPoint = serde_json::from_str(&json).unwrap();
        parsed
    }
}

impl From<G2Point> for G2CircuitPoint {
    fn from(val: G2Point) -> Self {
        let json = serde_json::to_string(&val).unwrap();
        let parsed: G2CircuitPoint = serde_json::from_str(&json).unwrap();
        parsed
    }
}

impl From<EpochUpdateBatch> for EpochUpdateBatchCircuit {
    fn from(val: EpochUpdateBatch) -> Self {
        let circuit_input = EpochUpdateBatchCircuitInputs {
            committee_hash: Uint256(BigUint::from_bytes_be(
                val.circuit_inputs.committee_hash.as_slice(),
            )),
            epochs: val
                .circuit_inputs
                .epochs
                .into_iter()
                .map(|e| e.into())
                .collect::<Vec<EpochUpdateCircuit>>(),
        };
        let latest_batch_output = circuit_input
            .epochs
            .last()
            .unwrap()
            .expected_circuit_outputs
            .clone();
        let expected_circuit_outputs = ExpectedEpochUpdateBatchCircuitOutputs {
            batch_root: Felt(val.expected_circuit_outputs.batch_root),
            latest_batch_output,
        };
        EpochUpdateBatchCircuit {
            circuit_inputs: circuit_input,
            expected_circuit_outputs,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CairoError {
    #[error("Cairo runner error: {0}")]
    CairoRunner(#[from] CairoRunnerError),
}
