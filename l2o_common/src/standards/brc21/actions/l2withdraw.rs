use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound = "MerkleProof: Serialize, for<'de2> MerkleProof: Deserialize<'de2>")]
pub struct BRC21L2Withdraw<MerkleProof>
where
    MerkleProof: Serialize,
    for<'de2> MerkleProof: Deserialize<'de2>,
{
    pub tick: String,
    pub to: String,
    pub amt: String,
    pub proof: MerkleProof,
}

impl<MerkleProof> BRC21L2Withdraw<MerkleProof>
where
    MerkleProof: Serialize,
    for<'de2> MerkleProof: Deserialize<'de2>,
{
    pub fn new(tick: String, to: String, amt: String, proof: MerkleProof) -> Self {
        BRC21L2Withdraw {
            tick,
            to,
            amt,
            proof,
        }
    }
}
