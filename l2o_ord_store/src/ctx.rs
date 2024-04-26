use l2o_ord::chain::Chain;
use redb::MultimapTable;
use redb::Table;

use crate::entry::HeaderValue;
use crate::entry::InscriptionEntryValue;
use crate::entry::InscriptionIdValue;
use crate::entry::OutPointValue;
use crate::entry::SatPointValue;
use crate::entry::TxidValue;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ChainContext {
    pub chain: Chain,
    pub blockheight: u32,
    pub blocktime: u32,
}

pub struct Context<'a, 'db, 'txn> {
    pub chain_ctx: ChainContext,

    pub height_to_block_header: &'a mut Table<'db, 'txn, u32, &'static HeaderValue>,
    pub height_to_last_sequence_number: &'a mut Table<'db, 'txn, u32, u32>,

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
}
