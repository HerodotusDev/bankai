use crate::utils::rpc::BeaconRpcClient;
use crate::Error;
use crate::{
    traits::Submittable,
    utils::{hashing::get_committee_hash, merkle},
};
use alloy_primitives::{FixedBytes, U64};
use beacon_state_proof::state_proof_fetcher::StateProofFetcher;
use beacon_state_proof::state_proof_fetcher::{SyncCommitteeProof, TreeHash};
use bls12_381::G1Affine;
use serde::Serialize;
use sha2::{Digest, Sha256};
use starknet::core::types::Felt;

#[derive(Debug, Serialize)]
pub struct SyncCommitteeUpdate {
    /// The circuit inputs
    pub circuit_inputs: CommitteeCircuitInputs,
    /// The circuit outputs
    pub expected_circuit_outputs: ExpectedCircuitOutputs,
}

impl SyncCommitteeUpdate {
    pub async fn new(client: &BeaconRpcClient, slot: u64) -> Result<SyncCommitteeUpdate, Error> {
        let state_proof_fetcher = StateProofFetcher::new(client.rpc_url.clone());
        let proof = state_proof_fetcher
            .fetch_next_sync_committee_proof(slot)
            .await
            .map_err(|_| Error::FailedFetchingBeaconState)?;

        let circuit_inputs = CommitteeCircuitInputs::from(proof);
        let expected_circuit_outputs = ExpectedCircuitOutputs::from_inputs(&circuit_inputs);

        Ok(SyncCommitteeUpdate {
            circuit_inputs,
            expected_circuit_outputs,
        })
    }
}

/// Represents a proof for updating the sync committee, containing necessary verification data
/// for validating sync committee transitions in the beacon chain.
#[derive(Debug, Serialize)]
pub struct CommitteeCircuitInputs {
    /// The beacon chain slot number for this proof
    pub beacon_slot: U64,
    /// Merkle branch proving inclusion of the next sync committee
    pub next_sync_committee_branch: Vec<FixedBytes<32>>,
    /// The aggregated public key of the next sync committee
    pub next_aggregate_sync_committee: FixedBytes<48>,
    /// Merkle root of the committee's public keys
    pub committee_keys_root: FixedBytes<32>,
}

impl CommitteeCircuitInputs {
    /// Computes the state root by hashing the committee keys root and the aggregate pubkey.
    ///
    /// # Returns
    ///
    /// * `Ok(FixedBytes<32>)` - The computed state root as a 32-byte hash.
    /// * `Err(SyncCommitteeUpdateError)` - If an error occurs during computation.
    pub fn compute_state_root(&self) -> FixedBytes<32> {
        // Pad the 48-byte aggregate pubkey to 64 bytes for hashing
        let mut padded_aggregate = vec![0u8; 64];
        padded_aggregate[..48].copy_from_slice(&self.next_aggregate_sync_committee[..]);
        let aggregate_root: FixedBytes<32> =
            FixedBytes::from_slice(&Sha256::digest(&padded_aggregate));

        // Prepare leaf data by concatenating the committee keys root and aggregate root
        let mut leaf_data = [0u8; 64];
        leaf_data[0..32].copy_from_slice(self.committee_keys_root.as_slice());
        leaf_data[32..64].copy_from_slice(aggregate_root.as_slice());
        let leaf = FixedBytes::from_slice(&Sha256::digest(leaf_data));

        // Compute the state root using the Merkle path

        merkle::hash_merkle_path(self.next_sync_committee_branch.clone(), leaf, 55)
    }
}

#[derive(Debug, Serialize)]
pub struct ExpectedCircuitOutputs {
    /// The state root containing the new sync committee.
    pub state_root: FixedBytes<32>,
    /// The slot containing the state_root
    pub slot: U64,
    /// The hash of the new sync committee
    pub committee_hash: FixedBytes<32>,
}

impl Submittable<CommitteeCircuitInputs> for ExpectedCircuitOutputs {
    fn from_inputs(circuit_inputs: &CommitteeCircuitInputs) -> Self {
        let mut compressed_aggregate_pubkey = [0u8; 48];
        compressed_aggregate_pubkey
            .copy_from_slice(circuit_inputs.next_aggregate_sync_committee.as_slice());
        let committee_hash =
            get_committee_hash(G1Affine::from_compressed(&compressed_aggregate_pubkey).unwrap());
        Self {
            state_root: circuit_inputs.compute_state_root(),
            slot: U64::from(circuit_inputs.beacon_slot),
            committee_hash,
        }
    }

    fn to_calldata(&self) -> Vec<Felt> {
        unimplemented!();
    }

    fn get_contract_selector(&self) -> Felt {
        unimplemented!();
    }
}

impl From<SyncCommitteeProof> for CommitteeCircuitInputs {
    /// Converts a `SyncCommitteeProof` into a `CommitteeCircuitInputs`.
    ///
    /// # Arguments
    ///
    /// * `committee_proof` - The original sync committee proof to convert.
    ///
    /// # Returns
    ///
    /// A new `CommitteeCircuitInputs` instance.
    fn from(committee_proof: SyncCommitteeProof) -> Self {
        let committee_keys_root = &committee_proof.next_sync_committee.pubkeys.tree_hash_root();

        Self {
            beacon_slot: U64::from(committee_proof.slot),
            next_sync_committee_branch: committee_proof
                .proof
                .into_iter()
                .map(|bytes| FixedBytes::from_slice(bytes.as_bytes()))
                .collect(),
            next_aggregate_sync_committee: FixedBytes::from_slice(
                committee_proof
                    .next_sync_committee
                    .aggregate_pubkey
                    .as_serialized(),
            ),
            committee_keys_root: FixedBytes::from_slice(committee_keys_root.as_bytes()),
        }
    }
}
