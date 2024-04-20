pub trait MerkleHasher<Hash: PartialEq> {
    fn two_to_one(left: &Hash, right: &Hash) -> Hash;
}
pub trait MerkleHasherWithMarkedLeaf<Hash: PartialEq>: MerkleHasher<Hash> {
    fn two_to_one_marked_leaf(left: &Hash, right: &Hash) -> Hash;
}

pub trait MerkleZeroHasher<Hash: PartialEq>: MerkleHasher<Hash> {
    fn get_zero_hash(reverse_level: usize) -> Hash;
}
pub trait MerkleZeroHasherWithMarkedLeaf<Hash: PartialEq>:
    MerkleHasherWithMarkedLeaf<Hash>
{
    fn get_zero_hash_marked(reverse_level: usize) -> Hash;
}

pub trait GeneralMerkleZeroHasher<Hash: PartialEq>:
    MerkleZeroHasher<Hash> + MerkleHasherWithMarkedLeaf<Hash>
{
    fn get_zero_hash_marked_if(reverse_level: usize, marked: bool) -> Hash;
    fn two_to_one_marked_if(left: &Hash, right: &Hash, marked: bool) -> Hash;
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
    MerkleZeroHasherWithMarkedLeaf<Hash> for T
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

impl<Hash: PartialEq + Copy, T: MerkleZeroHasher<Hash> + MerkleZeroHasherWithMarkedLeaf<Hash>>
    GeneralMerkleZeroHasher<Hash> for T
{
    fn get_zero_hash_marked_if(reverse_level: usize, marked: bool) -> Hash {
        if marked {
            T::get_zero_hash_marked(reverse_level)
        } else {
            T::get_zero_hash(reverse_level)
        }
    }

    fn two_to_one_marked_if(left: &Hash, right: &Hash, marked: bool) -> Hash {
        if marked {
            T::two_to_one_marked_leaf(left, right)
        } else {
            T::two_to_one(left, right)
        }
    }
}
