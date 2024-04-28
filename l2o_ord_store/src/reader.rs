use bitcoin::OutPoint;
use bitcoin::Txid;
use l2o_ord::chain::Chain;
use l2o_ord::operation::ProtocolType;
use l2o_ord::sat_point::SatPoint;
use l2o_ord::script_key::ScriptKey;
use l2o_ord::tick::Tick;

use crate::balance::Balance;
use crate::ctx::Context;
use crate::event::Receipt;
use crate::log::TransferableLog;
use crate::table::get_balance;
use crate::table::get_balances;
use crate::table::get_brc21_deposits_holding_balance;
use crate::table::get_token_info;
use crate::table::get_tokens_info;
use crate::table::get_transaction_receipts;
use crate::table::get_transferable_assets_by_account;
use crate::table::get_transferable_assets_by_account_ticker;
use crate::table::get_transferable_assets_by_outpoint;
use crate::table::get_transferable_assets_by_satpoint;
use crate::table::get_txout_by_outpoint;
use crate::token_info::TokenInfo;

impl<'a, 'db, 'txn> Context<'a, 'db, 'txn> {
    pub fn get_balances(
        &self,
        script_key: &ScriptKey,
        ptype: ProtocolType,
    ) -> anyhow::Result<Vec<Balance>> {
        match ptype {
            ProtocolType::BRC20 => get_balances(self.brc20_balances, script_key),
            ProtocolType::BRC21 => get_balances(self.brc21_balances, script_key),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_balance(
        &self,
        script_key: &ScriptKey,
        tick: &Tick,
        ptype: ProtocolType,
    ) -> anyhow::Result<Option<Balance>> {
        match ptype {
            ProtocolType::BRC20 => get_balance(self.brc20_balances, script_key, tick),
            ProtocolType::BRC21 => get_balance(self.brc21_balances, script_key, tick),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_token_info(
        &self,
        tick: &Tick,
        ptype: ProtocolType,
    ) -> anyhow::Result<Option<TokenInfo>> {
        match ptype {
            ProtocolType::BRC20 => get_token_info(self.brc20_token, tick),
            ProtocolType::BRC21 => get_token_info(self.brc21_token, tick),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_tokens_info(&self, ptype: ProtocolType) -> anyhow::Result<Vec<TokenInfo>> {
        match ptype {
            ProtocolType::BRC20 => get_tokens_info(self.brc20_token),
            ProtocolType::BRC21 => get_tokens_info(self.brc21_token),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_transaction_receipts(
        &self,
        txid: &Txid,
        ptype: ProtocolType,
    ) -> anyhow::Result<Option<Vec<Receipt>>> {
        match ptype {
            ProtocolType::BRC20 => get_transaction_receipts(self.brc20_events, txid),
            ProtocolType::BRC21 => get_transaction_receipts(self.brc21_events, txid),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_transferable_assets_by_account(
        &self,
        script: &ScriptKey,
        ptype: ProtocolType,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        match ptype {
            ProtocolType::BRC20 => get_transferable_assets_by_account(
                self.brc20_address_ticker_to_transferable_assets,
                self.brc20_satpoint_to_transferable_assets,
                script,
            ),
            ProtocolType::BRC21 => get_transferable_assets_by_account(
                self.brc21_address_ticker_to_transferable_assets,
                self.brc21_satpoint_to_transferable_assets,
                script,
            ),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_transferable_assets_by_account_ticker(
        &self,
        script: &ScriptKey,
        tick: &Tick,
        ptype: ProtocolType,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        match ptype {
            ProtocolType::BRC20 => get_transferable_assets_by_account_ticker(
                self.brc20_address_ticker_to_transferable_assets,
                self.brc20_satpoint_to_transferable_assets,
                script,
                tick,
            ),
            ProtocolType::BRC21 => get_transferable_assets_by_account_ticker(
                self.brc21_address_ticker_to_transferable_assets,
                self.brc21_satpoint_to_transferable_assets,
                script,
                tick,
            ),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_transferable_assets_by_satpoint(
        &self,
        satpoint: &SatPoint,
        ptype: ProtocolType,
    ) -> anyhow::Result<Option<TransferableLog>> {
        match ptype {
            ProtocolType::BRC20 => get_transferable_assets_by_satpoint(
                self.brc20_satpoint_to_transferable_assets,
                satpoint,
            ),
            ProtocolType::BRC21 => get_transferable_assets_by_satpoint(
                self.brc21_satpoint_to_transferable_assets,
                satpoint,
            ),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_transferable_assets_by_outpoint(
        &self,
        outpoint: OutPoint,
        ptype: ProtocolType,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        match ptype {
            ProtocolType::BRC20 => get_transferable_assets_by_outpoint(
                self.brc20_satpoint_to_transferable_assets,
                outpoint,
            ),
            ProtocolType::BRC21 => get_transferable_assets_by_outpoint(
                self.brc21_satpoint_to_transferable_assets,
                outpoint,
            ),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn get_brc20_balances(&self, script_key: &ScriptKey) -> anyhow::Result<Vec<Balance>> {
        get_balances(self.brc20_balances, script_key)
    }

    pub fn get_brc20_balance(
        &self,
        script_key: &ScriptKey,
        tick: &Tick,
    ) -> anyhow::Result<Option<Balance>> {
        get_balance(self.brc20_balances, script_key, tick)
    }

    pub fn get_brc20_token_info(&self, tick: &Tick) -> anyhow::Result<Option<TokenInfo>> {
        get_token_info(self.brc20_token, tick)
    }

    pub fn get_brc20_tokens_info(&self) -> anyhow::Result<Vec<TokenInfo>> {
        get_tokens_info(self.brc20_token)
    }

    pub fn get_brc20_transaction_receipts(
        &self,
        txid: &Txid,
    ) -> anyhow::Result<Option<Vec<Receipt>>> {
        get_transaction_receipts(self.brc20_events, txid)
    }

    pub fn get_brc20_transferable_assets_by_account(
        &self,
        script: &ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_account(
            self.brc20_address_ticker_to_transferable_assets,
            self.brc20_satpoint_to_transferable_assets,
            script,
        )
    }

    pub fn get_brc20_transferable_assets_by_account_ticker(
        &self,
        script: &ScriptKey,
        tick: &Tick,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_account_ticker(
            self.brc20_address_ticker_to_transferable_assets,
            self.brc20_satpoint_to_transferable_assets,
            script,
            tick,
        )
    }

    pub fn get_brc20_transferable_assets_by_satpoint(
        &self,
        satpoint: &SatPoint,
    ) -> anyhow::Result<Option<TransferableLog>> {
        get_transferable_assets_by_satpoint(self.brc20_satpoint_to_transferable_assets, satpoint)
    }

    pub fn get_brc20_transferable_assets_by_outpoint(
        &self,
        outpoint: OutPoint,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_outpoint(self.brc20_satpoint_to_transferable_assets, outpoint)
    }

    pub fn get_brc21_balances(&self, script_key: &ScriptKey) -> anyhow::Result<Vec<Balance>> {
        get_balances(self.brc21_balances, script_key)
    }

    pub fn get_brc21_balance(
        &self,
        script_key: &ScriptKey,
        tick: &Tick,
    ) -> anyhow::Result<Option<Balance>> {
        get_balance(self.brc21_balances, script_key, tick)
    }

    pub fn get_brc21_token_info(&self, tick: &Tick) -> anyhow::Result<Option<TokenInfo>> {
        get_token_info(self.brc21_token, tick)
    }

    pub fn get_brc21_tokens_info(&self) -> anyhow::Result<Vec<TokenInfo>> {
        get_tokens_info(self.brc21_token)
    }

    pub fn get_brc21_transaction_receipts(
        &self,
        txid: &Txid,
    ) -> anyhow::Result<Option<Vec<Receipt>>> {
        get_transaction_receipts(self.brc21_events, txid)
    }

    pub fn get_brc21_transferable_assets_by_account(
        &self,
        script: &ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_account(
            self.brc21_address_ticker_to_transferable_assets,
            self.brc21_satpoint_to_transferable_assets,
            script,
        )
    }

    pub fn get_brc21_transferable_assets_by_account_ticker(
        &self,
        script: &ScriptKey,
        tick: &Tick,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_account_ticker(
            self.brc21_address_ticker_to_transferable_assets,
            self.brc21_satpoint_to_transferable_assets,
            script,
            tick,
        )
    }

    pub fn get_brc21_transferable_assets_by_satpoint(
        &self,
        satpoint: &SatPoint,
    ) -> anyhow::Result<Option<TransferableLog>> {
        get_transferable_assets_by_satpoint(self.brc21_satpoint_to_transferable_assets, satpoint)
    }

    pub fn get_brc21_transferable_assets_by_outpoint(
        &self,
        outpoint: OutPoint,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        get_transferable_assets_by_outpoint(self.brc21_satpoint_to_transferable_assets, outpoint)
    }

    pub fn get_script_key_on_satpoint(
        &mut self,
        satpoint: &SatPoint,
        chain: Chain,
    ) -> anyhow::Result<ScriptKey> {
        if let Some(tx_out) = get_txout_by_outpoint(self.outpoint_to_entry, &satpoint.outpoint)? {
            Ok(ScriptKey::from_script(&tx_out.script_pubkey, chain))
        } else {
            Err(anyhow::anyhow!(
                "failed to get tx out! error: outpoint {} not found",
                &satpoint.outpoint
            ))
        }
    }

    pub fn get_brc21_deposits_holding_balance(
        &mut self,
        l2id: u64,
        tick: Tick,
    ) -> anyhow::Result<u128> {
        get_brc21_deposits_holding_balance(self.brc21_deposits_holding_balances, l2id, &tick)
    }
}
