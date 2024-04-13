use l2o_common::common::data::hash::Hash256;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;

use crate::fields::goldilocks::hash::hash256_to_goldilocks_u32;
use crate::standards::l2o_a::L2OBlockInscriptionV1;

pub fn get_block_payload_bytes(block: &L2OBlockInscriptionV1) -> Vec<u8> {
    let mut payload_bytes: Vec<u8> = Vec::new();
    payload_bytes.extend_from_slice(&block.l2id.to_le_bytes());
    payload_bytes.extend_from_slice(&block.l2_block_number.to_le_bytes());
    // payload_bytes.extend_from_slice(&block.bitcoin_block_number.to_le_bytes());
    // payload_bytes.extend_from_slice(&block.bitcoin_block_hash.0);
    payload_bytes.extend_from_slice(&block.public_key.0);

    payload_bytes.extend_from_slice(&block.start_state_root.0);
    payload_bytes.extend_from_slice(&block.end_state_root.0);

    payload_bytes.extend_from_slice(&block.deposit_state_root.0);
    payload_bytes.extend_from_slice(&block.start_withdrawal_state_root.0);
    payload_bytes.extend_from_slice(&block.end_withdrawal_state_root.0);

    payload_bytes.extend_from_slice(&block.superchain_root.0);

    payload_bytes
}

pub fn get_block_payload_goldilocks_hash_u32_mode(
    block: &L2OBlockInscriptionV1,
) -> Vec<GoldilocksField> {
    let mut payload_bytes: Vec<GoldilocksField> = Vec::new();
    payload_bytes.push(GoldilocksField::from_canonical_u64(block.l2id));
    payload_bytes.push(GoldilocksField::from_noncanonical_u64(
        block.l2_block_number,
    ));
    // payload_bytes.push(GoldilocksField::from_noncanonical_u64(
    //     block.bitcoin_block_number,
    // ));

    // payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(&block.
    // bitcoin_block_hash));
    payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(&Hash256(block.public_key.0)));
    payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(&block.start_state_root));
    payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(&block.end_state_root));
    payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(&block.deposit_state_root));
    payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(
        &block.start_withdrawal_state_root,
    ));
    payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(&block.end_withdrawal_state_root));
    payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(&block.superchain_root));

    payload_bytes
}
