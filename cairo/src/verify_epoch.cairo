from starkware.cairo.common.cairo_builtins import PoseidonBuiltin, ModBuiltin, BitwiseBuiltin
from starkware.cairo.common.registers import get_fp_and_pc
from starkware.cairo.common.alloc import alloc
from starkware.cairo.common.uint256 import Uint256
from definitions import bn, bls, UInt384, one_E12D, N_LIMBS, BASE, E12D, G1Point, G2Point, G1G2Pair
from bls12_381.multi_pairing_2 import multi_pairing_2P
from hash_to_curve import hash_to_curve
from cairo.src.ssz import SSZ, MerkleTree, MerkleUtils
from cairo.src.constants import g1_negative
from cairo.src.domain import Domain
from cairo.src.signer import (
    fast_aggregate_signer_pubs,
    aggregate_signer_pubs,
    faster_fast_aggregate_signer_pubs,
)
from cairo.src.utils import pow2alloc128
from sha import SHA256

func run_epoch_update{
    output_ptr: felt*,
    range_check_ptr,
    bitwise_ptr: BitwiseBuiltin*,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
    pow2_array: felt*,
    sha256_ptr: felt*,
}() {
    alloc_locals;

    local sig_point: G2Point;
    local slot: felt;
    let (execution_path: felt**) = alloc();
    local execution_path_len: felt;

    %{
        from cairo.py.utils import write_g2, write_g1g2, write_g1, print_g2, int_to_uint256, hex_to_chunks_32
        write_g2(ids.sig_point, program_input["circuit_inputs"]["signature_point"])
        ids.slot = program_input["circuit_inputs"]["header"]["slot"]

        execution_path = [hex_to_chunks_32(node) for node in program_input["circuit_inputs"]["execution_header_proof"]["path"]]
        ids.execution_path_len = len(execution_path)
        segments.write_arg(ids.execution_path, execution_path)
    %}
    // %{ print("Running Verification for Slot: ", ids.slot) %}

    let (header_root, body_root, state_root) = hash_header();
    
    // %{ print("HeaderRoot: ", hex(ids.header_root.high * 2**128 + ids.header_root.low)) %}

    let signing_root = Domain.compute_signing_root(header_root, slot);

    // %{ print("SigningRoot: ", hex(ids.signing_root.high * 2**128 + ids.signing_root.low)) %}

    let (msg_point) = hash_to_curve(1, signing_root);
    

    // %{ print_g2("MsgPoint", ids.msg_point) %}

    let (committee_hash, agg_key, n_non_signers) = faster_fast_aggregate_signer_pubs();
    
    let n_signers = 512 - n_non_signers;
    verify_signature(agg_key, msg_point, sig_point);

    // Verify Execution Header
    let (execution_root, execution_hash, execution_height) = SSZ.hash_execution_payload_header_root();
    let root_felts = MerkleUtils.chunk_uint256(execution_root);
    let computed_body_root = MerkleTree.hash_merkle_path(
        path=execution_path, path_len=4, leaf=root_felts, index=9
    );

    %{
        print("execution header hash", hex(ids.execution_hash.low), hex(ids.execution_hash.high))
    %}

    assert computed_body_root.low = body_root.low;
    assert computed_body_root.high = body_root.high;


    %{
        from cairo.py.utils import uint256_to_int
        assert uint256_to_int(ids.header_root) == int(program_input["expected_circuit_outputs"]["beacon_header_root"], 16), "Header Root Mismatch"
        assert uint256_to_int(ids.state_root) == int(program_input["expected_circuit_outputs"]["beacon_state_root"], 16), "State Root Mismatch"
        assert uint256_to_int(ids.committee_hash) == int(program_input["expected_circuit_outputs"]["committee_hash"], 16), "Committee Hash Mismatch"
        assert ids.n_signers == program_input["expected_circuit_outputs"]["n_signers"], "Number of Signers Mismatch"
        assert ids.slot == program_input["expected_circuit_outputs"]["slot"], "Slot Mismatch"
        assert uint256_to_int(ids.execution_hash) == int(program_input["expected_circuit_outputs"]["execution_header_hash"], 16), "Execution Header Hash Mismatch"
        assert ids.execution_height == program_input["expected_circuit_outputs"]["execution_header_height"], "Execution Header Height Mismatch"
    %}

    assert [output_ptr] = header_root.low;
    assert [output_ptr + 1] = header_root.high;
    assert [output_ptr + 2] = state_root.low;
    assert [output_ptr + 3] = state_root.high;
    assert [output_ptr + 4] = slot;
    assert [output_ptr + 5] = committee_hash.low;
    assert [output_ptr + 6] = committee_hash.high;
    assert [output_ptr + 7] = n_signers;
    assert [output_ptr + 8] = execution_hash.low;
    assert [output_ptr + 9] = execution_hash.high;
    assert [output_ptr + 10] = execution_height;
    let output_ptr = output_ptr + 11;

    return ();
}

func hash_header{
    range_check_ptr, bitwise_ptr: BitwiseBuiltin*, pow2_array: felt*, sha256_ptr: felt*
}() -> (header_root: Uint256, body_root: Uint256, state_root: Uint256) {
    alloc_locals;

    local slot: Uint256;
    local proposer_index: Uint256;
    local parent_root: Uint256;
    local state_root: Uint256;
    local body_root: Uint256;
    %{
        from cairo.py.utils import split_uint256
        ids.slot.low = program_input["circuit_inputs"]["header"]["slot"]
        ids.slot.high = 0

        ids.proposer_index.low = program_input["circuit_inputs"]["header"]["proposer_index"]
        ids.proposer_index.high = 0

        parent_root = split_uint256(int(program_input["circuit_inputs"]["header"]["parent_root"], 16))
        ids.parent_root.low, ids.parent_root.high = parent_root

        state_root = split_uint256(int(program_input["circuit_inputs"]["header"]["state_root"], 16))
        ids.state_root.low, ids.state_root.high = state_root

        body_root = split_uint256(int(program_input["circuit_inputs"]["header"]["body_root"], 16))
        ids.body_root.low, ids.body_root.high = body_root
    %}

    let header_root = SSZ.hash_header_root(
        slot, proposer_index, parent_root, state_root, body_root
    );

    return (header_root=header_root, body_root=body_root, state_root=state_root);
}

func verify_signature{
    range_check_ptr,
    poseidon_ptr: PoseidonBuiltin*,
    range_check96_ptr: felt*,
    add_mod_ptr: ModBuiltin*,
    mul_mod_ptr: ModBuiltin*,
}(agg_pub: G1Point, msg_point: G2Point, sig_point: G2Point) {
    let neg_g1: G1Point = g1_negative();
    let g1_sig_pair: G1G2Pair = G1G2Pair(P=neg_g1, Q=sig_point);
    let pk_msg_pair: G1G2Pair = G1G2Pair(P=agg_pub, Q=msg_point);

    let (inputs: G1G2Pair*) = alloc();
    assert inputs[0] = g1_sig_pair;
    assert inputs[1] = pk_msg_pair;

    // We check the pairs are on the curve in the pairing function
    let (res) = multi_pairing_2P(inputs);
    let (one) = one_E12D();
    assert res = one;
    return ();
}
