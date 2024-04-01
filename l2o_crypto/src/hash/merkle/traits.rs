pub trait MerkleHasher<Hash: PartialEq> {
    fn two_to_one(left: &Hash, right: &Hash) -> Hash;
}
pub trait MerkleHasherWithMarkedLeaf<Hash: PartialEq>: MerkleHasher<Hash> {
    fn two_to_one_marked_leaf(left: &Hash, right: &Hash) -> Hash;
}

pub trait MerkleZeroHasher<Hash: PartialEq>: MerkleHasher<Hash> {
    fn get_zero_hash(reverse_level: usize) -> Hash;
}
pub trait BaseMerkleZeroHasherWithMarkedLeaf<Hash: PartialEq>:
    MerkleHasherWithMarkedLeaf<Hash>
{
    fn get_zero_hash_marked(reverse_level: usize) -> Hash;
}
pub trait MerkleZeroHasherWithMarkedLeaf<Hash: PartialEq>:
    BaseMerkleZeroHasherWithMarkedLeaf<Hash> + MerkleZeroHasher<Hash>
{
}

pub const ZERO_HASH_CACHE_SIZE: usize = 128;
pub trait MerkleZeroHasherWithCache<Hash: PartialEq + Copy>: MerkleHasher<Hash> {
    const CACHED_ZERO_HASHES: [Hash; ZERO_HASH_CACHE_SIZE];
}
pub trait MerkleZeroHasherWithCacheMarkedLeaf<Hash: PartialEq + Copy>:
    MerkleHasherWithMarkedLeaf<Hash>
{
    const CACHED_MARKED_LEAF_ZERO_HASHES: [Hash; ZERO_HASH_CACHE_SIZE];
}

fn iterate_merkle_hasher<Hash: PartialEq, Hasher: MerkleHasher<Hash>>(
    mut current: Hash,
    reverse_level: usize,
) -> Hash {
    for _ in 0..reverse_level {
        current = Hasher::two_to_one(&current, &current);
    }
    current
}
impl<Hash: PartialEq + Copy, T: MerkleZeroHasherWithCache<Hash>> MerkleZeroHasher<Hash> for T {
    fn get_zero_hash(reverse_level: usize) -> Hash {
        if reverse_level < ZERO_HASH_CACHE_SIZE {
            T::CACHED_ZERO_HASHES[reverse_level]
        } else {
            let current = T::CACHED_ZERO_HASHES[ZERO_HASH_CACHE_SIZE - 1];
            iterate_merkle_hasher::<Hash, Self>(current, reverse_level - ZERO_HASH_CACHE_SIZE + 1)
        }
    }
}

impl<Hash: PartialEq + Copy, T: MerkleZeroHasherWithCacheMarkedLeaf<Hash>>
    BaseMerkleZeroHasherWithMarkedLeaf<Hash> for T
{
    fn get_zero_hash_marked(reverse_level: usize) -> Hash {
        if reverse_level < ZERO_HASH_CACHE_SIZE {
            T::CACHED_MARKED_LEAF_ZERO_HASHES[reverse_level]
        } else {
            let current = T::CACHED_MARKED_LEAF_ZERO_HASHES[ZERO_HASH_CACHE_SIZE - 1];
            iterate_merkle_hasher::<Hash, Self>(current, reverse_level - ZERO_HASH_CACHE_SIZE + 1)
        }
    }
}

impl<
        Hash: PartialEq + Copy,
        T: MerkleZeroHasherWithCacheMarkedLeaf<Hash> + MerkleZeroHasherWithCache<Hash>,
    > MerkleZeroHasherWithMarkedLeaf<Hash> for T
{
}
