use std::net::SocketAddr;
use std::sync::Arc;

use ark_bn254::Fr;
use ark_groth16::Groth16;
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
use l2o_crypto::hash::hash_functions::block_hasher::get_block_payload_bytes;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::traits::L2OBlockHasher;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use l2o_crypto::proof::groth16::bn128::verifier_data::Groth16BN128VerifierData;
use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::proof::L2OAVerifierData;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_store::core::store::L2OStoreV1Core;
use l2o_store::core::traits::L2OStoreV1;
use l2o_store_rocksdb::KVQRocksDBStore;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub mod l2o;
pub mod proof;
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
            if store.lock().await.has_deployed_l2id(deploy.l2id.into())? {
                anyhow::bail!("L2o already deployed");
            }
            let proof_type: L2OAProofType = deploy.proof_type.parse().unwrap();
            let deploy_inscription = L2ODeployInscription {
                p: "l2o-a".to_string(),
                op: "Deploy".to_string(),
                l2id: deploy.l2id.into(),
                public_key: L2OCompactPublicKey::from_hex(&deploy.public_key).unwrap(),
                start_state_root: Hash256::from_hex(&deploy.start_state_root).unwrap(),
                hash_function: deploy.hash_function.parse().unwrap(),
                proof_type: proof_type,
                verifier_data: if proof_type == L2OAProofType::Groth16BN128 {
                    L2OAVerifierData::Groth16BN128(Groth16BN128VerifierData(
                        deploy.vk.to_verifying_key_groth16_bn254(),
                    ))
                } else {
                    panic!("Unsupported verifier type")
                },
            };
            store
                .lock()
                .await
                .report_deploy_inscription(deploy_inscription)?;
            tracing::info!("deployed l2o: {:?}", deploy.l2id);
            Ok(())
        }
        L2OInscription::Block(block) => {
            if !store.lock().await.has_deployed_l2id(block.l2id.into())? {
                anyhow::bail!("L2o not deployed yet");
            }

            let deploy_inscription = store
                .lock()
                .await
                .get_deploy_inscription(block.l2id.into())?;

            let last_block_res = store
                .lock()
                .await
                .get_last_block_inscription(block.l2id.into());

            if let Err(_) = last_block_res {
                let block_proof = block.proof.to_proof_groth16_bn254();

                let block_inscription = L2OBlockInscriptionV1 {
                    p: "l2o-a".to_string(),
                    op: "Block".to_string(),

                    l2id: block.l2id.into(),
                    l2_block_number: block.block_parameters.block_number.into(),

                    bitcoin_block_number: bitcoin_block.block_identifier.index,
                    bitcoin_block_hash: Hash256::from_hex(&bitcoin_block.block_identifier.hash)
                        .unwrap(),

                    public_key: L2OCompactPublicKey::from_hex(&block.block_parameters.public_key)
                        .unwrap(),

                    start_state_root: Hash256::zero(),
                    end_state_root: deploy_inscription.start_state_root,

                    deposit_state_root: Hash256::from_hex(&block.block_parameters.deposits_root)
                        .unwrap(),

                    start_withdrawal_state_root: Hash256::zero(),
                    end_withdrawal_state_root: Hash256::from_hex(
                        &block.block_parameters.withdrawals_root,
                    )
                    .unwrap(),

                    proof: L2OAProofData::Groth16BN128(Groth16BN128ProofData {
                        proof: block_proof,
                        public_inputs: vec![],
                    }),

                    superchain_root: Hash256::zero(),
                    signature: L2OSignature512::from_hex(&block.signature).unwrap(),
                };

                let msg = get_block_payload_bytes(&block_inscription);
                if !deploy_inscription.public_key.is_zero() {
                    l2o_crypto::signature::schnorr::verify(
                        &deploy_inscription.public_key,
                        &block_inscription.signature,
                        &msg,
                    )
                    .unwrap();
                }

                store
                    .lock()
                    .await
                    .set_last_block_inscription(block_inscription)?;

                return Ok(());
            }

            let last_block = last_block_res.unwrap();
            if u64::from(block.block_parameters.block_number) != last_block.l2_block_number + 1 {
                anyhow::bail!("block must be consecutive");
            }

            if bitcoin_block.block_identifier.index > last_block.bitcoin_block_number {
                anyhow::bail!("bitcoin block must be bigger than previous");
            }

            if last_block.end_state_root
                != Hash256::from_hex(&block.block_parameters.state_root).unwrap()
            {
                anyhow::bail!(
                    "last block's end state root must be equal to this block's start state root"
                );
            }

            if last_block.end_withdrawal_state_root
                != Hash256::from_hex(&block.block_parameters.withdrawals_root).unwrap()
            {
                anyhow::bail!("last block's end withdrawals_root must be equal to this block's start withdrawals_root");
            }

            let deploy_inscription = store
                .lock()
                .await
                .get_deploy_inscription(block.l2id.into())?;

            if deploy_inscription.proof_type != L2OAProofType::Groth16BN128 {
                anyhow::bail!("only groth16bn128 is supported");
            }

            let block_proof = block.proof.to_proof_groth16_bn254();

            let mut block_inscription = L2OBlockInscriptionV1 {
                p: "l2o-a".to_string(),
                op: "Block".to_string(),

                l2id: block.l2id.into(),
                l2_block_number: block.block_parameters.block_number.into(),

                bitcoin_block_number: bitcoin_block.block_identifier.index,
                bitcoin_block_hash: Hash256::from_hex(&bitcoin_block.block_identifier.hash)
                    .unwrap(),

                public_key: L2OCompactPublicKey::from_hex(&block.block_parameters.public_key)
                    .unwrap(),

                start_state_root: last_block.end_state_root,
                end_state_root: Hash256::from_hex(&block.block_parameters.state_root).unwrap(),

                deposit_state_root: Hash256::from_hex(&block.block_parameters.deposits_root)
                    .unwrap(),

                start_withdrawal_state_root: last_block.end_withdrawal_state_root,
                end_withdrawal_state_root: Hash256::from_hex(
                    &block.block_parameters.withdrawals_root,
                )
                .unwrap(),

                proof: L2OAProofData::Groth16BN128(Groth16BN128ProofData {
                    proof: block_proof.clone(),
                    public_inputs: vec![],
                }),

                superchain_root: Hash256::zero(),

                signature: L2OSignature512::from_hex(&block.signature).unwrap(),
            };

            let msg = get_block_payload_bytes(&block_inscription);
            if !last_block.public_key.is_zero() {
                l2o_crypto::signature::schnorr::verify(
                    &deploy_inscription.public_key,
                    &block_inscription.signature,
                    &msg,
                )
                .unwrap();
            }

            let public_inputs: [Fr; 2] = Sha256Hasher::get_l2_block_hash(&block_inscription).into();
            match block_inscription.proof {
                L2OAProofData::Groth16BN128(ref mut p) => {
                    p.public_inputs.extend(public_inputs.into_iter());
                }
                _ => {
                    unreachable!()
                }
            }

            let vk = match deploy_inscription.verifier_data {
                L2OAVerifierData::Groth16BN128(vk) => vk.0,
                _ => {
                    unreachable!()
                }
            };

            Groth16::<ark_bn254::Bn254>::verify_proof(&vk.into(), &block_proof, &public_inputs)
                .unwrap();

            store
                .lock()
                .await
                .set_last_block_inscription(block_inscription)?;

            Ok(())
        }
    }
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

async fn api_post_response(
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

async fn route(
    store: Arc<Mutex<L2OStoreV1Core<KVQRocksDBStore>>>,
    req: Request<IncomingBody>,
) -> anyhow::Result<Response<BoxBody>> {
    match (req.method(), req.uri().path()) {
        (&Method::POST, "/api/events") => api_post_response(store, req).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(NOTFOUND))
                .unwrap())
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

        let service = service_fn(|req| async { route(store.clone(), req).await });

        if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
            tracing::error!("Failed to serve connection: {:?}", err);
        }
    }
}
