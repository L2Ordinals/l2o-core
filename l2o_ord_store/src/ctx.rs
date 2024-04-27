use std::sync::Arc;

use bitcoincore_rpc::Client;
use l2o_ord::chain::Chain;
use l2o_store::core::store::L2OStoreV1Core;
use l2o_store_redb::KVQReDBStore;
use redb::MultimapTable;
use redb::Table;

use crate::entry::HeaderValue;
use crate::entry::InscriptionEntryValue;
use crate::entry::InscriptionIdValue;
use crate::entry::OutPointValue;
use crate::entry::SatPointValue;
use crate::entry::TxidValue;

#[derive(Debug, Clone)]
pub struct ChainContext {
    pub chain: Chain,
    pub blockheight: u32,
    pub blocktime: u32,
    pub bitcoin_rpc: Arc<Client>,
}

pub struct Context<'a, 'db, 'txn> {
    pub chain_ctx: ChainContext,

    pub kv: &'a mut L2OStoreV1Core<KVQReDBStore<Table<'db, 'txn, &'static [u8], &'static [u8]>>>,

    pub height_to_block_header: &'a mut Table<'db, 'txn, u32, &'static HeaderValue>,
    pub height_to_last_sequence_number: &'a mut Table<'db, 'txn, u32, u32>,

    pub brc21_deposits_holding_balances: &'a mut Table<'db, 'txn, &'static [u8], u128>,

    pub sat_to_satpoint: &'a mut Table<'db, 'txn, u64, &'static SatPointValue>,
    pub sat_to_sequence_number: &'a mut MultimapTable<'db, 'txn, u64, u32>,
    pub satpoint_to_sequence_number: &'a mut MultimapTable<'db, 'txn, &'static SatPointValue, u32>,

    pub outpoint_to_entry: &'a mut Table<'db, 'txn, &'static OutPointValue, &'static [u8]>,
    pub outpoint_to_sat_ranges: &'a mut Table<'db, 'txn, &'static OutPointValue, &'static [u8]>,

    pub inscription_id_to_sequence_number: &'a mut Table<'db, 'txn, InscriptionIdValue, u32>,
    pub inscription_number_to_sequence_number: &'a mut Table<'db, 'txn, i32, u32>,
    pub sequence_number_to_inscription_entry: &'a mut Table<'db, 'txn, u32, InscriptionEntryValue>,
    pub sequence_number_to_satpoint: &'a mut Table<'db, 'txn, u32, &'static SatPointValue>,

    pub statistic_to_count: &'a mut Table<'db, 'txn, u64, u64>,

    // BRC20 tables
    pub brc20_balances: &'a mut Table<'db, 'txn, &'static str, &'static [u8]>,
    pub brc20_token: &'a mut Table<'db, 'txn, &'static str, &'static [u8]>,
    pub brc20_events: &'a mut Table<'db, 'txn, &'static TxidValue, &'static [u8]>,
    pub brc20_satpoint_to_transferable_assets:
        &'a mut Table<'db, 'txn, &'static SatPointValue, &'static [u8]>,
    pub brc20_address_ticker_to_transferable_assets:
        &'a mut MultimapTable<'db, 'txn, &'static str, &'static SatPointValue>,

    // BRC21 tables
    pub brc21_balances: &'a mut Table<'db, 'txn, &'static str, &'static [u8]>,
    pub brc21_token: &'a mut Table<'db, 'txn, &'static str, &'static [u8]>,
    pub brc21_events: &'a mut Table<'db, 'txn, &'static TxidValue, &'static [u8]>,
    pub brc21_satpoint_to_transferable_assets:
        &'a mut Table<'db, 'txn, &'static SatPointValue, &'static [u8]>,
    pub brc21_address_ticker_to_transferable_assets:
        &'a mut MultimapTable<'db, 'txn, &'static str, &'static SatPointValue>,
}
