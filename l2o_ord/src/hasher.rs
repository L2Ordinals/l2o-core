use l2o_common::common::data::hash::Hash256;
use l2o_crypto::fields::goldilocks::hash::hash256_to_goldilocks_hash;
use l2o_crypto::fields::goldilocks::hash::hash256_to_goldilocks_u32;
use l2o_crypto::hash::hash_functions::blake3;
use l2o_crypto::hash::hash_functions::blake3::Blake3Hasher;
use l2o_crypto::hash::hash_functions::keccak256;
use l2o_crypto::hash::hash_functions::keccak256::Keccak256Hasher;
use l2o_crypto::hash::hash_functions::poseidon_goldilocks::PoseidonHasher;
use l2o_crypto::hash::hash_functions::sha256;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::traits::L2OHash;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::HashOut;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::Hasher;

use crate::operation::l2o_a::L2OABlockV1;

pub trait L2OBlockHasher {
    fn get_l2_block_hash(block: &L2OABlockV1) -> Hash256;
}

impl L2OBlockHasher for Blake3Hasher {
    fn get_l2_block_hash(block: &L2OABlockV1) -> Hash256 {
        let payload = get_block_payload_bytes(block);
        blake3::hash(&payload)
    }
}

impl L2OBlockHasher for Keccak256Hasher {
    fn get_l2_block_hash(block: &L2OABlockV1) -> Hash256 {
        let payload = get_block_payload_bytes(block);
        keccak256::hash(&payload)
    }
}

impl L2OBlockHasher for Sha256Hasher {
    fn get_l2_block_hash(block: &L2OABlockV1) -> Hash256 {
        let payload = get_block_payload_bytes(block);
        sha256::hash(&payload)
    }
}

impl L2OBlockHasher for PoseidonHasher {
    fn get_l2_block_hash(block: &L2OABlockV1) -> Hash256 {
        let payload_a = HashOut {
            elements: [
                GoldilocksField::from_canonical_u64(block.l2id),
                GoldilocksField::from_noncanonical_u64(block.l2_block_number),
                GoldilocksField::from_noncanonical_u64(block.bitcoin_block_number),
                GoldilocksField::ZERO,
            ],
        };
        let payload_b =
            PoseidonHash::hash_no_pad(&hash256_to_goldilocks_u32(&block.bitcoin_block_hash));
        let payload_c =
            PoseidonHash::hash_no_pad(&hash256_to_goldilocks_u32(&Hash256(block.public_key.0)));

        let payload_d = PoseidonHash::two_to_one(
            PoseidonHash::two_to_one(
                PoseidonHash::two_to_one(
                    hash256_to_goldilocks_hash(&block.start_state_root),
                    hash256_to_goldilocks_hash(&block.end_state_root),
                ),
                PoseidonHash::two_to_one(
                    hash256_to_goldilocks_hash(&block.start_withdrawal_state_root),
                    hash256_to_goldilocks_hash(&block.end_withdrawal_state_root),
                ),
            ),
            PoseidonHash::two_to_one(
                hash256_to_goldilocks_hash(&block.deposit_state_root),
                hash256_to_goldilocks_hash(&block.superchain_root),
            ),
        );

        let final_gl = PoseidonHash::two_to_one(
            PoseidonHash::two_to_one(payload_a, payload_b),
            PoseidonHash::two_to_one(payload_c, payload_d),
        );
        final_gl.to_hash_256()
    }
}

pub fn get_block_payload_goldilocks_hash_u32_mode(block: &L2OABlockV1) -> Vec<GoldilocksField> {
    let mut payload_bytes: Vec<GoldilocksField> = Vec::new();
    payload_bytes.push(GoldilocksField::from_canonical_u64(block.l2id));
    payload_bytes.push(GoldilocksField::from_noncanonical_u64(
        block.l2_block_number,
    ));
    payload_bytes.push(GoldilocksField::from_noncanonical_u64(
        block.bitcoin_block_number,
    ));

    payload_bytes.extend_from_slice(&hash256_to_goldilocks_u32(&block.bitcoin_block_hash));
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

pub fn get_block_payload_bytes(block: &L2OABlockV1) -> Vec<u8> {
    let mut payload_bytes: Vec<u8> = Vec::new();
    payload_bytes.extend_from_slice(&block.l2id.to_le_bytes());
    payload_bytes.extend_from_slice(&block.l2_block_number.to_le_bytes());
    payload_bytes.extend_from_slice(&block.bitcoin_block_number.to_le_bytes());
    payload_bytes.extend_from_slice(&block.bitcoin_block_hash.0);
    payload_bytes.extend_from_slice(&block.public_key.0);

    payload_bytes.extend_from_slice(&block.start_state_root.0);
    payload_bytes.extend_from_slice(&block.end_state_root.0);

    payload_bytes.extend_from_slice(&block.deposit_state_root.0);
    payload_bytes.extend_from_slice(&block.start_withdrawal_state_root.0);
    payload_bytes.extend_from_slice(&block.end_withdrawal_state_root.0);

    payload_bytes.extend_from_slice(&block.superchain_root.0);

    payload_bytes
}
