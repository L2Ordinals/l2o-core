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
use l2o_ord_store::rtx::Rtx;
use l2o_ord_store::table::KV;
use l2o_rpc::request;
use l2o_rpc::request::RequestParams;
use l2o_rpc::request::RpcRequest;
use l2o_rpc::response::ResponseResult;
use l2o_rpc::response::RpcResponse;
use l2o_store::core::store::L2OStoreV1Core;
use l2o_store::core::traits::L2OStoreReaderV1;
use l2o_store_redb::KVQReDBStore;
use tokio::net::TcpListener;

use crate::Indexer;

type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

static NOTFOUND: &[u8] = b"Not Found";

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

impl Indexer {
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
            RequestParams::BRC20GetTickInfo(ref tick) => {
                serde_json::to_value(rxn.brc20_get_tick_info(tick)?)?
            }
            RequestParams::BRC20GetAllTickInfo => {
                serde_json::to_value(rxn.brc20_get_all_tick_info()?)?
            }
            RequestParams::BRC20GetBalanceByAddress((ref tick, ref script_key)) => {
                serde_json::to_value(rxn.brc20_get_balance_by_address(tick, script_key.clone())?)?
            }
            RequestParams::BRC20GetAllBalanceByAddress(ref script_key) => {
                serde_json::to_value(rxn.brc20_get_all_balance_by_address(script_key.clone())?)?
            }
            RequestParams::BRC20TransactionIdToTransactionReceipt(ref txid) => {
                serde_json::to_value(
                    rxn.brc20_transaction_id_to_transaction_receipt(txid.clone())?,
                )?
            }
            RequestParams::BRC20GetTickTransferableByAddress((ref tick, ref script_key)) => {
                serde_json::to_value(
                    rxn.brc20_get_tick_transferable_by_address(tick, script_key.clone())?,
                )?
            }
            RequestParams::BRC20GetAllTransferableByAddress(ref script_key) => {
                serde_json::to_value(
                    rxn.brc20_get_all_transferable_by_address(script_key.clone())?,
                )?
            }
            RequestParams::BRC20TransferableAssetsOnOutputWithSatpoints(ref outpoint) => {
                serde_json::to_value(
                    rxn.brc20_transferable_assets_on_output_with_satpoints(outpoint.clone())?,
                )?
            }
            RequestParams::BRC21GetTickInfo(ref tick) => {
                serde_json::to_value(rxn.brc21_get_tick_info(tick)?)?
            }
            RequestParams::BRC21GetAllTickInfo => {
                serde_json::to_value(rxn.brc21_get_all_tick_info()?)?
            }
            RequestParams::BRC21GetBalanceByAddress((ref tick, ref script_key)) => {
                serde_json::to_value(rxn.brc21_get_balance_by_address(tick, script_key.clone())?)?
            }
            RequestParams::BRC21GetAllBalanceByAddress(ref script_key) => {
                serde_json::to_value(rxn.brc21_get_all_balance_by_address(script_key.clone())?)?
            }
            RequestParams::BRC21TransactionIdToTransactionReceipt(ref txid) => {
                serde_json::to_value(
                    rxn.brc21_transaction_id_to_transaction_receipt(txid.clone())?,
                )?
            }
            RequestParams::BRC21GetTickTransferableByAddress((ref tick, ref script_key)) => {
                serde_json::to_value(
                    rxn.brc21_get_tick_transferable_by_address(tick, script_key.clone())?,
                )?
            }
            RequestParams::BRC21GetAllTransferableByAddress(ref script_key) => {
                serde_json::to_value(
                    rxn.brc21_get_all_transferable_by_address(script_key.clone())?,
                )?
            }
            RequestParams::BRC21TransferableAssetsOnOutputWithSatpoints(ref outpoint) => {
                serde_json::to_value(
                    rxn.brc21_transferable_assets_on_output_with_satpoints(outpoint.clone())?,
                )?
            }
        };
        Ok(RpcResponse {
            jsonrpc: request::Version::V2,
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
}
