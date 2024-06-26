use clap::Args;

#[derive(Clone, Args)]
pub struct IndexerArgs {
    #[clap(short, env, long, default_value = "0.0.0.0:3000", env)]
    pub addr: String,
    #[clap(env, long, default_value = "regtest", env)]
    pub network: String,
    #[clap(env, long, default_value = "http://localhost:18443", env)]
    pub bitcoin_rpc: String,
    #[clap(env, long, default_value = "devnet", env)]
    pub bitcoin_rpcuser: String,
    #[clap(env, long, default_value = "devnet", env)]
    pub bitcoin_rpcpassword: String,
    #[clap(short, env, long, default_value = "db", env)]
    pub db_path: String,
}

#[derive(Clone, Args)]
#[cfg(debug_assertions)]
pub struct SequencerArgs {
    #[clap(short, env, long, default_value = "http://localhost:3000", env)]
    pub indexer_url: String,
    #[clap(env, long, default_value = "http://localhost:18443", env)]
    pub bitcoin_rpc: String,
    #[clap(env, long, default_value = "devnet", env)]
    pub bitcoin_rpcuser: String,
    #[clap(env, long, default_value = "devnet", env)]
    pub bitcoin_rpcpassword: String,
    #[clap(short, env, long, default_value = "1", env)]
    pub l2id: u64,
}

#[derive(Clone, Args)]
#[cfg(debug_assertions)]
pub struct InitializerArgs {
    #[clap(short, env, long, default_value = "http://localhost:3000", env)]
    pub indexer_url: String,
    #[clap(env, long, default_value = "http://localhost:18443", env)]
    pub bitcoin_rpc: String,
    #[clap(env, long, default_value = "devnet", env)]
    pub bitcoin_rpcuser: String,
    #[clap(env, long, default_value = "devnet", env)]
    pub bitcoin_rpcpassword: String,
    #[clap(short, env, long, default_value = "1", env)]
    pub l2id: u64,
}
