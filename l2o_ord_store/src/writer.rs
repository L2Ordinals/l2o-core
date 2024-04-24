use bitcoin::Txid;
use l2o_ord::sat_point::SatPoint;

use crate::balance::Balance;
use crate::ctx::Context;
use crate::event::Receipt;
use crate::log::TransferableLog;
use crate::script_key::ScriptKey;
use crate::table::insert_token_info;
use crate::table::insert_transferable_asset;
use crate::table::remove_transferable_asset;
use crate::table::save_transaction_receipts;
use crate::table::update_burned_token_info;
use crate::table::update_mint_token_info;
use crate::table::update_token_balance;
use crate::tick::Tick;
use crate::token_info::TokenInfo;

impl<'a, 'db, 'txn> Context<'a, 'db, 'txn> {
    pub fn update_token_balance(
        &mut self,
        script_key: &ScriptKey,
        new_balance: Balance,
    ) -> anyhow::Result<()> {
        update_token_balance(self.BRC20_BALANCES, script_key, new_balance)
    }

    pub fn insert_token_info(&mut self, tick: &Tick, new_info: &TokenInfo) -> anyhow::Result<()> {
        insert_token_info(self.BRC20_TOKEN, tick, new_info)
    }

    pub fn update_mint_token_info(
        &mut self,
        tick: &Tick,
        minted_amt: u128,
        minted_block_number: u32,
    ) -> anyhow::Result<()> {
        update_mint_token_info(self.BRC20_TOKEN, tick, minted_amt, minted_block_number)
    }

    pub fn update_burned_token_info(
        &mut self,
        tick: &Tick,
        burned_amt: u128,
    ) -> anyhow::Result<()> {
        update_burned_token_info(self.BRC20_TOKEN, tick, burned_amt)
    }

    pub fn save_transaction_receipts(
        &mut self,
        txid: &Txid,
        receipt: &[Receipt],
    ) -> anyhow::Result<()> {
        save_transaction_receipts(self.BRC20_EVENTS, txid, receipt)
    }

    pub fn insert_transferable_asset(
        &mut self,
        satpoint: SatPoint,
        transferable_asset: &TransferableLog,
    ) -> anyhow::Result<()> {
        insert_transferable_asset(
            self.BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS,
            self.BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS,
            satpoint,
            transferable_asset,
        )
    }

    pub fn remove_transferable_asset(&mut self, satpoint: SatPoint) -> anyhow::Result<()> {
        remove_transferable_asset(
            self.BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS,
            self.BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS,
            satpoint,
        )
    }
}
