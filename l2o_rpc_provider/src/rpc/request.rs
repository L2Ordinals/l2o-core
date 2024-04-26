use l2o_ord::operation::l2o_a::L2OAHashFunction;
use serde::Deserialize;
use serde::Serialize;

/// Represents the version of the RPC protocol
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Version {
    #[serde(rename = "2.0")]
    V2,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Id {
    String(String),
    Number(i64),
    Null,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "method", content = "params")]
pub enum RequestParams {
    #[serde(rename = "l2o_getLastBlockInscription")]
    L2OGetLastBlockInscription(u64),
    #[serde(rename = "l2o_getDeployInscription")]
    L2OGetDeployInscription(u64),
    #[serde(rename = "l2o_getStateRootAtBlock")]
    L2OGetStateRootAtBlock((u64, u64, L2OAHashFunction)),
    #[serde(rename = "l2o_getMerkleProofStateRootAtBlock")]
    L2OGetMerkleProofStateRootAtBlock((u64, u64, L2OAHashFunction)),
    #[serde(rename = "l2o_getSuperchainStateRootAtBlock")]
    L2OGetSuperchainStateRootAtBlock((u64, L2OAHashFunction)),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RpcRequest {
    /// The version of the protocol
    pub jsonrpc: Version,
    #[serde(flatten)]
    pub request: RequestParams,
    /// The name of the method to execute
    /// The identifier for this request issued by the client,
    /// An [Id] must be a String, null or a number.
    /// If missing it's considered a notification in [Version::V2]
    pub id: Id,
}
