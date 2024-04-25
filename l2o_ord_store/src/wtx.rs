use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

use bitcoin::block::Header;
use bitcoin::Amount;
use bitcoin::Block;
use bitcoin::OutPoint;
use bitcoin::Transaction;
use bitcoin::TxOut;
use bitcoin::Txid;
use l2o_ord::chain::Chain;
use l2o_ord::error::JSONError;
use l2o_ord::height::Height;
use l2o_ord::inscription::envelope::ParsedEnvelope;
use l2o_ord::inscription::inscription::Inscription;
use l2o_ord::inscription::inscription_id::InscriptionId;
use l2o_ord::operation::deserialize_brc20;
use l2o_ord::operation::Operation;
use l2o_ord::operation::RawOperation;
use l2o_ord::sat_point::SatPoint;
use redb::ReadableTable;
use serde::Deserialize;
use serde::Serialize;

use crate::charm::Charm;
use crate::ctx::ChainContext;
use crate::ctx::Context;
use crate::entry::Entry;
use crate::entry::InscriptionEntry;
use crate::entry::SatPointValue;
use crate::executor;
use crate::executor::ExecutionMessage;
use crate::executor::Message;
use crate::log::TransferableLog;
use crate::lru::SimpleLru;
use crate::table::get_transferable_assets_by_outpoint;
use crate::table::get_txout_by_outpoint;
use crate::table::inscriptions_on_output;
use crate::table::BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS;
use crate::table::BRC20_BALANCES;
use crate::table::BRC20_EVENTS;
use crate::table::BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS;
use crate::table::BRC20_TOKEN;
use crate::table::HEIGHT_TO_BLOCK_HEADER;
use crate::table::HEIGHT_TO_LAST_SEQUENCE_NUMBER;
use crate::table::INSCRIPTION_ID_TO_SEQUENCE_NUMBER;
use crate::table::INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER;
use crate::table::OUTPOINT_TO_ENTRY;
use crate::table::OUTPOINT_TO_SAT_RANGES;
use crate::table::SATPOINT_TO_SEQUENCE_NUMBER;
use crate::table::SAT_TO_SATPOINT;
use crate::table::SAT_TO_SEQUENCE_NUMBER;
use crate::table::SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY;
use crate::table::SEQUENCE_NUMBER_TO_SATPOINT;

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

#[derive(Debug, PartialEq, Copy, Clone)]
enum Curse {
    DuplicateField,
    IncompleteField,
    NotAtOffsetZero,
    NotInFirstInput,
    Pointer,
    Pushnum,
    Reinscription,
    Stutter,
    UnrecognizedEvenField,
}

#[derive(Debug, Clone)]
pub struct Flotsam {
    txid: Txid,
    inscription_id: InscriptionId,
    offset: u64,
    old_satpoint: SatPoint,
    origin: Origin,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
enum Origin {
    New {
        cursed: bool,
        fee: u64,
        hidden: bool,
        parent: Option<InscriptionId>,
        pointer: Option<u64>,
        reinscription: bool,
        unbound: bool,
        inscription: Inscription,
        vindicated: bool,
    },
    Old,
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

pub trait Wtx {
    fn index_block(
        &mut self,
        block: &BlockData,
        sender: &SyncSender<OutPoint>,
        receiver: &Receiver<TxOut>,
    ) -> anyhow::Result<()>;

    fn index_envelopes<'a, 'db, 'txn>(
        &self,
        tx: &Transaction,
        txid: Txid,
        ctx: &mut Context<'a, 'db, 'txn>,
        sender: SyncSender<OutPoint>,
        receiver: &Receiver<TxOut>,
        tx_out_cache: &mut SimpleLru<OutPoint, TxOut>,
    ) -> anyhow::Result<()>;

    // fn update_inscription_location<'a, 'db, 'txn>(
    //     &mut self,
    //     input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    //     flotsam: Flotsam,
    //     ctx: &mut Context<'a, 'db, 'txn>,
    //     new_satpoint: SatPoint,
    // ) -> anyhow::Result<()>;
}

impl<'a> Wtx for redb::WriteTransaction<'a> {
    fn index_block(
        &mut self,
        block: &BlockData,
        sender: &SyncSender<OutPoint>,
        receiver: &Receiver<TxOut>,
    ) -> anyhow::Result<()> {
        // TODO: detect reorg
        let first_brc20_height = 0;

        let operations = HashMap::<Txid, Vec<InscriptionOp>>::new();
        let mut tx_out_cache = SimpleLru::<OutPoint, TxOut>::new(1000);

        {
            let outpoint_to_entry = self.open_table(OUTPOINT_TO_ENTRY)?;

            let fetching_outputs_count = AtomicUsize::new(0);
            let total_outputs_count = AtomicUsize::new(0);
            let cache_outputs_count = AtomicUsize::new(0);
            let miss_outputs_count = AtomicUsize::new(0);
            let meet_outputs_count = AtomicUsize::new(0);
            // Send all missing input outpoints to be fetched right away
            let txids = block
                .txdata
                .iter()
                .map(|(_, txid)| txid)
                .collect::<HashSet<_>>();
            use rayon::prelude::*;
            let tx_outs = block
                .txdata
                .par_iter()
                .flat_map(|(tx, _)| tx.input.par_iter())
                .filter_map(|input| {
                    total_outputs_count.fetch_add(1, Ordering::Relaxed);
                    let prev_output = input.previous_output;
                    // We don't need coinbase input value
                    if prev_output.is_null() {
                        None
                    } else if txids.contains(&prev_output.txid) {
                        meet_outputs_count.fetch_add(1, Ordering::Relaxed);
                        None
                    } else if tx_out_cache.contains(&prev_output) {
                        cache_outputs_count.fetch_add(1, Ordering::Relaxed);
                        None
                    } else if let Some(txout) =
                        get_txout_by_outpoint(&outpoint_to_entry, &prev_output).unwrap()
                    {
                        miss_outputs_count.fetch_add(1, Ordering::Relaxed);
                        Some((prev_output, Some(txout)))
                    } else {
                        fetching_outputs_count.fetch_add(1, Ordering::Relaxed);
                        Some((prev_output, None))
                    }
                })
                .collect::<Vec<_>>();

            for (out_point, value) in tx_outs.into_iter() {
                if let Some(tx_out) = value {
                    tx_out_cache.insert(out_point, tx_out);
                } else {
                    sender.send(out_point).unwrap();
                }
            }
        }

        let mut ctx = Context {
            chain_conf: ChainContext {
                blockheight: 0,
                chain: Chain::Regtest,
                blocktime: 0,
            },
            height_to_block_header: &mut self.open_table(HEIGHT_TO_BLOCK_HEADER)?,
            height_to_last_sequence_number: &mut self.open_table(HEIGHT_TO_LAST_SEQUENCE_NUMBER)?,

            sat_to_satpoint: &mut self.open_table(SAT_TO_SATPOINT)?,
            sat_to_sequence_number: &mut self.open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?,
            satpoint_to_sequence_number: &mut self
                .open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?,

            outpoint_to_entry: &mut self.open_table(OUTPOINT_TO_ENTRY)?,
            outpoint_to_sat_ranges: &mut self.open_table(OUTPOINT_TO_SAT_RANGES)?,

            inscription_id_to_sequence_number: &mut self
                .open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?,
            inscription_number_to_sequence_number: &mut self
                .open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?,
            sequence_number_to_inscription_entry: &mut self
                .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?,
            sequence_number_to_satpoint: &mut self.open_table(SEQUENCE_NUMBER_TO_SATPOINT)?,

            brc20_balances: &mut self.open_table(BRC20_BALANCES)?,
            brc20_token: &mut self.open_table(BRC20_TOKEN)?,
            brc20_events: &mut self.open_table(BRC20_EVENTS)?,
            brc20_satpoint_to_transferable_assets: &mut self
                .open_table(BRC20_SATPOINT_TO_TRANSFERABLE_ASSETS)?,
            brc20_address_ticker_to_transferable_assets: &mut self
                .open_multimap_table(BRC20_ADDRESS_TICKER_TO_TRANSFERABLE_ASSETS)?,
        };

        let ctx_mut = &mut ctx;

        for (tx, txid) in block.txdata.iter().skip(1).chain(block.txdata.first()) {
            self.index_envelopes(
                tx,
                *txid,
                ctx_mut,
                sender.clone(),
                &receiver,
                &mut tx_out_cache,
            )?;
        }

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
                        // "operations" is a list of all the operations in the current block, and
                        // they are ordered. We just need to find the
                        // operation corresponding to the current
                        // transaction here.
                        while let Some(operation) = operation_iter.peek() {
                            if operation.old_satpoint.outpoint != input.previous_output {
                                break;
                            }
                            let operation = operation_iter.next().unwrap();

                            // Parse BRC20 message through inscription operation.
                            if ctx_mut.chain_conf.blockheight >= first_brc20_height {
                                let satpoint_to_transfer_assets: HashMap<
                                    SatPointValue,
                                    TransferableLog,
                                > = get_transferable_assets_by_outpoint(
                                    ctx_mut.brc20_satpoint_to_transferable_assets,
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
                            ExecutionMessage::from_message(ctx_mut, msg, ctx_mut.chain_conf.chain)?;
                        let receipt = ExecutionMessage::execute(ctx_mut, &msg)?;
                        receipts.push(receipt);
                    }

                    ctx_mut
                        .save_transaction_receipts(txid, &receipts)
                        .map_err(|e| {
                            anyhow::anyhow!(
                                "failed to add transaction receipt to state! error: {e}"
                            )
                        })?;

                    // let _brc20_inscriptions = receipts
                    //     .into_iter()
                    //     .map(|receipt| receipt.inscription_id)
                    //     .collect::<HashSet<_>>();

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

    fn index_envelopes(
        &self,
        tx: &Transaction,
        txid: Txid,
        ctx: &mut Context,
        _sender: SyncSender<OutPoint>,
        receiver: &Receiver<TxOut>,
        tx_out_cache: &mut SimpleLru<OutPoint, TxOut>,
    ) -> anyhow::Result<()> {
        let mut floating_inscriptions = Vec::new();
        let mut id_counter = 0;
        let mut inscribed_offsets = BTreeMap::new();
        let mut total_input_value = 0;
        let mut flotsam = vec![];
        let total_output_value = tx.output.iter().map(|txout| txout.value).sum::<Amount>();

        let envelopes = ParsedEnvelope::from_transaction(tx);
        let _inscriptions = !envelopes.is_empty();
        let mut envelopes = envelopes.into_iter().peekable();

        for (input_index, tx_in) in tx.input.iter().enumerate() {
            // skip subsidy since no inscriptions possible
            if tx_in.previous_output.is_null() {
                total_input_value += Height(ctx.chain_conf.blockheight).subsidy();
                continue;
            }
            // find existing inscriptions on input (transfers of inscriptions)
            for (old_satpoint, inscription_id) in inscriptions_on_output(
                ctx.satpoint_to_sequence_number,
                ctx.sequence_number_to_inscription_entry,
                tx_in.previous_output,
            )? {
                let offset = total_input_value + old_satpoint.offset;
                floating_inscriptions.push(Flotsam {
                    txid,
                    offset,
                    inscription_id,
                    old_satpoint,
                    origin: Origin::Old,
                });

                inscribed_offsets
                    .entry(offset)
                    .or_insert((inscription_id, 0))
                    .1 += 1;
            }

            let offset = total_input_value;

            // multi-level cache for UTXO set to get to the input amount
            let current_input_value = if let Some(tx_out) = tx_out_cache.get(&tx_in.previous_output)
            {
                tx_out.value
            } else {
                let tx_out = receiver.recv().map_err(|_| {
                    anyhow::anyhow!(
                        "failed to get transaction for {}",
                        tx_in.previous_output.txid
                    )
                })?;
                // received new tx out from chain node, add it to new_outpoints first and
                // persist it in db later.
                tx_out_cache.insert(tx_in.previous_output, tx_out.clone());
                tx_out.value
            };

            total_input_value += current_input_value.to_sat();

            // go through all inscriptions in this input
            while let Some(inscription) = envelopes.peek() {
                if inscription.input != u32::try_from(input_index).unwrap() {
                    break;
                }

                let inscription_id = InscriptionId {
                    txid,
                    index: id_counter,
                };

                let curse = if inscription.payload.unrecognized_even_field {
                    Some(Curse::UnrecognizedEvenField)
                } else if inscription.payload.duplicate_field {
                    Some(Curse::DuplicateField)
                } else if inscription.payload.incomplete_field {
                    Some(Curse::IncompleteField)
                } else if inscription.input != 0 {
                    Some(Curse::NotInFirstInput)
                } else if inscription.offset != 0 {
                    Some(Curse::NotAtOffsetZero)
                } else if inscription.payload.pointer.is_some() {
                    Some(Curse::Pointer)
                } else if inscription.pushnum {
                    Some(Curse::Pushnum)
                } else if inscription.stutter {
                    Some(Curse::Stutter)
                } else if let Some((id, count)) = inscribed_offsets.get(&offset) {
                    if *count > 1 {
                        Some(Curse::Reinscription)
                    } else {
                        let initial_inscription_sequence_number = ctx
                            .inscription_id_to_sequence_number
                            .get(id.store())?
                            .unwrap()
                            .value();

                        let entry = InscriptionEntry::load(
                            ctx.sequence_number_to_inscription_entry
                                .get(initial_inscription_sequence_number)?
                                .unwrap()
                                .value(),
                        );

                        let initial_inscription_was_cursed_or_vindicated =
                            entry.inscription_number < 0 || Charm::Vindicated.is_set(entry.charms);

                        if initial_inscription_was_cursed_or_vindicated {
                            None
                        } else {
                            Some(Curse::Reinscription)
                        }
                    }
                } else {
                    None
                };

                let unbound = current_input_value.to_sat() == 0
                    || curse == Some(Curse::UnrecognizedEvenField)
                    || inscription.payload.unrecognized_even_field;

                let offset = inscription
                    .payload
                    .pointer()
                    .filter(|&pointer| pointer < total_output_value.to_sat())
                    .unwrap_or(offset);

                floating_inscriptions.push(Flotsam {
                    txid,
                    inscription_id,
                    offset,
                    old_satpoint: SatPoint {
                        outpoint: tx_in.previous_output,
                        offset: 0,
                    },
                    origin: Origin::New {
                        cursed: curse.is_some(),
                        fee: 0,
                        hidden: inscription.payload.hidden(),
                        parent: inscription.payload.parent(),
                        pointer: inscription.payload.pointer(),
                        reinscription: inscribed_offsets.get(&offset).is_some(),
                        unbound,
                        inscription: inscription.payload.clone(),
                        vindicated: curse.is_some(),
                    },
                });

                inscribed_offsets
                    .entry(offset)
                    .or_insert((inscription_id, 0))
                    .1 += 1;

                envelopes.next();
                id_counter += 1;
            }
        }

        let potential_parents = floating_inscriptions
            .iter()
            .map(|flotsam| flotsam.inscription_id)
            .collect::<HashSet<InscriptionId>>();

        for flotsam in &mut floating_inscriptions {
            if let Flotsam {
                origin: Origin::New { parent, .. },
                ..
            } = flotsam
            {
                if let Some(purported_parent) = parent {
                    if !potential_parents.contains(purported_parent) {
                        *parent = None;
                    }
                }
            }
        }

        // still have to normalize over inscription size
        for flotsam in &mut floating_inscriptions {
            if let Flotsam {
                origin: Origin::New { ref mut fee, .. },
                ..
            } = flotsam
            {
                *fee = (total_input_value - total_output_value.to_sat()) / u64::from(id_counter);
            }
        }

        let is_coinbase = tx
            .input
            .first()
            .map(|tx_in| tx_in.previous_output.is_null())
            .unwrap_or_default();

        if is_coinbase {
            floating_inscriptions.append(&mut flotsam);
        }

        floating_inscriptions.sort_by_key(|flotsam| flotsam.offset);
        let mut inscriptions = floating_inscriptions.into_iter().peekable();

        let mut range_to_vout = BTreeMap::new();
        let mut new_locations = Vec::new();
        let mut output_value = 0;
        for (vout, tx_out) in tx.output.iter().enumerate() {
            let end = output_value + tx_out.value.to_sat();

            while let Some(flotsam) = inscriptions.peek() {
                if flotsam.offset >= end {
                    break;
                }

                let new_satpoint = SatPoint {
                    outpoint: OutPoint {
                        txid,
                        vout: vout.try_into().unwrap(),
                    },
                    offset: flotsam.offset - output_value,
                };

                new_locations.push((new_satpoint, inscriptions.next().unwrap()));
            }

            range_to_vout.insert((output_value, end), vout.try_into().unwrap());

            output_value = end;

            tx_out_cache.insert(
                OutPoint {
                    vout: vout.try_into().unwrap(),
                    txid,
                },
                tx_out.clone(),
            );
        }

        for (new_satpoint, mut flotsam) in new_locations.into_iter() {
            let _new_satpoint = match flotsam.origin {
                Origin::New {
                    pointer: Some(pointer),
                    ..
                } if pointer < output_value => {
                    match range_to_vout.iter().find_map(|((start, end), vout)| {
                        (pointer >= *start && pointer < *end).then(|| (vout, pointer - start))
                    }) {
                        Some((vout, offset)) => {
                            flotsam.offset = pointer;
                            SatPoint {
                                outpoint: OutPoint { txid, vout: *vout },
                                offset,
                            }
                        }
                        _ => new_satpoint,
                    }
                }
                _ => new_satpoint,
            };

            // self.update_inscription_location(input_sat_ranges, flotsam, ctx,
            // new_satpoint)?;
        }
        Ok(())
        //
        // if is_coinbase {
        //     for flotsam in inscriptions {
        //         let new_satpoint = SatPoint {
        //             outpoint: OutPoint::null(),
        //             offset: self.lost_sats + flotsam.offset - output_value,
        //         };
        //         self.update_inscription_location(input_sat_ranges, flotsam,
        // ctx, new_satpoint)?;     }
        //     self.lost_sats += self.reward - output_value;
        //     Ok(())
        // } else {
        //     self.flotsam.extend(inscriptions.map(|flotsam| Flotsam {
        //         offset: self.reward + flotsam.offset - output_value,
        //         ..flotsam
        //     }));
        //     self.reward += total_input_value - output_value;
        //     Ok(())
        // }
    }

    // fn update_inscription_location(
    //     &mut self,
    //     input_sat_ranges: Option<&VecDeque<(u64, u64)>>,
    //     flotsam: Flotsam,
    //     ctx: &mut Context,
    //     new_satpoint: SatPoint,
    // ) -> anyhow::Result<()> { let inscription_id = flotsam.inscription_id; let
    //   (unbound, sequence_number) = match flotsam.origin { Origin::Old => {
    //   ctx.satpoint_to_sequence_number
    //   .remove_all(&flotsam.old_satpoint.store())?;
    //
    //             (
    //                 false,
    //                 ctx.inscription_id_to_sequence_number
    //                     .get(&inscription_id.store())?
    //                     .unwrap()
    //                     .value(),
    //             )
    //         }
    //         Origin::New {
    //             cursed,
    //             fee,
    //             hidden,
    //             parent,
    //             pointer: _,
    //             reinscription,
    //             unbound,
    //             inscription: _,
    //             vindicated,
    //         } => {
    //             let inscription_number = if cursed {
    //                 let number: i32 =
    // self.cursed_inscription_count.try_into().unwrap();
    // self.cursed_inscription_count += 1;
    //
    //                 // because cursed numbers start at -1
    //                 -(number + 1)
    //             } else {
    //                 let number: i32 =
    // self.blessed_inscription_count.try_into().unwrap();
    // self.blessed_inscription_count += 1;
    //
    //                 number
    //             };
    //
    //             let sequence_number = self.next_sequence_number;
    //             self.next_sequence_number += 1;
    //
    //             self.inscription_number_to_sequence_number
    //                 .insert(inscription_number, sequence_number)?;
    //
    //             let sat = if unbound {
    //                 None
    //             } else {
    //                 Self::calculate_sat(input_sat_ranges, flotsam.offset)
    //             };
    //
    //             let mut charms = 0;
    //
    //             if cursed {
    //                 Charm::Cursed.set(&mut charms);
    //             }
    //
    //             if reinscription {
    //                 Charm::Reinscription.set(&mut charms);
    //             }
    //
    //             if let Some(sat) = sat {
    //                 if sat.nineball() {
    //                     Charm::Nineball.set(&mut charms);
    //                 }
    //
    //                 if sat.coin() {
    //                     Charm::Coin.set(&mut charms);
    //                 }
    //
    //                 match sat.rarity() {
    //                     Rarity::Common | Rarity::Mythic => {}
    //                     Rarity::Uncommon => Charm::Uncommon.set(&mut charms),
    //                     Rarity::Rare => Charm::Rare.set(&mut charms),
    //                     Rarity::Epic => Charm::Epic.set(&mut charms),
    //                     Rarity::Legendary => Charm::Legendary.set(&mut charms),
    //                 }
    //             }
    //
    //             if new_satpoint.outpoint == OutPoint::null() {
    //                 Charm::Lost.set(&mut charms);
    //             }
    //
    //             if unbound {
    //                 Charm::Unbound.set(&mut charms);
    //             }
    //
    //             if vindicated {
    //                 Charm::Vindicated.set(&mut charms);
    //             }
    //
    //             if let Some(Sat(n)) = sat {
    //                 ctx.sat_to_sequence_number.insert(&n, &sequence_number)?;
    //             }
    //
    //             let parent = match parent {
    //                 Some(parent_id) => {
    //                     let parent_sequence_number = ctx
    //                         .inscription_id_to_sequence_number
    //                         .get(&parent_id.store())?
    //                         .unwrap()
    //                         .value();
    //                     ctx.sequence_number_to_children
    //                         .insert(parent_sequence_number, sequence_number)?;
    //
    //                     Some(parent_sequence_number)
    //                 }
    //                 None => None,
    //             };
    //
    //             ctx.sequence_number_to_entry.insert(
    //                 sequence_number,
    //                 &InscriptionEntry {
    //                     charms,
    //                     fee,
    //                     height: self.height,
    //                     id: inscription_id,
    //                     inscription_number,
    //                     parent,
    //                     sat,
    //                     sequence_number,
    //                     timestamp: self.timestamp,
    //                 }
    //                 .store(),
    //             )?;
    //
    //             self.id_to_sequence_number
    //                 .insert(&inscription_id.store(), sequence_number)?;
    //
    //             if !hidden {
    //                 self.home_inscriptions
    //                     .insert(&sequence_number, inscription_id.store())?;
    //
    //                 if self.home_inscription_count == 100 {
    //                     self.home_inscriptions.pop_first()?;
    //                 } else {
    //                     self.home_inscription_count += 1;
    //                 }
    //             }
    //
    //             (unbound, sequence_number)
    //         }
    //     };
    //
    //     let satpoint = if unbound {
    //         let new_unbound_satpoint = SatPoint {
    //             outpoint: unbound_outpoint(),
    //             offset: self.unbound_inscriptions,
    //         };
    //         self.unbound_inscriptions += 1;
    //         new_unbound_satpoint.store()
    //     } else {
    //         new_satpoint.store()
    //     };
    //
    //     self.operations
    //         .entry(flotsam.txid)
    //         .or_default()
    //         .push(InscriptionOp {
    //             txid: flotsam.txid,
    //             sequence_number,
    //             inscription_number: ctx
    //                 .sequence_number_to_inscription_entry
    //                 .get(sequence_number)?
    //                 .map(|entry|
    // InscriptionEntry::load(entry.value()).inscription_number),
    // inscription_id: flotsam.inscription_id,             action: match
    // flotsam.origin {                 Origin::Old => Action::Transfer,
    //                 Origin::New {
    //                     cursed,
    //                     fee: _,
    //                     hidden: _,
    //                     pointer: _,
    //                     reinscription: _,
    //                     unbound,
    //                     parent,
    //                     inscription,
    //                     vindicated,
    //                 } => Action::New {
    //                     cursed,
    //                     unbound,
    //                     vindicated,
    //                     parent,
    //                     inscription,
    //                 },
    //             },
    //             old_satpoint: flotsam.old_satpoint,
    //             new_satpoint: Some(Entry::load(satpoint)),
    //         });
    //
    //     ctx.satpoint_to_sequence_number
    //         .insert(&satpoint, sequence_number)?;
    //     ctx.sequence_number_to_satpoint
    //         .insert(sequence_number, &satpoint)?;
    //
    //     Ok(())
    // }
}
