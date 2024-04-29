use bitcoin::OutPoint;
use bitcoin::Txid;
use l2o_ord::operation::l2o_a::L2OAHashFunction;
use l2o_ord::script_key::ScriptKey;
use l2o_ord::tick::Tick;
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
    // L2O
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
    // BRC20
    #[serde(rename = "brc20_getTickInfo")]
    BRC20GetTickInfo(Tick),
    #[serde(rename = "brc20_getAllTickInfo")]
    BRC20GetAllTickInfo,
    #[serde(rename = "brc20_getBalanceByAddress")]
    BRC20GetBalanceByAddress((Tick, ScriptKey)),
    #[serde(rename = "brc20_getAllBalanceByAddress")]
    BRC20GetAllBalanceByAddress(ScriptKey),
    #[serde(rename = "brc20_transactionIdToTransactionReceipt")]
    BRC20TransactionIdToTransactionReceipt(Txid),
    #[serde(rename = "brc20_getTickTransferableByAddress")]
    BRC20GetTickTransferableByAddress((Tick, ScriptKey)),
    #[serde(rename = "brc20_getAllTransferableByAddress")]
    BRC20GetAllTransferableByAddress(ScriptKey),
    #[serde(rename = "brc20_transferableAssetsOnOutputWithSatpoints")]
    BRC20TransferableAssetsOnOutputWithSatpoints(OutPoint),

    // BRC21
    #[serde(rename = "brc21_getTickInfo")]
    BRC21GetTickInfo(Tick),
    #[serde(rename = "brc21_getAllTickInfo")]
    BRC21GetAllTickInfo,
    #[serde(rename = "brc21_getBalanceByAddress")]
    BRC21GetBalanceByAddress((Tick, ScriptKey)),
    #[serde(rename = "brc21_getAllBalanceByAddress")]
    BRC21GetAllBalanceByAddress(ScriptKey),
    #[serde(rename = "brc21_transactionIdToTransactionReceipt")]
    BRC21TransactionIdToTransactionReceipt(Txid),
    #[serde(rename = "brc21_getTickTransferableByAddress")]
    BRC21GetTickTransferableByAddress((Tick, ScriptKey)),
    #[serde(rename = "brc21_getAllTransferableByAddress")]
    BRC21GetAllTransferableByAddress(ScriptKey),
    #[serde(rename = "brc21_transferableAssetsOnOutputWithSatpoints")]
    BRC21TransferableAssetsOnOutputWithSatpoints(OutPoint),
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
