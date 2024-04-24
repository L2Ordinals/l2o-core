use l2o_ord::chain::Chain;
use redb::MultimapTable;
use redb::Table;

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
    pub chain_conf: ChainContext,

    pub(crate) OUTPOINT_TO_ENTRY: &'a mut Table<'db, 'txn, &'static OutPointValue, &'static [u8]>,

    // BRC20 tables
    pub BRC20_BALANCES: &'a mut Table<'db, 'txn, &'static str, &'static [u8]>,
    pub BRC20_TOKEN: &'a mut Table<'db, 'txn, &'static str, &'static [u8]>,
    pub BRC20_EVENTS: &'a mut Table<'db, 'txn, &'static TxidValue, &'static [u8]>,
    pub BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS:
        &'a mut Table<'db, 'txn, &'static SatPointValue, &'static [u8]>,
    pub BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS:
        &'a mut MultimapTable<'db, 'txn, &'static str, &'static SatPointValue>,
}
