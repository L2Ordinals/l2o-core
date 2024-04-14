use async_trait::async_trait;
use l2o_common::common::data::hash::Hash256;
use l2o_common::standards::l2o_a::supported_crypto::L2OAHashFunction;
use l2o_crypto::hash::merkle::core::MerkleProofCore;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_crypto::standards::l2o_a::L2ODeployInscriptionV1;
use reqwest::Client;
use serde_json::Value;

use crate::rpc::request::Id;
use crate::rpc::request::RequestParams;
use crate::rpc::request::RpcRequest;
use crate::rpc::request::Version;

pub mod rpc;

#[async_trait]
pub trait L2OAProvider {
    async fn get_last_block_inscription(&self, l2id: u64) -> anyhow::Result<L2OBlockInscriptionV1>;
    async fn get_deploy_inscription(&self, l2id: u64) -> anyhow::Result<L2ODeployInscriptionV1>;
    async fn get_state_root_at_block(
        &self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<Hash256>;
    async fn get_superchainroot_at_block(
        &self,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<Hash256>;
    async fn get_merkle_proof_state_root_at_block(
        &self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<MerkleProofCore<Hash256>>;
}

pub struct Provider {
    url: String,
    client: Client,
}

impl Provider {
    pub fn new(url: String) -> Self {
        Self {
            url: url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl L2OAProvider for Provider {
    async fn get_last_block_inscription(&self, l2id: u64) -> anyhow::Result<L2OBlockInscriptionV1> {
        let response = self
            .client
            .post(&self.url)
            .json(&RpcRequest {
                jsonrpc: Version::V2,
                request: RequestParams::L2OGetLastBlockInscription(l2id),
                id: Id::Number(1),
            })
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(serde_json::from_value::<L2OBlockInscriptionV1>(
            response["result"].clone(),
        )?)
    }

    async fn get_deploy_inscription(&self, l2id: u64) -> anyhow::Result<L2ODeployInscriptionV1> {
        let response = self
            .client
            .post(&self.url)
            .json(&RpcRequest {
                jsonrpc: Version::V2,
                request: RequestParams::L2OGetDeployInscription(l2id),
                id: Id::Number(1),
            })
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(serde_json::from_value::<L2ODeployInscriptionV1>(
            response["result"].clone(),
        )?)
    }

    async fn get_state_root_at_block(
        &self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<Hash256> {
        let response = self
            .client
            .post(&self.url)
            .json(&RpcRequest {
                jsonrpc: Version::V2,
                request: RequestParams::L2OGetStateRootAtBlock((l2id, block_number, hash)),
                id: Id::Number(1),
            })
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(serde_json::from_value::<Hash256>(
            response["result"].clone(),
        )?)
    }

    async fn get_superchainroot_at_block(
        &self,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<Hash256> {
        let response = self
            .client
            .post(&self.url)
            .json(&RpcRequest {
                jsonrpc: Version::V2,
                request: RequestParams::L2OGetSuperchainStateRootAtBlock((block_number, hash)),
                id: Id::Number(1),
            })
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(serde_json::from_value::<Hash256>(
            response["result"].clone(),
        )?)
    }

    async fn get_merkle_proof_state_root_at_block(
        &self,
        l2id: u64,
        block_number: u64,
        hash: L2OAHashFunction,
    ) -> anyhow::Result<MerkleProofCore<Hash256>> {
        let response = self
            .client
            .post(&self.url)
            .json(&RpcRequest {
                jsonrpc: Version::V2,
                request: RequestParams::L2OGetMerkleProofStateRootAtBlock((
                    l2id,
                    block_number,
                    hash,
                )),
                id: Id::Number(1),
            })
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(serde_json::from_value::<MerkleProofCore<Hash256>>(
            response["result"].clone(),
        )?)
    }
}
