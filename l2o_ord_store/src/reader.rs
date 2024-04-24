use bitcoin::OutPoint;
use bitcoin::Txid;
use l2o_ord::chain::Chain;
use l2o_ord::sat_point::SatPoint;

use crate::balance::Balance;
use crate::ctx::Context;
use crate::event::Receipt;
use crate::log::TransferableLog;
use crate::script_key::ScriptKey;
use crate::table::get_balance;
use crate::table::get_balances;
use crate::table::get_token_info;
use crate::table::get_tokens_info;
use crate::table::get_transaction_receipts;
use crate::table::get_transferable_assets_by_account;
use crate::table::get_transferable_assets_by_account_ticker;
use crate::table::get_transferable_assets_by_outpoint;
use crate::table::get_transferable_assets_by_satpoint;
use crate::table::get_txout_by_outpoint;
use crate::tick::Tick;
use crate::token_info::TokenInfo;

impl<'a, 'db, 'txn> Context<'a, 'db, 'txn> {
    pub fn get_balances(&self, script_key: &ScriptKey) -> anyhow::Result<Vec<Balance>> {
        get_balances(self.BRC20_BALANCES, script_key)
    }

    pub fn get_balance(
        &self,
        script_key: &ScriptKey,
        tick: &Tick,
    ) -> anyhow::Result<Option<Balance>> {
        get_balance(self.BRC20_BALANCES, script_key, tick)
    }

    pub fn get_token_info(&self, tick: &Tick) -> anyhow::Result<Option<TokenInfo>> {
        get_token_info(self.BRC20_TOKEN, tick)
    }

    pub fn get_tokens_info(&self) -> anyhow::Result<Vec<TokenInfo>> {
        get_tokens_info(self.BRC20_TOKEN)
    }

    pub fn get_transaction_receipts(&self, txid: &Txid) -> anyhow::Result<Option<Vec<Receipt>>> {
        get_transaction_receipts(self.BRC20_EVENTS, txid)
    }

    pub fn get_transferable_assets_by_account(
        &self,
        script: &ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_account(
            self.BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS,
            self.BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS,
            script,
        )
    }

    pub fn get_transferable_assets_by_account_ticker(
        &self,
        script: &ScriptKey,
        tick: &Tick,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_account_ticker(
            self.BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS,
            self.BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS,
            script,
            tick,
        )
    }

    pub fn get_transferable_assets_by_satpoint(
        &self,
        satpoint: &SatPoint,
    ) -> anyhow::Result<Option<TransferableLog>> {
        get_transferable_assets_by_satpoint(self.BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS, satpoint)
    }

    pub fn get_transferable_assets_by_outpoint(
        &self,
        outpoint: OutPoint,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_outpoint(self.BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS, outpoint)
    }

    pub fn get_script_key_on_satpoint(
        &mut self,
        satpoint: &SatPoint,
        chain: Chain,
    ) -> anyhow::Result<ScriptKey> {
        if let Some(tx_out) = get_txout_by_outpoint(self.OUTPOINT_TO_ENTRY, &satpoint.outpoint)? {
            Ok(ScriptKey::from_script(&tx_out.script_pubkey, chain))
        } else {
            Err(anyhow::anyhow!(
                "failed to get tx out! error: outpoint {} not found",
                &satpoint.outpoint
            ))
        }
    }
}
