use std::sync::Arc;

use bitcoin::block::Header;
use bitcoin::BlockHash;
use bitcoin::OutPoint;
use bitcoin::TxOut;
use bitcoin::Txid;
use bitcoincore_rpc::Client;
use bitcoincore_rpc::RpcApi;
use l2o_ord::height::Height;
use l2o_ord::sat_point::SatPoint;
use l2o_ord::script_key::ScriptKey;
use l2o_ord::tick::Tick;
use redb::ReadableTable;

use crate::balance::Balance;
use crate::entry::Entry;
use crate::event::Receipt;
use crate::log::TransferableLog;
use crate::reorg::ReorgError;
use crate::reorg::MAX_SAVEPOINTS;
use crate::reorg::SAVEPOINT_INTERVAL;
use crate::table::get_balance;
use crate::table::get_balances;
use crate::table::get_token_info;
use crate::table::get_tokens_info;
use crate::table::get_transaction_receipts;
use crate::table::get_transferable_assets_by_account;
use crate::table::get_transferable_assets_by_account_ticker;
use crate::table::get_transferable_assets_by_outpoint;
use crate::table::get_txout_by_outpoint;
use crate::table::BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS;
use crate::table::BRC20_BALANCES;
use crate::table::BRC20_EVENTS;
use crate::table::BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS;
use crate::table::BRC20_TOKEN;
use crate::table::BRC21_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS;
use crate::table::BRC21_BALANCES;
use crate::table::BRC21_EVENTS;
use crate::table::BRC21_SATPOINT_TO_TRANSFERABLE_ASSETS;
use crate::table::BRC21_TOKEN;
use crate::table::HEIGHT_TO_BLOCK_HEADER;
use crate::table::OUTPOINT_TO_ENTRY;
use crate::token_info::TokenInfo;
use crate::wtx::BlockData;

pub trait Rtx {
    fn block_height(&self) -> anyhow::Result<Option<Height>>;
    fn block_count(&self) -> anyhow::Result<u32>;
    fn block_hash(&self, height: Option<u32>) -> anyhow::Result<Option<BlockHash>>;
    fn latest_block(&self) -> anyhow::Result<Option<(Height, BlockHash)>>;
    fn outpoint_to_entry(&self, outpoint: OutPoint) -> anyhow::Result<Option<TxOut>>;
    fn brc20_get_tick_info(&self, name: &Tick) -> anyhow::Result<Option<TokenInfo>>;
    fn brc20_get_all_tick_info(&self) -> anyhow::Result<Vec<TokenInfo>>;
    fn brc20_get_balance_by_address(
        &self,
        tick: &Tick,
        script_key: ScriptKey,
    ) -> anyhow::Result<Option<Balance>>;
    fn brc20_get_all_balance_by_address(
        &self,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<Balance>>;
    fn brc20_transaction_id_to_transaction_receipt(
        &self,
        txid: Txid,
    ) -> anyhow::Result<Option<Vec<Receipt>>>;
    fn brc20_get_tick_transferable_by_address(
        &self,
        tick: &Tick,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>>;
    fn brc20_get_all_transferable_by_address(
        &self,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>>;
    fn brc20_transferable_assets_on_output_with_satpoints(
        &self,
        outpoint: OutPoint,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>>;
    fn brc21_get_tick_info(&self, name: &Tick) -> anyhow::Result<Option<TokenInfo>>;
    fn brc21_get_all_tick_info(&self) -> anyhow::Result<Vec<TokenInfo>>;
    fn brc21_get_balance_by_address(
        &self,
        tick: &Tick,
        script_key: ScriptKey,
    ) -> anyhow::Result<Option<Balance>>;
    fn brc21_get_all_balance_by_address(
        &self,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<Balance>>;
    fn brc21_transaction_id_to_transaction_receipt(
        &self,
        txid: Txid,
    ) -> anyhow::Result<Option<Vec<Receipt>>>;
    fn brc21_get_tick_transferable_by_address(
        &self,
        tick: &Tick,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>>;
    fn brc21_get_all_transferable_by_address(
        &self,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>>;
    fn brc21_transferable_assets_on_output_with_satpoints(
        &self,
        outpoint: OutPoint,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>>;
    fn detect_reorg(
        &self,
        client: Arc<Client>,
        block: &BlockData,
        height: u32,
    ) -> anyhow::Result<()>;
}

impl<'a> Rtx for redb::ReadTransaction<'a> {
    fn block_height(&self) -> anyhow::Result<Option<Height>> {
        Ok(self
            .open_table(HEIGHT_TO_BLOCK_HEADER)?
            .range(0..)?
            .next_back()
            .and_then(|result| result.ok())
            .map(|(height, _header)| Height(height.value())))
    }

    fn block_count(&self) -> anyhow::Result<u32> {
        Ok(self
            .open_table(HEIGHT_TO_BLOCK_HEADER)?
            .range(0..)?
            .next_back()
            .and_then(|result| result.ok())
            .map(|(height, _header)| height.value() + 1)
            .unwrap_or(0))
    }

    fn block_hash(&self, height: Option<u32>) -> anyhow::Result<Option<BlockHash>> {
        let height_to_block_header = self.open_table(HEIGHT_TO_BLOCK_HEADER)?;

        Ok(match height {
            Some(height) => height_to_block_header.get(height)?,
            None => height_to_block_header
                .range(0..)?
                .next_back()
                .transpose()?
                .map(|(_height, header)| header),
        }
        .map(|header| Header::load(*header.value()).block_hash()))
    }

    fn latest_block(&self) -> anyhow::Result<Option<(Height, BlockHash)>> {
        Ok(self
            .open_table(HEIGHT_TO_BLOCK_HEADER)?
            .range(0..)?
            .next_back()
            .and_then(|result| result.ok())
            .map(|(height, hash)| {
                (
                    Height(height.value()),
                    Header::load(*hash.value()).block_hash(),
                )
            }))
    }

    fn outpoint_to_entry(&self, outpoint: OutPoint) -> anyhow::Result<Option<TxOut>> {
        let table = self.open_table(OUTPOINT_TO_ENTRY)?;
        get_txout_by_outpoint(&table, &outpoint)
    }

    fn brc20_get_tick_info(&self, name: &Tick) -> anyhow::Result<Option<TokenInfo>> {
        let table = self.open_table(BRC20_TOKEN)?;
        get_token_info(&table, name)
    }

    fn brc20_get_all_tick_info(&self) -> anyhow::Result<Vec<TokenInfo>> {
        let table = self.open_table(BRC20_TOKEN)?;
        get_tokens_info(&table)
    }

    fn brc20_get_balance_by_address(
        &self,
        tick: &Tick,
        script_key: ScriptKey,
    ) -> anyhow::Result<Option<Balance>> {
        let table = self.open_table(BRC20_BALANCES)?;
        get_balance(&table, &script_key, tick)
    }

    fn brc20_get_all_balance_by_address(
        &self,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<Balance>> {
        let table = self.open_table(BRC20_BALANCES)?;
        get_balances(&table, &script_key)
    }

    fn brc20_transaction_id_to_transaction_receipt(
        &self,
        txid: Txid,
    ) -> anyhow::Result<Option<Vec<Receipt>>> {
        let table = self.open_table(BRC20_EVENTS)?;
        get_transaction_receipts(&table, &txid)
    }

    fn brc20_get_tick_transferable_by_address(
        &self,
        tick: &Tick,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        let address_table =
            self.open_multimap_table(BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS)?;
        let satpoint_table = self.open_table(BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS)?;
        get_transferable_assets_by_account_ticker(
            &address_table,
            &satpoint_table,
            &script_key,
            tick,
        )
    }

    fn brc20_get_all_transferable_by_address(
        &self,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        let address_table =
            self.open_multimap_table(BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS)?;
        let satpoint_table = self.open_table(BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS)?;
        get_transferable_assets_by_account(&address_table, &satpoint_table, &script_key)
    }

    fn brc20_transferable_assets_on_output_with_satpoints(
        &self,
        outpoint: OutPoint,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        let satpoint_to_sequence_number = self.open_table(BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS)?;
        get_transferable_assets_by_outpoint(&satpoint_to_sequence_number, outpoint)
    }

    fn brc21_get_tick_info(&self, name: &Tick) -> anyhow::Result<Option<TokenInfo>> {
        let table = self.open_table(BRC21_TOKEN)?;
        get_token_info(&table, name)
    }

    fn brc21_get_all_tick_info(&self) -> anyhow::Result<Vec<TokenInfo>> {
        let table = self.open_table(BRC21_TOKEN)?;
        get_tokens_info(&table)
    }

    fn brc21_get_balance_by_address(
        &self,
        tick: &Tick,
        script_key: ScriptKey,
    ) -> anyhow::Result<Option<Balance>> {
        let table = self.open_table(BRC21_BALANCES)?;
        get_balance(&table, &script_key, tick)
    }

    fn brc21_get_all_balance_by_address(
        &self,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<Balance>> {
        let table = self.open_table(BRC21_BALANCES)?;
        get_balances(&table, &script_key)
    }

    fn brc21_transaction_id_to_transaction_receipt(
        &self,
        txid: Txid,
    ) -> anyhow::Result<Option<Vec<Receipt>>> {
        let table = self.open_table(BRC21_EVENTS)?;
        get_transaction_receipts(&table, &txid)
    }

    fn brc21_get_tick_transferable_by_address(
        &self,
        tick: &Tick,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        let address_table =
            self.open_multimap_table(BRC21_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS)?;
        let satpoint_table = self.open_table(BRC21_SATPOINT_TO_TRANSFERABLE_ASSETS)?;
        get_transferable_assets_by_account_ticker(
            &address_table,
            &satpoint_table,
            &script_key,
            tick,
        )
    }

    fn brc21_get_all_transferable_by_address(
        &self,
        script_key: ScriptKey,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        let address_table =
            self.open_multimap_table(BRC21_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS)?;
        let satpoint_table = self.open_table(BRC21_SATPOINT_TO_TRANSFERABLE_ASSETS)?;
        get_transferable_assets_by_account(&address_table, &satpoint_table, &script_key)
    }

    fn brc21_transferable_assets_on_output_with_satpoints(
        &self,
        outpoint: OutPoint,
    ) -> anyhow::Result<Vec<(SatPoint, TransferableLog)>> {
        let satpoint_to_sequence_number = self.open_table(BRC21_SATPOINT_TO_TRANSFERABLE_ASSETS)?;
        get_transferable_assets_by_outpoint(&satpoint_to_sequence_number, outpoint)
    }

    fn detect_reorg(
        &self,
        client: Arc<Client>,
        block: &BlockData,
        height: u32,
    ) -> anyhow::Result<()> {
        let bitcoin_prev_blockhash = block.header.prev_blockhash;

        match self.block_hash(height.checked_sub(1))? {
            Some(index_prev_blockhash) if index_prev_blockhash == bitcoin_prev_blockhash => Ok(()),
            Some(index_prev_blockhash) if index_prev_blockhash != bitcoin_prev_blockhash => {
                let max_recoverable_reorg_depth =
                    (MAX_SAVEPOINTS - 1) * SAVEPOINT_INTERVAL + height % SAVEPOINT_INTERVAL;

                for depth in 1..max_recoverable_reorg_depth {
                    let index_block_hash = self.block_hash(height.checked_sub(depth))?;
                    let bitcoin_block_hash = client
                        .get_block_hash(u64::from(height.saturating_sub(depth)))
                        .ok();

                    if index_block_hash == bitcoin_block_hash {
                        return Err(anyhow::anyhow!(ReorgError::Recoverable { height, depth }));
                    }
                }

                Err(anyhow::anyhow!(ReorgError::Unrecoverable))
            }
            _ => Ok(()),
        }
    }
}
