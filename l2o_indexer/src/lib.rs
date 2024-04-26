use std::cmp::Ordering;
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use ark_bn254::Bn254;
use ark_bn254::Fr;
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use base64::Engine;
use bitcoin::Block;
use bitcoin::OutPoint;
use bitcoin::Transaction;
use bitcoin::TxOut;
use bitcoin::Txid;
use bitcoincore_rpc::Auth;
use bitcoincore_rpc::Client;
use bitcoincore_rpc::RpcApi;
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
use jsonrpc_core::types::Request as JsonRpcRequest;
use jsonrpc_core::types::Response as JsonRpcResponse;
use jsonrpc_core::Success;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::hash::L2OHash;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;
use l2o_common::IndexerArgs;
use l2o_crypto::hash::hash_functions::blake3::Blake3Hasher;
use l2o_crypto::hash::hash_functions::keccak256::Keccak256Hasher;
use l2o_crypto::hash::hash_functions::poseidon_goldilocks::PoseidonHasher;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use l2o_crypto::proof::groth16::bn128::verifier_data::Groth16BN128VerifierData;
use l2o_crypto::signature::schnorr::verify_sig;
use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_macros::quick;
use l2o_ord::chain::Chain;
use l2o_ord::hasher::L2OBlockHasher;
use l2o_ord::height::Height;
use l2o_ord::operation::l2o_a::deploy::L2OADeployInscription;
use l2o_ord::operation::l2o_a::L2OABlockInscriptionV1;
use l2o_ord::operation::l2o_a::L2OAInscription;
use l2o_ord_store::ctx::ChainContext;
use l2o_ord_store::rtx::Rtx;
use l2o_ord_store::wtx::BlockData;
use l2o_ord_store::wtx::Wtx;
use l2o_rpc_provider::rpc;
use l2o_rpc_provider::rpc::request::RequestParams;
use l2o_rpc_provider::rpc::request::RpcRequest;
use l2o_rpc_provider::rpc::response::ResponseResult;
use l2o_rpc_provider::rpc::response::RpcResponse;
use l2o_store::core::store::L2OStoreV1Core;
use l2o_store::core::traits::L2OStoreV1;
use l2o_store_rocksdb::KVQRocksDBStore;
use redb::Database;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

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

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub struct Indexer {
    addr: SocketAddr,
    http: Arc<reqwest::blocking::Client>,
    kv: Arc<Mutex<L2OStoreV1Core<KVQRocksDBStore>>>,
    db: Arc<Database>,
    bitcoin_rpc_url: &'static str,
    bitcoin_rpc_auth: &'static str,
    bitcoin_rpc: Arc<Client>,
}

impl Clone for Indexer {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr,
            http: Arc::clone(&self.http),
            kv: Arc::clone(&self.kv),
            db: Arc::clone(&self.db),
            bitcoin_rpc_url: self.bitcoin_rpc_url,
            bitcoin_rpc_auth: self.bitcoin_rpc_auth,
            bitcoin_rpc: Arc::clone(&self.bitcoin_rpc),
        }
    }
}

impl Indexer {
    pub fn new(args: IndexerArgs) -> anyhow::Result<Self> {
        let addr: SocketAddr = args.addr.parse()?;
        let kv = Arc::new(Mutex::new(L2OStoreV1Core::new(KVQRocksDBStore::new(
            &args.db_path,
        )?)));
        let db = Arc::new(Database::create(&args.redb_path)?);
        let bitcoin_rpc_auth = format!(
            "Basic {}",
            &base64::engine::general_purpose::STANDARD.encode(format!(
                "{}:{}",
                args.bitcoin_rpcuser, args.bitcoin_rpcpassword
            ),)
        );
        let bitcoin_rpc = Arc::new(Client::new(
            &args.bitcoin_rpc,
            Auth::UserPass(
                args.bitcoin_rpcuser.to_string(),
                args.bitcoin_rpcpassword.to_string(),
            ),
        )?);
        let http = Arc::new(reqwest::blocking::Client::new());

        let indexer = Indexer {
            addr,
            http,
            kv,
            db,
            bitcoin_rpc_url: Box::leak(args.bitcoin_rpc.into_boxed_str()),
            bitcoin_rpc_auth: Box::leak(bitcoin_rpc_auth.into_boxed_str()),
            bitcoin_rpc,
        };
        let indexerc = indexer.clone();

        thread::spawn(move || -> anyhow::Result<()> {
            let (sender, receiver) = indexerc.spawn_fetcher()?;

            loop {
                let db_block_height = indexerc
                    .db
                    .begin_read()
                    .map_err(anyhow::Error::from)
                    .and_then(|x| {
                        x.block_height()
                            .map(|y| y.unwrap_or(Height(0)))
                            .map_err(anyhow::Error::from)
                    });
                let rpc_block_count = indexerc.bitcoin_rpc.get_block_count();

                match (&db_block_height, &rpc_block_count) {
                    (Ok(db_block_height), Ok(rpc_block_count))
                        if u64::from(db_block_height.n() + 1) <= *rpc_block_count => {}
                    _ => {
                        thread::sleep(Duration::from_secs(600));
                        continue;
                    }
                }

                if let Err(err) = db_block_height
                    .and_then(|height| {
                        indexerc
                            .get_block_with_retries(height.n())
                            .map(|block| (height, block))
                    })
                    .and_then(|(height, block)| {
                        let wxn = indexerc.db.begin_write()?;
                        let chain_ctx = ChainContext {
                            chain: Chain::Regtest,
                            blockheight: height.n(),
                            blocktime: block.header.time,
                        };

                        wxn.index_block(chain_ctx, BlockData::from(block), &sender, &receiver)?;

                        wxn.commit()?;
                        Ok(())
                    })
                {
                    tracing::error!("index error: {}", err);
                }
                thread::sleep(Duration::from_secs(3));
            }
        });

        Ok(indexer)
    }

    pub async fn listen(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        tracing::info!("Listening on http://{}", self.addr);
        loop {
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);
            let indexer = self.clone();

            tokio::task::spawn(async move {
                let service = service_fn(|req| async { indexer.route(req).await });

                if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                    tracing::error!("{:?}", err);
                }
            });
        }
    }

    pub fn spawn_fetcher(&self) -> anyhow::Result<(SyncSender<OutPoint>, Receiver<TxOut>)> {
        // Not sure if any block has more than 20k inputs, but none so far after first
        // inscription block
        const CHANNEL_BUFFER_SIZE: usize = 20_000;
        let (outpoint_sender, outpoint_receiver) =
            mpsc::sync_channel::<OutPoint>(CHANNEL_BUFFER_SIZE);
        let (txout_sender, tx_out_receiver) = mpsc::sync_channel::<TxOut>(CHANNEL_BUFFER_SIZE);

        // Batch 2048 missing inputs at a time. Arbitrarily chosen for now, maybe higher
        // or lower can be faster? Did rudimentary benchmarks with 1024 and 4096
        // and time was roughly the same.
        const BATCH_SIZE: usize = 2048;
        // Default rpcworkqueue in bitcoind is 16, meaning more than 16 concurrent
        // requests will be rejected. Since we are already requesting blocks on
        // a separate thread, and we don't want to break if anything else runs a
        // request, we keep this to 12.
        const PARALLEL_REQUESTS: usize = 12;

        let this = self.clone();
        std::thread::spawn(move || {
            loop {
                let Ok(outpoint) = outpoint_receiver.recv() else {
                    tracing::debug!("Outpoint channel closed");
                    return;
                };
                // There's no try_iter on tokio::sync::mpsc::Receiver like
                // std::sync::mpsc::Receiver. So we just loop until
                // BATCH_SIZE doing try_recv until it returns None.
                let mut outpoints = vec![outpoint];
                for _ in 0..BATCH_SIZE - 1 {
                    let Ok(outpoint) = outpoint_receiver.try_recv() else {
                        break;
                    };
                    outpoints.push(outpoint);
                }
                // Break outpoints into chunks for parallel requests
                let chunk_size = (outpoints.len() / PARALLEL_REQUESTS) + 1;
                let mut results = Vec::with_capacity(PARALLEL_REQUESTS);
                for chunk in outpoints.chunks(chunk_size) {
                    let txids = chunk.iter().map(|outpoint| outpoint.txid).collect();
                    let result =
                        Self::retry(|| this.get_transactions_with_retries(&txids)).unwrap();
                    results.push(result);
                }
                // Send all tx output values back in order
                for (i, tx) in results.iter().flatten().enumerate() {
                    let Ok(_) = txout_sender
                        .send(tx.output[usize::try_from(outpoints[i].vout).unwrap()].clone())
                    else {
                        tracing::error!("Value channel closed unexpectedly");
                        return;
                    };
                }
            }
        });

        Ok((outpoint_sender, tx_out_receiver))
    }

    // pub async fn process_brc21_inscription(
    //     &self,
    //     _bitcoin_block: &BitcoinBlockDataV2,
    //     _bitcoin_tx: &BitcoinTransactionDataV2,
    //     inscription: BRC21Inscription,
    // ) -> anyhow::Result<()> { match inscription {
    //   BRC21Inscription::L2Deposit(_l2deposit) => todo!(),
    //   BRC21Inscription::L2Withdraw(_l2withdraw) => todo!(),
    //   BRC21Inscription::Transfer(_transfer) => {} } Ok(())
    // }

    pub async fn process_events(
        &self,
        payload: &BitcoinChainhookOccurrencePayloadV2,
    ) -> anyhow::Result<()> {
        for block in payload.apply.iter() {
            for tx in block.transactions.iter() {
                for ordinal_operation in tx.metadata.ordinal_operations.iter() {
                    match ordinal_operation {
                        OrdinalOperation::InscriptionRevealed(revealed) => {
                            if !revealed.content_type.starts_with("application/json") {
                                continue;
                            }

                            let decoded = hex::decode(&revealed.content_bytes[2..])?;
                            // let inscription =
                            // serde_json::from_slice::<L2OInscription>(&
                            // decoded)?;
                            // match inscription {
                            // L2OInscription::BRC21(inscription) => {
                            //     self.process_brc21_inscription(block, tx,
                            // inscription)
                            //         .await?
                            // }
                            // L2OInscription::L2OA(inscription) => {
                            //     self.process_l2o_a_inscription(block, tx,
                            // inscription)
                            //         .await?;
                            // }
                            // L2OInscription::BRC20(inscription) => {
                            //     self.process_brc20_inscription(block, tx,
                            // inscription)
                            //         .await?;
                            // }
                            // }
                        }
                        OrdinalOperation::InscriptionTransferred(transfer_data) => {
                            tracing::info!("transfer {:?}", transfer_data);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn process_l2o_a_inscription(
        &self,
        _bitcoin_block: &BitcoinBlockDataV2,
        _bitcoin_tx: &BitcoinTransactionDataV2,
        inscription: L2OAInscription,
    ) -> anyhow::Result<()> {
        match inscription {
            L2OAInscription::Deploy(deploy) => {
                let l2id: u64 = deploy.l2id.into();
                if self.kv.lock().await.has_deployed_l2id(l2id)? {
                    tracing::debug!("l2o {} already deployed", l2id);
                    return Ok(());
                }
                let verifier_data = if deploy.vk.is_groth_16_verifier_serializable() {
                    deploy
                        .vk
                        .try_as_groth_16_verifier_serializable()
                        .ok_or(anyhow::anyhow!("marformed verifier"))?
                        .to_vk()?
                } else {
                    anyhow::bail!("unsupported verifier type");
                };
                let deploy_inscription = L2OADeployInscription {
                    l2id,
                    public_key: L2OCompactPublicKey::from_hex(&deploy.public_key)?,
                    start_state_root: Hash256::from_hex(&deploy.start_state_root)?,
                    hash_function: deploy.hash_function.parse()?,
                    verifier_data: Groth16BN128VerifierData(verifier_data).into(),
                };
                self.kv
                    .lock()
                    .await
                    .report_deploy_inscription(deploy_inscription)?;
                tracing::info!("l2o {} deployed", deploy.l2id);
                Ok(())
            }
            L2OAInscription::Block(block) => {
                let l2id: u64 = block.l2id.into();
                if !self.kv.lock().await.has_deployed_l2id(l2id)? {
                    tracing::debug!("l2o {} not deployed yet", l2id);
                    return Ok(());
                }

                let deploy = self.kv.lock().await.get_deploy_inscription(l2id)?;

                let block_proof = if deploy.verifier_data.is_groth_16_bn_128() {
                    block
                        .proof
                        .try_as_groth_16_proof_serializable()
                        .ok_or(anyhow::anyhow!("marformed proof"))?
                        .to_proof_with_public_inputs_groth16_bn254()?
                } else {
                    anyhow::bail!("unsupported proof type");
                };

                let bitcoin_block_hash = self
                    .bitcoin_rpc
                    .get_block_hash(block.bitcoin_block_number)?
                    .to_string()
                    .trim_start_matches("0x")
                    .to_string();
                if bitcoin_block_hash != block.bitcoin_block_hash {
                    anyhow::bail!("bitcoin block number mismatch");
                }

                let superchain_root = self.kv.lock().await.get_superchainroot_at_block(
                    block.bitcoin_block_number,
                    deploy.hash_function,
                )?;
                if superchain_root != Hash256::from_hex(&block.superchain_root)? {
                    anyhow::bail!("superchain root mismatch");
                }

                let (start_state_root, end_state_root, start_withdrawal_state_root, public_key) =
                    if let Ok(last_block) = self.kv.lock().await.get_last_block_inscription(l2id) {
                        if u64::from(block.block_parameters.block_number)
                            != last_block.l2_block_number + 1
                        {
                            anyhow::bail!("block must be consecutive");
                        }

                        if block.bitcoin_block_number <= last_block.bitcoin_block_number {
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

                let block_inscription = L2OABlockInscriptionV1 {
                    l2id,
                    l2_block_number: block.block_parameters.block_number.into(),

                    bitcoin_block_number: block.bitcoin_block_number,
                    bitcoin_block_hash: Hash256::from_hex(&block.bitcoin_block_hash)?,

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

                    superchain_root: superchain_root,
                    signature: L2OSignature512::from_hex(&block.signature)?,
                };

                let mut uncompressed_bytes = Vec::new();
                block_proof.serialize_uncompressed(&mut uncompressed_bytes)?;

                let block_hash = if deploy.hash_function.is_sha_256() {
                    Sha256Hasher::get_l2_block_hash(&block_inscription)
                } else if deploy.hash_function.is_blake_3() {
                    Blake3Hasher::get_l2_block_hash(&block_inscription)
                } else if deploy.hash_function.is_keccak_256() {
                    Keccak256Hasher::get_l2_block_hash(&block_inscription)
                } else if deploy.hash_function.is_poseidon_goldilocks() {
                    PoseidonHasher::get_l2_block_hash(&block_inscription)
                } else {
                    anyhow::bail!("unsupported hash function");
                };

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

                self.kv
                    .lock()
                    .await
                    .set_last_block_inscription(block_inscription)?;
                tracing::info!("l2id {} block: {}", l2id, block.bitcoin_block_number);

                return Ok(());
            }
        }
    }

    // pub async fn process_brc20_inscription(
    //     &self,
    //     _bitcoin_block: &BitcoinBlockDataV2,
    //     _bitcoin_tx: &BitcoinTransactionDataV2,
    //     inscription: BRC20Inscription,
    // ) -> anyhow::Result<()> { match inscription {
    //   BRC20Inscription::Transfer(transfer) => { tracing::info!("{:?}", transfer);
    //   } _ => {} } Ok(())
    // }

    pub async fn process_rpc_requests(&self, req: &RpcRequest) -> anyhow::Result<RpcResponse> {
        let response = match req.request {
            RequestParams::L2OGetLastBlockInscription(l2oid) => {
                let last_block = self.kv.lock().await.get_last_block_inscription(l2oid)?;
                serde_json::to_value(last_block)?
            }
            RequestParams::L2OGetDeployInscription(l2oid) => {
                let deploy_inscription = self.kv.lock().await.get_deploy_inscription(l2oid)?;
                serde_json::to_value(deploy_inscription)?
            }
            RequestParams::L2OGetStateRootAtBlock((l2oid, block_number, hash_function)) => {
                let state_root = self.kv.lock().await.get_state_root_at_block(
                    l2oid,
                    block_number,
                    hash_function,
                )?;
                serde_json::to_value(state_root)?
            }
            RequestParams::L2OGetMerkleProofStateRootAtBlock((
                l2oid,
                block_number,
                hash_function,
            )) => {
                let merkle_proof_state_root = self
                    .kv
                    .lock()
                    .await
                    .get_merkle_proof_state_root_at_block(l2oid, block_number, hash_function)?;
                serde_json::to_value(merkle_proof_state_root)?
            }
            RequestParams::L2OGetSuperchainStateRootAtBlock((block_number, hash_function)) => {
                let superchain_state_root = self
                    .kv
                    .lock()
                    .await
                    .get_superchainroot_at_block(block_number, hash_function)?;
                serde_json::to_value(superchain_state_root)?
            }
        };
        Ok(RpcResponse {
            jsonrpc: rpc::request::Version::V2,
            id: Some(req.id.clone()),
            result: ResponseResult::Success(response),
        })
    }

    pub async fn handle_ordinal_events(
        &self,

        req: Request<IncomingBody>,
    ) -> anyhow::Result<Response<BoxBody>> {
        // Aggregate the body...
        let whole_body = req.collect().await?.aggregate();
        // Decode as JSON...
        let data =
            serde_json::from_reader::<_, BitcoinChainhookOccurrencePayloadV2>(whole_body.reader())?;
        self.process_events(&data).await?;

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full("ok"))?;
        Ok(response)
    }

    pub async fn handle_rpc_requests(
        &self,
        req: Request<IncomingBody>,
    ) -> anyhow::Result<Response<BoxBody>> {
        // Aggregate the body...
        let whole_body = req.collect().await?.aggregate();
        // Decode as JSON...
        let data = serde_json::from_reader::<_, RpcRequest>(whole_body.reader())?;
        let response = self.process_rpc_requests(&data).await?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(serde_json::to_vec(&response)?))?)
    }

    pub async fn route(&self, req: Request<IncomingBody>) -> anyhow::Result<Response<BoxBody>> {
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/api/events") => self.handle_ordinal_events(req).await,
            (&Method::POST, "/") => self.handle_rpc_requests(req).await,
            _ => {
                // Return 404 not found response.
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(full(NOTFOUND))?)
            }
        }
    }

    pub fn get_block_with_retries(&self, height: u32) -> anyhow::Result<Block> {
        let block_hash = Self::retry(|| self.bitcoin_rpc.get_block_hash(height.into()))?;
        Ok(Self::retry(|| self.bitcoin_rpc.get_block(&block_hash))?)
    }

    pub fn get_transactions_with_retries(
        &self,
        txids: &Vec<Txid>,
    ) -> anyhow::Result<Vec<Transaction>> {
        use jsonrpc_core::types::*;

        if txids.is_empty() {
            return Ok(Vec::new());
        }

        let requests = JsonRpcRequest::Batch(
            txids
                .iter()
                .enumerate()
                .map(|(i, txid)| {
                    Call::MethodCall(MethodCall {
                        jsonrpc: Some(Version::V2),
                        method: "getrawtransaction".to_string(),
                        params: Params::Array(vec![json!(txid)]),
                        id: Id::Num(i as u64),
                    })
                })
                .collect(),
        );

        let responses = Self::retry(|| self.bitcoin_raw_rpc_call(&requests))?;

        responses
            .into_iter()
            .map(|response| {
                let hex = match response.result {
                    Value::String(hex) => hex,
                    _ => return Err(anyhow::anyhow!("invalid response")),
                };

                let tx = bitcoin::consensus::deserialize(&hex::decode(&hex)?)?;
                Ok(tx)
            })
            .collect()
    }

    fn bitcoin_raw_rpc_call(&self, request: &JsonRpcRequest) -> anyhow::Result<Vec<Success>> {
        use jsonrpc_core::types::*;

        let resp = self
            .http
            .post(self.bitcoin_rpc_url)
            .header("Authorization", self.bitcoin_rpc_auth)
            .json(request)
            .send()?
            .json::<JsonRpcResponse>()?;

        let unwrap_output = |output: Output| match output {
            Output::Success(r) => Ok(r),
            Output::Failure(f) => Err(f.error),
        };

        let mut res = match resp {
            JsonRpcResponse::Single(output) => vec![unwrap_output(output)?],
            JsonRpcResponse::Batch(outputs) => outputs
                .into_iter()
                .map(|output| unwrap_output(output))
                .collect::<Result<Vec<Success>, Error>>()?,
        };

        res.sort_by(|a, b| match (&a.id, &b.id) {
            (&Id::Num(a), &Id::Num(b)) => a.cmp(&b),
            _ => Ordering::Equal,
        });

        Ok(res)
    }

    fn retry<T, E: Display>(f: impl Fn() -> Result<T, E>) -> Result<T, E> {
        let mut retries = 0;
        loop {
            let err = quick!(f());

            retries += 1;
            let seconds = 1 << retries;
            tracing::warn!("retrying in {seconds}s: {err}");

            if seconds > 120 {
                tracing::error!("would sleep for more than 120s, giving up");
                return Err(err);
            }

            std::thread::sleep(Duration::from_secs(seconds));
        }
    }
}
