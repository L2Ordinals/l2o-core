use bitcoin::Txid;
use l2o_ord::operation::ProtocolType;
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
        ptype: ProtocolType,
    ) -> anyhow::Result<()> {
        match ptype {
            ProtocolType::BRC20 => {
                update_token_balance(self.brc20_balances, script_key, new_balance)
            }
            ProtocolType::BRC21 => {
                update_token_balance(self.brc21_balances, script_key, new_balance)
            }
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn insert_token_info(
        &mut self,
        tick: &Tick,
        new_info: &TokenInfo,
        ptype: ProtocolType,
    ) -> anyhow::Result<()> {
        match ptype {
            ProtocolType::BRC20 => insert_token_info(self.brc20_token, tick, new_info),
            ProtocolType::BRC21 => insert_token_info(self.brc21_token, tick, new_info),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn update_mint_token_info(
        &mut self,
        tick: &Tick,
        minted_amt: u128,
        minted_block_number: u32,
        ptype: ProtocolType,
    ) -> anyhow::Result<()> {
        match ptype {
            ProtocolType::BRC20 => {
                update_mint_token_info(self.brc20_token, tick, minted_amt, minted_block_number)
            }
            ProtocolType::BRC21 => {
                update_mint_token_info(self.brc21_token, tick, minted_amt, minted_block_number)
            }
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn update_burned_token_info(
        &mut self,
        tick: &Tick,
        burned_amt: u128,
        ptype: ProtocolType,
    ) -> anyhow::Result<()> {
        match ptype {
            ProtocolType::BRC20 => update_burned_token_info(self.brc20_token, tick, burned_amt),
            ProtocolType::BRC21 => update_burned_token_info(self.brc21_token, tick, burned_amt),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn save_transaction_receipts(
        &mut self,
        txid: &Txid,
        receipt: &[Receipt],
        ptype: ProtocolType,
    ) -> anyhow::Result<()> {
        match ptype {
            ProtocolType::BRC20 => save_transaction_receipts(self.brc20_events, txid, receipt),
            ProtocolType::BRC21 => save_transaction_receipts(self.brc21_events, txid, receipt),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn insert_transferable_asset(
        &mut self,
        satpoint: SatPoint,
        transferable_asset: &TransferableLog,
        ptype: ProtocolType,
    ) -> anyhow::Result<()> {
        match ptype {
            ProtocolType::BRC20 => insert_transferable_asset(
                self.brc20_satpoint_to_transferable_assets,
                self.brc20_address_ticker_to_transferable_assets,
                satpoint,
                transferable_asset,
            ),
            ProtocolType::BRC21 => insert_transferable_asset(
                self.brc21_satpoint_to_transferable_assets,
                self.brc21_address_ticker_to_transferable_assets,
                satpoint,
                transferable_asset,
            ),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn remove_transferable_asset(
        &mut self,
        satpoint: SatPoint,
        ptype: ProtocolType,
    ) -> anyhow::Result<()> {
        match ptype {
            ProtocolType::BRC20 => remove_transferable_asset(
                self.brc20_satpoint_to_transferable_assets,
                self.brc20_address_ticker_to_transferable_assets,
                satpoint,
            ),
            ProtocolType::BRC21 => remove_transferable_asset(
                self.brc21_satpoint_to_transferable_assets,
                self.brc21_address_ticker_to_transferable_assets,
                satpoint,
            ),
            ProtocolType::L2OA => unreachable!(),
        }
    }

    pub fn update_brc20_token_balance(
        &mut self,
        script_key: &ScriptKey,
        new_balance: Balance,
    ) -> anyhow::Result<()> {
        update_token_balance(self.brc20_balances, script_key, new_balance)
    }

    pub fn insert_brc20_token_info(
        &mut self,
        tick: &Tick,
        new_info: &TokenInfo,
    ) -> anyhow::Result<()> {
        insert_token_info(self.brc20_token, tick, new_info)
    }

    pub fn update_brc20_mint_token_info(
        &mut self,
        tick: &Tick,
        minted_amt: u128,
        minted_block_number: u32,
    ) -> anyhow::Result<()> {
        update_mint_token_info(self.brc20_token, tick, minted_amt, minted_block_number)
    }

    pub fn update_brc20_burned_token_info(
        &mut self,
        tick: &Tick,
        burned_amt: u128,
    ) -> anyhow::Result<()> {
        update_burned_token_info(self.brc20_token, tick, burned_amt)
    }

    pub fn save_brc20_transaction_receipts(
        &mut self,
        txid: &Txid,
        receipt: &[Receipt],
    ) -> anyhow::Result<()> {
        save_transaction_receipts(self.brc20_events, txid, receipt)
    }

    pub fn insert_brc20_transferable_asset(
        &mut self,
        satpoint: SatPoint,
        transferable_asset: &TransferableLog,
    ) -> anyhow::Result<()> {
        insert_transferable_asset(
            self.brc20_satpoint_to_transferable_assets,
            self.brc20_address_ticker_to_transferable_assets,
            satpoint,
            transferable_asset,
        )
    }

    pub fn remove_brc20_transferable_asset(&mut self, satpoint: SatPoint) -> anyhow::Result<()> {
        remove_transferable_asset(
            self.brc20_satpoint_to_transferable_assets,
            self.brc20_address_ticker_to_transferable_assets,
            satpoint,
        )
    }

    pub fn update_brc21_token_balance(
        &mut self,
        script_key: &ScriptKey,
        new_balance: Balance,
    ) -> anyhow::Result<()> {
        update_token_balance(self.brc21_balances, script_key, new_balance)
    }

    pub fn insert_brc21_token_info(
        &mut self,
        tick: &Tick,
        new_info: &TokenInfo,
    ) -> anyhow::Result<()> {
        insert_token_info(self.brc21_token, tick, new_info)
    }

    pub fn update_brc21_mint_token_info(
        &mut self,
        tick: &Tick,
        minted_amt: u128,
        minted_block_number: u32,
    ) -> anyhow::Result<()> {
        update_mint_token_info(self.brc21_token, tick, minted_amt, minted_block_number)
    }

    pub fn update_brc21_burned_token_info(
        &mut self,
        tick: &Tick,
        burned_amt: u128,
    ) -> anyhow::Result<()> {
        update_burned_token_info(self.brc21_token, tick, burned_amt)
    }

    pub fn save_brc21_transaction_receipts(
        &mut self,
        txid: &Txid,
        receipt: &[Receipt],
    ) -> anyhow::Result<()> {
        save_transaction_receipts(self.brc21_events, txid, receipt)
    }

    pub fn insert_brc21_transferable_asset(
        &mut self,
        satpoint: SatPoint,
        transferable_asset: &TransferableLog,
    ) -> anyhow::Result<()> {
        insert_transferable_asset(
            self.brc21_satpoint_to_transferable_assets,
            self.brc21_address_ticker_to_transferable_assets,
            satpoint,
            transferable_asset,
        )
    }

    pub fn remove_brc21_transferable_asset(&mut self, satpoint: SatPoint) -> anyhow::Result<()> {
        remove_transferable_asset(
            self.brc21_satpoint_to_transferable_assets,
            self.brc21_address_ticker_to_transferable_assets,
            satpoint,
        )
    }
}
