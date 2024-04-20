use l2o_common::common::data::hash::Hash256;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::HashOut;
use plonky2::hash::hash_types::RichField;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::config::Hasher;

use crate::fields::goldilocks::hash::hash256_to_goldilocks_hash;
use crate::fields::goldilocks::hash::hash256_to_goldilocks_u32;
use crate::hash::merkle::traits::MerkleHasher;
use crate::hash::merkle::traits::MerkleHasherWithMarkedLeaf;
use crate::hash::traits::L2OBlockHasher;
use crate::hash::traits::L2OHash;
use crate::standards::l2o_a::L2OABlockInscriptionV1;

pub struct PoseidonGoldilocksHasher;

pub struct PoseidonHasher;

impl<F: RichField> MerkleHasher<HashOut<F>> for PoseidonHasher {
    fn two_to_one(left: &HashOut<F>, right: &HashOut<F>) -> HashOut<F> {
        PoseidonHash::two_to_one(*left, *right)
    }
}
impl<F: RichField> MerkleHasherWithMarkedLeaf<HashOut<F>> for PoseidonHasher {
    fn two_to_one_marked_leaf(left: &HashOut<F>, right: &HashOut<F>) -> HashOut<F> {
        PoseidonHash::hash_no_pad(&[
            left.elements[0],
            left.elements[1],
            left.elements[2],
            left.elements[3],
            right.elements[0],
            right.elements[1],
            right.elements[2],
            right.elements[3],
            F::ONE,
        ])
    }
}

impl L2OBlockHasher for PoseidonHasher {
    fn get_l2_block_hash(block: &L2OABlockInscriptionV1) -> Hash256 {
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

pub fn get_block_payload_goldilocks_hash_u32_mode(
    block: &L2OABlockInscriptionV1,
) -> Vec<GoldilocksField> {
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
