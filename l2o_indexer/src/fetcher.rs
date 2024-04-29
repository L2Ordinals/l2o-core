use std::cmp::Ordering;
use std::fmt::Display;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::time::Duration;

use bitcoin::Block;
use bitcoin::OutPoint;
use bitcoin::Transaction;
use bitcoin::TxOut;
use bitcoin::Txid;
use bitcoincore_rpc::RpcApi;
use jsonrpc_core::types::Request as JsonRpcRequest;
use jsonrpc_core::types::Response as JsonRpcResponse;
use jsonrpc_core::Success;
use l2o_macros::quick;
use serde_json::json;

use crate::Indexer;

impl Indexer {
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

    pub fn spawn_fetcher(&self) -> anyhow::Result<(SyncSender<OutPoint>, Receiver<TxOut>)> {
        tracing::info!("spawning fetcher...");
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
}
