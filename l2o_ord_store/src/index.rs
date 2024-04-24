use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

use bitcoin::block::Header;
use bitcoin::Block;
use bitcoin::Transaction;
use bitcoin::Txid;
use l2o_ord::chain::Chain;
use l2o_ord::error::JSONError;
use l2o_ord::inscription::inscription::Inscription;
use l2o_ord::inscription::inscription_id::InscriptionId;
use l2o_ord::operation::deserialize_brc20;
use l2o_ord::operation::Operation;
use l2o_ord::operation::RawOperation;
use l2o_ord::sat_point::SatPoint;
use redb::Database;
use serde::Deserialize;
use serde::Serialize;

use crate::ctx::ChainContext;
use crate::ctx::Context;
use crate::entry::Entry;
use crate::entry::SatPointValue;
use crate::executor;
use crate::executor::ExecutionMessage;
use crate::executor::Message;
use crate::log::TransferableLog;
use crate::table::get_transferable_assets_by_outpoint;
use crate::table::BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS;
use crate::table::BRC20_BALANCES;
use crate::table::BRC20_EVENTS;
use crate::table::BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS;
use crate::table::BRC20_TOKEN;
use crate::table::HEIGHT_TO_BLOCK_HEADER;
use crate::table::OUTPOINT_TO_ENTRY;

pub struct BlockData {
    pub header: Header,
    pub txdata: Vec<(Transaction, Txid)>,
}

impl From<Block> for BlockData {
    fn from(block: Block) -> Self {
        BlockData {
            header: block.header,
            txdata: block
                .txdata
                .into_iter()
                .map(|transaction| {
                    let txid = transaction.txid();
                    (transaction, txid)
                })
                .collect(),
        }
    }
}

// collect the inscription operation.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InscriptionOp {
    pub txid: Txid,
    pub action: Action,
    pub sequence_number: u32,
    pub inscription_number: Option<i32>,
    pub inscription_id: InscriptionId,
    pub old_satpoint: SatPoint,
    pub new_satpoint: Option<SatPoint>,
}

// the act of marking an inscription.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Action {
    New {
        cursed: bool,
        unbound: bool,
        #[serde(skip)]
        inscription: Inscription,
        #[serde(skip)]
        vindicated: bool,
        #[serde(skip)]
        parent: Option<InscriptionId>,
    },
    Transfer,
}

pub fn deserialize_brc20_operation(
    inscription: &Inscription,
    action: &Action,
) -> anyhow::Result<Operation> {
    let content_body = std::str::from_utf8(inscription.body().ok_or(JSONError::InvalidJson)?)?;
    if content_body.len() < 40 {
        return Err(JSONError::NotBRC20Json.into());
    }

    let content_type = inscription
        .content_type()
        .ok_or(JSONError::InvalidContentType)?;

    if content_type != "text/plain"
        && content_type != "text/plain;charset=utf-8"
        && content_type != "text/plain;charset=UTF-8"
        && content_type != "application/json"
        && !content_type.starts_with("text/plain;")
    {
        return Err(JSONError::UnSupportContentType.into());
    }
    let raw_operation = match deserialize_brc20(content_body) {
        Ok(op) => op,
        Err(e) => {
            return Err(e.into());
        }
    };

    match action {
        Action::New { parent, .. } => match raw_operation {
            RawOperation::Deploy(deploy) => Ok(Operation::Deploy(deploy)),
            RawOperation::Mint(mint) => Ok(Operation::Mint {
                mint,
                parent: *parent,
            }),
            RawOperation::Transfer(transfer) => Ok(Operation::InscribeTransfer(transfer)),
        },
        Action::Transfer => match raw_operation {
            RawOperation::Transfer(transfer) => Ok(Operation::Transfer(transfer)),
            _ => Err(JSONError::NotBRC20Json.into()),
        },
    }
}

pub fn index_block<P: AsRef<Path>>(
    path: P,
    first_brc20_height: u32,
    block: &BlockData,
    operations: HashMap<Txid, Vec<InscriptionOp>>,
) -> anyhow::Result<()> {
    let db = Database::create(path)?;
    let wxn = db.begin_write()?;

    let height_to_block_header = wxn.open_table(HEIGHT_TO_BLOCK_HEADER)?;
    let mut outpoint_to_entry = wxn.open_table(OUTPOINT_TO_ENTRY)?;

    let mut ctx = Context {
        chain_conf: ChainContext {
            blockheight: 0,
            chain: Chain::Regtest,
            blocktime: 0,
        },
        OUTPOINT_TO_ENTRY: &mut outpoint_to_entry,
        BRC20_BALANCES: &mut wxn.open_table(BRC20_BALANCES)?,
        BRC20_TOKEN: &mut wxn.open_table(BRC20_TOKEN)?,
        BRC20_EVENTS: &mut wxn.open_table(BRC20_EVENTS)?,
        BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS: &mut wxn
            .open_table(BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS)?,
        BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS: &mut wxn
            .open_multimap_table(BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS)?,
    };

    let context = &mut ctx;

    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter() {
        // skip coinbase transaction.
        if tx
            .input
            .first()
            .is_some_and(|tx_in| tx_in.previous_output.is_null())
        {
            continue;
        }

        // index inscription operations.
        if let Some(tx_operations) = operations.get(txid) {
            // save all transaction operations to ord database.

            // Resolve and execute messages.
            let messages = {
                let operations: &[InscriptionOp] = tx_operations;
                tracing::debug!(
                    "Resolve Manager indexed transaction {}, operations size: {}, data: {:?}",
                    tx.txid(),
                    operations.len(),
                    operations
                );
                let mut messages = Vec::new();
                let mut operation_iter = operations.iter().peekable();

                for input in &tx.input {
                    // "operations" is a list of all the operations in the current block, and they
                    // are ordered. We just need to find the operation
                    // corresponding to the current transaction here.
                    while let Some(operation) = operation_iter.peek() {
                        if operation.old_satpoint.outpoint != input.previous_output {
                            break;
                        }
                        let operation = operation_iter.next().unwrap();

                        // Parse BRC20 message through inscription operation.
                        if context.chain_conf.blockheight >= first_brc20_height {
                            let satpoint_to_transfer_assets: HashMap<
                                SatPointValue,
                                TransferableLog,
                            > = get_transferable_assets_by_outpoint(
                                context.BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS,
                                input.previous_output,
                            )?
                            .into_iter()
                            .map(|(satpoint, asset)| (satpoint.store(), asset))
                            .collect();

                            if let Some(msg) =
                                Message::resolve(operation, satpoint_to_transfer_assets)?
                            {
                                tracing::debug!(
                                    "BRC20 resolved the message from {:?}, msg {:?}",
                                    operation,
                                    msg
                                );
                                messages.push(msg);
                                continue;
                            }
                        }
                    }
                }
                Ok::<Vec<executor::Message>, anyhow::Error>(messages)
            }?;

            {
                let msgs: &[Message] = &messages;
                let mut receipts = vec![];
                // execute message
                for msg in msgs {
                    let msg =
                        ExecutionMessage::from_message(context, msg, context.chain_conf.chain)?;
                    let receipt = ExecutionMessage::execute(context, &msg)?;
                    receipts.push(receipt);
                }

                context
                    .save_transaction_receipts(txid, &receipts)
                    .map_err(|e| {
                        anyhow::anyhow!("failed to add transaction receipt to state! error: {e}")
                    })?;

                let _brc20_inscriptions = receipts
                    .into_iter()
                    .map(|receipt| receipt.inscription_id)
                    .collect::<HashSet<_>>();

                // for inscription_id in brc20_inscriptions {
                //     context
                //         .add_inscription_attributes(&inscription_id, CollectionKind::BRC20)
                //         .map_err(|e| {
                //             anyhow::anyhow!("failed to add inscription attributes to state!
                // error: {e}")         })?;
                // }
                Ok::<(), anyhow::Error>(())
            }?;
        }
    }

    Ok(())
}
