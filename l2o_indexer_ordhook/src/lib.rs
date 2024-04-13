use std::net::SocketAddr;
use std::sync::Arc;

use ark_bn254::Bn254;
use ark_bn254::Fr;
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use bytes::Buf;
use bytes::Bytes;
use chainhook_sdk::types::BitcoinBlockMetadata;
use chainhook_sdk::types::BlockIdentifier;
use chainhook_sdk::types::OrdinalOperation;
use chainhook_sdk::types::TransactionIdentifier;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Incoming as IncomingBody;
use hyper::header;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Method;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use l2o::inscription::L2OInscription;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::hash::L2OHash;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;
use l2o_common::standards::l2o_a::actions::deploy::L2ODeployInscription;
use l2o_common::standards::l2o_a::supported_crypto::L2OAProofType;
use l2o_common::IndexerOrdHookArgs;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::traits::L2OBlockHasher;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use l2o_crypto::proof::groth16::bn128::verifier_data::Groth16BN128VerifierData;
use l2o_crypto::signature::schnorr::verify_sig;
use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_store::core::store::L2OStoreV1Core;
use l2o_store::core::traits::L2OStoreV1;
use l2o_store_rocksdb::KVQRocksDBStore;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::rpc::request::RequestParams;
use crate::rpc::request::RpcRequest;
use crate::rpc::response::ResponseResult;
use crate::rpc::response::RpcResponse;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub mod l2o;
pub mod rpc;
pub mod store;

static NOTFOUND: &[u8] = b"Not Found";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BitcoinTransactionDataV2 {
    pub transaction_identifier: TransactionIdentifier,
    /// Transactions that are related to other transactions should include the
    /// transaction_identifier of these transactions in the metadata.
    pub metadata: BitcoinTransactionMetadataV2,
}

/// Extra data for Transaction
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BitcoinTransactionMetadataV2 {
    pub ordinal_operations: Vec<OrdinalOperation>,
    pub proof: Option<String>,
}

/// BitcoinBlock contain an array of Transactions that occurred at a particular
/// BlockIdentifier. A hard requirement for blocks returned by Rosetta
/// implementations is that they MUST be _inalterable_: once a client has
/// requested and received a block identified by a specific BlockIndentifier,
/// all future calls for that same BlockIdentifier must return the same block
/// contents.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BitcoinBlockDataV2 {
    pub block_identifier: BlockIdentifier,
    pub parent_block_identifier: BlockIdentifier,
    /// The timestamp of the block in milliseconds since the Unix Epoch. The
    /// timestamp is stored in milliseconds because some blockchains produce
    /// blocks more often than once a second.
    pub timestamp: u32,
    pub transactions: Vec<BitcoinTransactionDataV2>,
    pub metadata: BitcoinBlockMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinChainhookOccurrencePayloadV2 {
    pub apply: Vec<BitcoinBlockDataV2>,
    pub rollback: Vec<BitcoinBlockDataV2>,
}

async fn process_l2o_inscription(
    store: Arc<Mutex<L2OStoreV1Core<KVQRocksDBStore>>>,
    bitcoin_block: &BitcoinBlockDataV2,
    _bitcoin_tx: &BitcoinTransactionDataV2,
    inscription: L2OInscription,
) -> anyhow::Result<()> {
    match inscription {
        L2OInscription::Deploy(deploy) => {
            let l2id: u64 = deploy.l2id.into();
            if store.lock().await.has_deployed_l2id(l2id)? {
                tracing::debug!("l2o {} already deployed", l2id);
                return Ok(());
            }
            let proof_type: L2OAProofType = deploy.proof_type.parse()?;
            let verifier_data = if proof_type.is_groth_16_bn_128() {
                deploy
                    .vk
                    .try_as_groth_16_verifier_serializable()
                    .ok_or(anyhow::anyhow!("marformed verifier"))?
                    .to_vk()?
            } else {
                anyhow::bail!("unsupported verifier type");
            };
            let deploy_inscription = L2ODeployInscription {
                p: "l2o-a".to_string(),
                op: "Deploy".to_string(),
                l2id,
                public_key: L2OCompactPublicKey::from_hex(&deploy.public_key)?,
                start_state_root: Hash256::from_hex(&deploy.start_state_root)?,
                hash_function: deploy.hash_function.parse()?,
                proof_type,
                verifier_data: Groth16BN128VerifierData(verifier_data).into(),
            };
            store
                .lock()
                .await
                .report_deploy_inscription(deploy_inscription)?;
            tracing::info!("deployed l2o: {}", deploy.l2id);
            Ok(())
        }
        L2OInscription::Block(block) => {
            let l2id: u64 = block.l2id.into();
            if !store.lock().await.has_deployed_l2id(l2id)? {
                tracing::debug!("l2o {} not deployed yet", l2id);
                return Ok(());
            }

            let deploy = store.lock().await.get_deploy_inscription(l2id)?;

            let block_proof = if deploy.proof_type.is_groth_16_bn_128() {
                block
                    .proof
                    .try_as_groth_16_proof_serializable()
                    .ok_or(anyhow::anyhow!("marformed proof"))?
                    .to_proof_with_public_inputs_groth16_bn254()?
            } else {
                anyhow::bail!("unsupported proof type");
            };

            let (start_state_root, end_state_root, start_withdrawal_state_root, public_key) =
                if let Ok(last_block) = store.lock().await.get_last_block_inscription(l2id) {
                    if u64::from(block.block_parameters.block_number)
                        != last_block.l2_block_number + 1
                    {
                        anyhow::bail!("block must be consecutive");
                    }

                    if bitcoin_block.block_identifier.index <= last_block.bitcoin_block_number {
                        anyhow::bail!("bitcoin block must be bigger than previous");
                    }

                    (
                        last_block.end_state_root,
                        Hash256::from_hex(&block.block_parameters.state_root)?,
                        last_block.end_withdrawal_state_root,
                        last_block.public_key,
                    )
                } else {
                    if block.block_parameters.block_number != 0 {
                        anyhow::bail!("genesis block number must be zero");
                    }
                    if block.block_parameters.state_root != deploy.start_state_root.to_hex() {
                        anyhow::bail!(
                            "genesis block state root must be equal to deploy start state root"
                        );
                    }

                    (
                        Hash256::zero(),
                        deploy.start_state_root,
                        Hash256::zero(),
                        deploy.public_key,
                    )
                };

            let block_inscription = L2OBlockInscriptionV1 {
                p: "l2o-a".to_string(),
                op: "Block".to_string(),

                l2id,
                l2_block_number: block.block_parameters.block_number.into(),

                bitcoin_block_number: bitcoin_block.block_identifier.index,
                bitcoin_block_hash: Hash256::from_hex(&bitcoin_block.block_identifier.hash)?,

                public_key: L2OCompactPublicKey::from_hex(&block.block_parameters.public_key)?,

                start_state_root,
                end_state_root: end_state_root,

                deposit_state_root: Hash256::from_hex(&block.block_parameters.deposits_root)?,

                start_withdrawal_state_root: start_withdrawal_state_root,
                end_withdrawal_state_root: Hash256::from_hex(
                    &block.block_parameters.withdrawals_root,
                )?,

                proof: L2OAProofData::Groth16BN128(Groth16BN128ProofData {
                    proof: block_proof.proof.clone(),
                    public_inputs: block_proof.public_inputs.clone(),
                }),

                superchain_root: Hash256::zero(),
                signature: L2OSignature512::from_hex(&block.signature)?,
            };

            let mut uncompressed_bytes = Vec::new();
            block_proof.serialize_uncompressed(&mut uncompressed_bytes)?;

            let block_hash = Sha256Hasher::get_l2_block_hash(&block_inscription);
            let public_inputs: [Fr; 2] = block_hash.into();
            if public_inputs.to_vec() != block_proof.public_inputs {
                anyhow::bail!("public inputs mismatch");
            }

            let vk = deploy
                .verifier_data
                .try_as_groth_16_bn_128()
                .ok_or(anyhow::anyhow!("marformed verifier"))?
                .0;

            let processed_vk = Groth16::<Bn254>::process_vk(&vk)?;

            assert!(Groth16::<Bn254>::verify_proof(
                &processed_vk,
                &block_proof.proof,
                &block_proof.public_inputs,
            )?);

            if !public_key.is_zero() {
                verify_sig(&public_key, &block_inscription.signature, &block_hash.0)?;
            }

            store
                .lock()
                .await
                .set_last_block_inscription(block_inscription)?;

            return Ok(());
        }
    }
}

async fn process_rpc_requests(
    store: Arc<Mutex<L2OStoreV1Core<KVQRocksDBStore>>>,
    req: &RpcRequest,
) -> anyhow::Result<RpcResponse> {
    let response = match req.request {
        RequestParams::L2OGetLastBlockInscription(l2oid) => {
            let last_block = store.lock().await.get_last_block_inscription(l2oid)?;

            RpcResponse {
                jsonrpc: rpc::request::Version::V2,
                id: Some(req.id.clone()),
                result: ResponseResult::Success(serde_json::to_value(last_block)?),
            }
        }
        RequestParams::L2OGetDeployInscription(l2oid) => {
            let deploy_inscription = store.lock().await.get_deploy_inscription(l2oid)?;

            RpcResponse {
                jsonrpc: rpc::request::Version::V2,
                id: Some(req.id.clone()),
                result: ResponseResult::Success(serde_json::to_value(deploy_inscription)?),
            }
        }
    };
    Ok(response)
}

async fn process_ordinal_ops(
    store: Arc<Mutex<L2OStoreV1Core<KVQRocksDBStore>>>,
    payload: &BitcoinChainhookOccurrencePayloadV2,
) -> anyhow::Result<()> {
    for block in payload.apply.iter() {
        for tx in block.transactions.iter() {
            for ordinal_operation in tx.metadata.ordinal_operations.iter() {
                match ordinal_operation {
                    OrdinalOperation::InscriptionRevealed(revealed) => {
                        if revealed.content_type.starts_with("application/json") {
                            let decoded = hex::decode(&revealed.content_bytes[2..])?;
                            let inscription = serde_json::from_slice::<L2OInscription>(&decoded)?;
                            process_l2o_inscription(store.clone(), block, tx, inscription).await?;
                        }
                    }
                    OrdinalOperation::InscriptionTransferred(_) => {
                        tracing::info!("xfer")
                    }
                }
            }
        }
    }
    Ok(())
}

async fn handle_ordinal_events(
    store: Arc<Mutex<L2OStoreV1Core<KVQRocksDBStore>>>,
    req: Request<IncomingBody>,
) -> anyhow::Result<Response<BoxBody>> {
    // Aggregate the body...
    let whole_body = req.collect().await?.aggregate();
    // Decode as JSON...
    let data =
        serde_json::from_reader::<_, BitcoinChainhookOccurrencePayloadV2>(whole_body.reader())?;
    process_ordinal_ops(store, &data).await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full("ok"))?;
    Ok(response)
}

async fn handle_rpc_requests(
    store: Arc<Mutex<L2OStoreV1Core<KVQRocksDBStore>>>,
    req: Request<IncomingBody>,
) -> anyhow::Result<Response<BoxBody>> {
    // Aggregate the body...
    let whole_body = req.collect().await?.aggregate();
    // Decode as JSON...
    let data = serde_json::from_reader::<_, RpcRequest>(whole_body.reader())?;
    let response = process_rpc_requests(store, &data).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full(serde_json::to_vec(&response)?))?)
}

async fn route(
    store: Arc<Mutex<L2OStoreV1Core<KVQRocksDBStore>>>,
    req: Request<IncomingBody>,
) -> anyhow::Result<Response<BoxBody>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/events") => handle_ordinal_events(store, req).await,
        (&Method::POST, "/") => handle_rpc_requests(store, req).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(NOTFOUND))?)
        }
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub async fn listen(args: &IndexerOrdHookArgs) -> anyhow::Result<()> {
    let addr: SocketAddr = args.addr.parse()?;
    let store = Arc::new(Mutex::new(L2OStoreV1Core::new(KVQRocksDBStore::new(
        &args.db_path,
    )?)));

    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let store = store.clone();

        tokio::task::spawn(async move {
            let service = service_fn(|req| async { route(store.clone(), req).await });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                tracing::error!("{:?}", err);
            }
        });
    }
}
