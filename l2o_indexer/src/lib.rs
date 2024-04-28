use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use base64::Engine;
use bitcoincore_rpc::Auth;
use bitcoincore_rpc::Client;
use bitcoincore_rpc::RpcApi;
use l2o_common::IndexerArgs;
use l2o_ord::chain::Chain;
use l2o_ord::height::Height;
use l2o_ord_store::ctx::ChainContext;
use l2o_ord_store::reorg::ReorgError;
use l2o_ord_store::rtx::Rtx;
use l2o_ord_store::table::HEIGHT_TO_BLOCK_HEADER;
use l2o_ord_store::wtx::BlockData;
use l2o_ord_store::wtx::Wtx;
use redb::Database;
use redb::ReadableTable;
use tokio::task::spawn_blocking;

pub mod fetcher;
pub mod rpc_server;

pub struct Indexer {
    addr: SocketAddr,
    http: Arc<reqwest::blocking::Client>,
    db: Arc<Database>,
    chain: Chain,
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
            chain: self.chain.clone(),
            bitcoin_rpc_url: self.bitcoin_rpc_url,
            bitcoin_rpc_auth: self.bitcoin_rpc_auth,
            bitcoin_rpc: Arc::clone(&self.bitcoin_rpc),
        }
    }
}

impl Indexer {
    pub async fn new(args: IndexerArgs) -> anyhow::Result<Self> {
        let addr: SocketAddr = args.addr.parse()?;
        let db = Arc::new(Database::create(&args.db_path)?);
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
        let chain = args.network.parse()?;
        let indexer = spawn_blocking(move || {
            let http = Arc::new(reqwest::blocking::Client::new());

            let indexer = Indexer {
                addr,
                http,
                db,
                chain,
                bitcoin_rpc_url: Box::leak(args.bitcoin_rpc.into_boxed_str()),
                bitcoin_rpc_auth: Box::leak(bitcoin_rpc_auth.into_boxed_str()),
                bitcoin_rpc,
            };

            indexer
        })
        .await?;

        let indexerc = indexer.clone();

        spawn_blocking(move || {
            indexerc.spawn_indexer();
        })
        .await?;

        Ok(indexer)
    }

    pub fn spawn_indexer(self) {
        tracing::info!("spawning indexer...");
        std::thread::spawn(move || -> anyhow::Result<()> {
            let (sender, receiver) = self.spawn_fetcher()?;

            loop {
                let db_block_height =
                    self.db
                        .begin_write()
                        .map_err(anyhow::Error::from)
                        .and_then(|mut wtx| {
                            let res = {
                                let table = wtx.open_table(HEIGHT_TO_BLOCK_HEADER)?;
                                let value = table
                                    .range(0..)?
                                    .next_back()
                                    .and_then(|result| result.ok())
                                    .map(|(height, _header)| Height(height.value() + 1));
                                value
                            };
                            wtx.set_durability(redb::Durability::Immediate);
                            wtx.commit()?;
                            Ok(res.unwrap_or(Height(0)))
                        });

                let rpc_block_count = self.bitcoin_rpc.get_block_count();

                match (&db_block_height, &rpc_block_count) {
                    (Ok(db_block_height), Ok(rpc_block_count))
                        if u64::from(db_block_height.n() + 1) <= *rpc_block_count => {}
                    _ => {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                }

                if let Err(err) = db_block_height
                    .and_then(|height| {
                        self.get_block_with_retries(height.n())
                            .map(|block| (height, block))
                    })
                    .and_then(|(height, block)| {
                        let block_data = BlockData::from(block);
                        {
                            let rxn = self.db.begin_read()?;
                            if let Err(err) =
                                rxn.detect_reorg(self.bitcoin_rpc.clone(), &block_data, height.n())
                            {
                                match err.downcast_ref() {
                                    Some(&ReorgError::Unrecoverable) => {
                                        panic!("unrecoverable reorg")
                                    }
                                    Some(&ReorgError::Recoverable { height, depth }) => {
                                        let mut wxn = self.db.begin_write()?;
                                        wxn.set_durability(redb::Durability::Immediate);
                                        wxn.handle_reorg(height, depth)?;
                                        wxn.commit()?;

                                        return Ok(());
                                    }
                                    _ => return Err(err),
                                }
                            }
                        }

                        let mut wxn = self.db.begin_write()?;
                        wxn.set_durability(redb::Durability::Immediate);
                        let chain_ctx = ChainContext {
                            chain: self.chain,
                            blockheight: height.n(),
                            blocktime: block_data.header.time,
                            bitcoin_rpc: Arc::clone(&self.bitcoin_rpc),
                        };

                        wxn.index_block(chain_ctx, block_data, &sender, &receiver)?;
                        wxn.commit()?;

                        let mut wxn = self.db.begin_write()?;
                        wxn.set_durability(redb::Durability::Immediate);
                        wxn.delete_oldest_savepoint(self.bitcoin_rpc.clone(), height.n())?;
                        wxn.commit()?;

                        let mut wxn = self.db.begin_write()?;
                        wxn.set_durability(redb::Durability::Immediate);
                        wxn.persistent_savepoint()?;

                        wxn.commit()?;
                        Ok(())
                    })
                {
                    tracing::error!("index error: {}", err);
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}
