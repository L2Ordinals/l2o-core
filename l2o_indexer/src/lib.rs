use std::cmp::Ordering;
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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
use l2o_common::IndexerArgs;
use l2o_macros::quick;
use l2o_ord::chain::Chain;
use l2o_ord::height::Height;
use l2o_ord_store::ctx::ChainContext;
use l2o_ord_store::rtx::Rtx;
use l2o_ord_store::table::KV;
use l2o_ord_store::wtx::BlockData;
use l2o_ord_store::wtx::Wtx;
use l2o_rpc_provider::rpc;
use l2o_rpc_provider::rpc::request::RequestParams;
use l2o_rpc_provider::rpc::request::RpcRequest;
use l2o_rpc_provider::rpc::response::ResponseResult;
use l2o_rpc_provider::rpc::response::RpcResponse;
use l2o_store::core::store::L2OStoreV1Core;
use l2o_store::core::traits::L2OStoreReaderV1;
use l2o_store_redb::KVQReDBStore;
use redb::Database;
use serde_json::json;
use tokio::net::TcpListener;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static NOTFOUND: &[u8] = b"Not Found";

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub struct Indexer {
    addr: SocketAddr,
    http: Arc<reqwest::blocking::Client>,
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
                            bitcoin_rpc: Arc::clone(&indexerc.bitcoin_rpc),
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

    pub async fn process_rpc_requests(&self, req: &RpcRequest) -> anyhow::Result<RpcResponse> {
        let rxn = self.db.begin_read()?;
        let store = L2OStoreV1Core::new(KVQReDBStore::new(rxn.open_table(KV)?));
        let response = match req.request {
            RequestParams::L2OGetLastBlockInscription(l2id) => {
                let last_block = store.get_last_block_inscription(l2id)?;
                serde_json::to_value(last_block)?
            }
            RequestParams::L2OGetDeployInscription(l2id) => {
                let deploy_inscription = store.get_deploy_inscription(l2id)?;
                serde_json::to_value(deploy_inscription)?
            }
            RequestParams::L2OGetStateRootAtBlock((l2id, block_number, hash_function)) => {
                let state_root =
                    store.get_state_root_at_block(l2id, block_number, hash_function)?;
                serde_json::to_value(state_root)?
            }
            RequestParams::L2OGetMerkleProofStateRootAtBlock((
                l2id,
                block_number,
                hash_function,
            )) => {
                let merkle_proof_state_root = store.get_merkle_proof_state_root_at_block(
                    l2id,
                    block_number,
                    hash_function,
                )?;
                serde_json::to_value(merkle_proof_state_root)?
            }
            RequestParams::L2OGetSuperchainStateRootAtBlock((block_number, hash_function)) => {
                let superchain_state_root =
                    store.get_superchainroot_at_block(block_number, hash_function)?;
                serde_json::to_value(superchain_state_root)?
            }
        };
        Ok(RpcResponse {
            jsonrpc: rpc::request::Version::V2,
            id: Some(req.id.clone()),
            result: ResponseResult::Success(response),
        })
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
