use std::str::FromStr;

use ark_bn254::Bn254;
use ark_bn254::Fr;
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;
use ark_snark::SNARK;
use bigdecimal::num_bigint::Sign;
use bitcoin::Address;
use bitcoin::Txid;
use bitcoincore_rpc::RpcApi;
use l2o_common::common::data::hash::Hash256;
use l2o_crypto::hash::hash_functions::blake3::Blake3Hasher;
use l2o_crypto::hash::hash_functions::keccak256::Keccak256Hasher;
use l2o_crypto::hash::hash_functions::poseidon_goldilocks::PoseidonHasher;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::signature::schnorr::verify_sig;
use l2o_ord::chain::Chain;
use l2o_ord::decimal::Decimal;
use l2o_ord::error::BRC2XError;
use l2o_ord::error::Error;
use l2o_ord::hasher::L2OBlockHasher;
use l2o_ord::hasher::L2OWithdrawHasher;
use l2o_ord::inscription::inscription_id::InscriptionId;
use l2o_ord::operation::brc20::deploy::Deploy;
use l2o_ord::operation::brc20::mint::Mint;
use l2o_ord::operation::brc20::transfer::Transfer;
use l2o_ord::operation::brc20::BRC20Operation;
use l2o_ord::operation::brc21::l2deposit::L2Deposit;
use l2o_ord::operation::brc21::BRC21Operation;
use l2o_ord::operation::brc21::L2WithdrawV1;
use l2o_ord::operation::l2o_a::L2OABlockV1;
use l2o_ord::operation::l2o_a::L2OADeployV1;
use l2o_ord::operation::l2o_a::L2OAOperation;
use l2o_ord::operation::Operation;
use l2o_ord::operation::ProtocolType;
use l2o_ord::sat_point::SatPoint;
use l2o_ord::BIGDECIMAL_TEN;
use l2o_ord::MAXIMUM_SUPPLY;
use l2o_ord::MAX_DECIMAL_WIDTH;
use l2o_store::core::traits::L2OStoreReaderV1;
use l2o_store::core::traits::L2OStoreV1;

use crate::balance::Balance;
use crate::ctx::Context;
use crate::event::DeployEvent;
use crate::event::Event;
use crate::event::InscribeTransferEvent;
use crate::event::L2DepositEvent;
use crate::event::L2WithdrawEvent;
use crate::event::MintEvent;
use crate::event::Receipt;
use crate::event::TransferEvent;
use crate::log::TransferableLog;
use crate::script_key::ScriptKey;
use crate::script_key::BURN_ADDRESS;
use crate::tick::Tick;
use crate::token_info::TokenInfo;

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub txid: Txid,
    pub sequence_number: u32,
    pub inscription_id: InscriptionId,
    pub old_satpoint: SatPoint,
    // `new_satpoint` may be none when the transaction is not yet confirmed and the sat has not
    // been bound to the current outputs.
    pub new_satpoint: Option<SatPoint>,
    pub op: Operation,
    pub sat_in_outputs: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionMessage {
    pub txid: Txid,
    pub inscription_id: InscriptionId,
    pub inscription_number: i32,
    pub old_satpoint: SatPoint,
    pub new_satpoint: SatPoint,
    pub from: ScriptKey,
    pub to: Option<ScriptKey>,
    pub op: Operation,
}

impl ExecutionMessage {
    pub fn from_message(
        context: &mut Context,
        msg: &Message,
        chain: Chain,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            txid: msg.txid,
            inscription_id: msg.inscription_id,
            old_satpoint: msg.old_satpoint,
            inscription_number: 0,
            new_satpoint: msg
                .new_satpoint
                .ok_or(anyhow::anyhow!("new satpoint cannot be None"))?,
            from: context.get_script_key_on_satpoint(&msg.old_satpoint, chain)?,
            to: if msg.sat_in_outputs {
                Some(
                    context
                        .get_script_key_on_satpoint(msg.new_satpoint.as_ref().unwrap(), chain)?,
                )
            } else {
                None
            },
            op: msg.op.clone(),
        })
    }

    pub fn execute(context: &mut Context, msg: &ExecutionMessage) -> anyhow::Result<Receipt> {
        tracing::debug!(
            "execute message:
            {:?}",
            msg
        );
        let event = match &msg.op {
            Operation::BRC20(BRC20Operation::Deploy(deploy)) => {
                Self::process_deploy(context, msg, deploy.clone())
            }
            Operation::BRC20(BRC20Operation::Mint { mint, parent }) => {
                Self::process_mint(context, msg, mint.clone(), *parent)
            }
            Operation::BRC20(BRC20Operation::InscribeTransfer(transfer)) => {
                Self::process_inscribe_transfer(context, msg, transfer.clone())
            }
            Operation::BRC20(BRC20Operation::Transfer(transfer)) => {
                Self::process_transfer(context, msg, transfer.clone())
            }
            Operation::BRC21(BRC21Operation::Deploy(deploy)) => {
                Self::process_deploy(context, msg, deploy.clone())
            }
            Operation::BRC21(BRC21Operation::Mint { mint, parent }) => {
                Self::process_mint(context, msg, mint.clone(), *parent)
            }
            Operation::BRC21(BRC21Operation::InscribeTransfer(transfer)) => {
                Self::process_inscribe_transfer(context, msg, transfer.clone())
            }
            Operation::BRC21(BRC21Operation::Transfer(transfer)) => {
                Self::process_transfer(context, msg, transfer.clone())
            }
            Operation::BRC21(BRC21Operation::L2Deposit(l2deposit)) => {
                Self::process_l2_deposit(context, msg, l2deposit.clone())
            }
            Operation::BRC21(BRC21Operation::L2Withdraw(l2withdraw)) => {
                Self::process_l2_withdraw(context, msg, l2withdraw.clone())
            }
            Operation::L2OA(L2OAOperation::Deploy(deploy)) => {
                Self::process_l2o_a_deploy(context, msg, deploy.clone())
            }
            Operation::L2OA(L2OAOperation::Block(block)) => {
                Self::process_l2o_a_block(context, msg, block.clone())
            }
        };

        let receipt = Receipt {
            inscription_id: msg.inscription_id,
            inscription_number: msg.inscription_number,
            old_satpoint: msg.old_satpoint,
            new_satpoint: msg.new_satpoint,
            from: msg.from.clone(),
            // redirect receiver to sender if transfer to conibase.
            to: msg.to.clone().map_or(msg.from.clone(), |v| v),
            op: msg.op.op_type(),
            result: match event {
                Ok(event) => Ok(event),
                Err(e) => return Err(anyhow::anyhow!("execute exception: {e}")),
            },
        };

        tracing::debug!("message receipt: {:?}", receipt);
        Ok(receipt)
    }

    fn process_deploy(
        context: &mut Context,
        msg: &ExecutionMessage,
        deploy: Deploy,
    ) -> anyhow::Result<Event> {
        // ignore inscribe inscription to coinbase. let
        let to_script_key = msg.to.clone().ok_or(BRC2XError::InscribeToCoinbase)?;
        let ptype = msg.op.p_type();

        let tick = deploy.tick.parse::<Tick>()?;
        let mut max_supply = deploy.max_supply.clone();
        let mut is_self_mint = false;

        // proposal for issuance self mint token.
        // https://l1f.discourse.group/t/brc-20-proposal-for-issuance-and-burn-enhancements-brc20-ip-1/621
        if tick.self_issuance_tick() {
            if context.chain_ctx.blockheight
                < context.chain_ctx.chain.self_issuance_activation_height()
            {
                return Err(Error::BRC2XError(BRC2XError::SelfIssuanceNotActivated).into());
            }
            if !deploy.self_mint.unwrap_or_default() {
                return Err(Error::BRC2XError(BRC2XError::SelfIssuanceCheckedFailed).into());
            }
            if deploy.max_supply == u64::MIN.to_string() {
                max_supply = u64::MAX.to_string();
            }
            is_self_mint = true;
        }

        if let Some(stored_tick_info) = context
            .get_token_info(&tick, ptype)
            .map_err(Error::LedgerError)?
        {
            return Err(Error::BRC2XError(BRC2XError::DuplicateTick(
                stored_tick_info.tick.to_string(),
            ))
            .into());
        }

        let dec = Decimal::from_str(&deploy.decimals.map_or(MAX_DECIMAL_WIDTH.to_string(), |v| v))?
            .checked_to_u8()?;
        if dec > MAX_DECIMAL_WIDTH {
            return Err(Error::BRC2XError(BRC2XError::DecimalsTooLarge(dec)).into());
        }
        let base = BIGDECIMAL_TEN.checked_powu(u64::from(dec))?;

        let supply = Decimal::from_str(&max_supply)?;

        if supply.sign() == Sign::NoSign
            || supply > MAXIMUM_SUPPLY.to_owned()
            || supply.scale() > i64::from(dec)
        {
            return Err(Error::BRC2XError(BRC2XError::InvalidSupply(supply.to_string())).into());
        }

        let limit = Decimal::from_str(&deploy.mint_limit.map_or(max_supply, |v| v))?;

        if limit.sign() == Sign::NoSign
            || limit > MAXIMUM_SUPPLY.to_owned()
            || limit.scale() > i64::from(dec)
        {
            return Err(Error::BRC2XError(BRC2XError::MintLimitOutOfRange(
                tick.to_lowercase().to_string(),
                limit.to_string(),
            ))
            .into());
        }

        let supply = supply.checked_mul(&base)?.checked_to_u128()?;
        let limit = limit.checked_mul(&base)?.checked_to_u128()?;

        let new_info = TokenInfo {
            inscription_id: msg.inscription_id,
            inscription_number: msg.inscription_number,
            tick: tick.clone(),
            decimal: dec,
            supply,
            burned_supply: 0u128,
            limit_per_mint: limit,
            minted: 0u128,
            deploy_by: to_script_key,
            is_self_mint,
            deployed_number: context.chain_ctx.blockheight,
            latest_mint_number: context.chain_ctx.blockheight,
            deployed_timestamp: context.chain_ctx.blocktime,
        };
        context
            .insert_token_info(&tick, &new_info, ptype)
            .map_err(Error::LedgerError)?;

        Ok(Event::Deploy(DeployEvent {
            supply,
            limit_per_mint: limit,
            decimal: dec,
            tick: new_info.tick,
            self_mint: is_self_mint,
        }))
    }

    fn process_mint(
        context: &mut Context,
        msg: &ExecutionMessage,
        mint: Mint,
        parent: Option<InscriptionId>,
    ) -> anyhow::Result<Event> {
        // ignore inscribe inscription to coinbase. let
        let to_script_key = msg.to.clone().ok_or(BRC2XError::InscribeToCoinbase)?;
        let ptype = msg.op.p_type();

        let tick = mint.tick.parse::<Tick>()?;

        let tick_info = context
            .get_token_info(&tick, ptype)
            .map_err(Error::LedgerError)?
            .ok_or(BRC2XError::TickNotFound(tick.to_string()))?;

        // check if self mint is allowed.
        if tick_info.is_self_mint
            && !parent.is_some_and(|parent| parent == tick_info.inscription_id)
        {
            return Err(Error::BRC2XError(BRC2XError::SelfMintPermissionDenied).into());
        }

        let base = BIGDECIMAL_TEN.checked_powu(u64::from(tick_info.decimal))?;

        let mut amt = Decimal::from_str(&mint.amount)?;

        if amt.scale() > i64::from(tick_info.decimal) {
            return Err(Error::BRC2XError(BRC2XError::AmountOverflow(amt.to_string())).into());
        }

        amt = amt.checked_mul(&base)?;
        if amt.sign() == Sign::NoSign {
            return Err(Error::BRC2XError(BRC2XError::InvalidZeroAmount).into());
        }
        if amt > Into::<Decimal>::into(tick_info.limit_per_mint) {
            return Err(Error::BRC2XError(BRC2XError::AmountExceedLimit(amt.to_string())).into());
        }
        let minted = Into::<Decimal>::into(tick_info.minted);
        let supply = Into::<Decimal>::into(tick_info.supply);

        if minted >= supply {
            return Err(
                Error::BRC2XError(BRC2XError::TickMinted(tick_info.tick.to_string())).into(),
            );
        }

        // cut off any excess.
        let mut out_msg = None;
        amt = if amt.checked_add(&minted)? > supply {
            let new = supply.checked_sub(&minted)?;
            out_msg = Some(format!(
                "amt has been cut off to fit the supply! origin: {}, now: {}",
                amt, new
            ));
            new
        } else {
            amt
        };

        // get or initialize user balance.
        let mut balance = context
            .get_balance(&to_script_key, &tick, ptype)
            .map_err(Error::LedgerError)?
            .map_or(Balance::new(&tick), |v| v);

        // add amount to available balance.
        balance.overall_balance = Into::<Decimal>::into(balance.overall_balance)
            .checked_add(&amt)?
            .checked_to_u128()?;

        // store to database.
        context
            .update_token_balance(&to_script_key, balance, ptype)
            .map_err(Error::LedgerError)?;

        // update token minted.
        let minted = minted.checked_add(&amt)?.checked_to_u128()?;
        context
            .update_mint_token_info(&tick, minted, context.chain_ctx.blockheight, ptype)
            .map_err(Error::LedgerError)?;

        Ok(Event::Mint(MintEvent {
            tick: tick_info.tick,
            amount: amt.checked_to_u128()?,
            msg: out_msg,
        }))
    }

    fn process_inscribe_transfer(
        context: &mut Context,
        msg: &ExecutionMessage,
        transfer: Transfer,
    ) -> anyhow::Result<Event> {
        // ignore inscribe inscription to coinbase. let
        let to_script_key = msg.to.clone().ok_or(BRC2XError::InscribeToCoinbase)?;
        let ptype = msg.op.p_type();

        let tick = transfer.tick.parse::<Tick>()?;

        let token_info = context
            .get_token_info(&tick, ptype)
            .map_err(Error::LedgerError)?
            .ok_or(BRC2XError::TickNotFound(tick.to_string()))?;

        let base = BIGDECIMAL_TEN.checked_powu(u64::from(token_info.decimal))?;

        let mut amt = Decimal::from_str(&transfer.amount)?;

        if amt.scale() > i64::from(token_info.decimal) {
            return Err(Error::BRC2XError(BRC2XError::AmountOverflow(amt.to_string())).into());
        }

        amt = amt.checked_mul(&base)?;
        if amt.sign() == Sign::NoSign || amt > Into::<Decimal>::into(token_info.supply) {
            return Err(Error::BRC2XError(BRC2XError::AmountOverflow(amt.to_string())).into());
        }

        let mut balance = context
            .get_balance(&to_script_key, &tick, ptype)
            .map_err(Error::LedgerError)?
            .map_or(Balance::new(&tick), |v| v);

        let overall = Into::<Decimal>::into(balance.overall_balance);
        let transferable = Into::<Decimal>::into(balance.transferable_balance);
        let available = overall.checked_sub(&transferable)?;
        if available < amt {
            return Err(Error::BRC2XError(BRC2XError::InsufficientBalance(
                available.to_string(),
                amt.to_string(),
            ))
            .into());
        }

        balance.transferable_balance = transferable.checked_add(&amt)?.checked_to_u128()?;

        let amt = amt.checked_to_u128()?;
        context
            .update_token_balance(&to_script_key, balance, ptype)
            .map_err(Error::LedgerError)?;

        let transferable_asset = TransferableLog {
            inscription_id: msg.inscription_id,
            inscription_number: msg.inscription_number,
            amount: amt,
            tick: token_info.tick.clone(),
            owner: to_script_key,
        };

        context
            .insert_transferable_asset(msg.new_satpoint, &transferable_asset, ptype)
            .map_err(Error::LedgerError)?;

        Ok(Event::InscribeTransfer(InscribeTransferEvent {
            tick: transferable_asset.tick,
            amount: amt,
        }))
    }

    fn process_transfer(
        context: &mut Context,
        msg: &ExecutionMessage,
        _transfer: Transfer,
    ) -> anyhow::Result<Event> {
        // redirect receiver to sender if transfer to coinbase.
        let mut out_msg = None;

        let to_script_key = if msg.to.clone().is_none() {
            out_msg = Some(
                "redirect receiver to sender, reason: transfer inscription to
    coinbase"
                    .to_string(),
            );
            msg.from.clone()
        } else {
            msg.to.clone().unwrap()
        };
        let from_ptype = msg.op.p_type();
        let is_burn = &to_script_key == &*BURN_ADDRESS;
        let to_ptype = if from_ptype.is_brc_20() && is_burn {
            ProtocolType::BRC21
        } else {
            from_ptype
        };
        let transferable = context
            .get_transferable_assets_by_satpoint(&msg.old_satpoint, from_ptype)
            .map_err(Error::LedgerError)?
            .ok_or(BRC2XError::TransferableNotFound(msg.inscription_id))?;

        let amt = Into::<Decimal>::into(transferable.amount);

        if transferable.owner != msg.from {
            return Err(Error::BRC2XError(BRC2XError::TransferableOwnerNotMatch(
                msg.inscription_id,
            ))
            .into());
        }

        let tick = transferable.tick;

        let token_info = context
            .get_token_info(&tick, from_ptype)
            .map_err(Error::LedgerError)?
            .ok_or(BRC2XError::TickNotFound(tick.to_string()))?;

        // update from key balance.
        let mut from_balance = context
            .get_balance(&msg.from, &tick, from_ptype)
            .map_err(Error::LedgerError)?
            .map_or(Balance::new(&tick), |v| v);

        let from_overall = Into::<Decimal>::into(from_balance.overall_balance);
        let from_transferable = Into::<Decimal>::into(from_balance.transferable_balance);

        let from_overall = from_overall.checked_sub(&amt)?.checked_to_u128()?;
        let from_transferable = from_transferable.checked_sub(&amt)?.checked_to_u128()?;

        from_balance.overall_balance = from_overall;
        from_balance.transferable_balance = from_transferable;

        context
            .update_token_balance(&msg.from, from_balance, from_ptype)
            .map_err(Error::LedgerError)?;

        // update to key balance.
        let mut to_balance = context
            .get_balance(&to_script_key, &tick, to_ptype)
            .map_err(Error::LedgerError)?
            .map_or(Balance::new(&tick), |v| v);

        let to_overall = Into::<Decimal>::into(to_balance.overall_balance);
        to_balance.overall_balance = to_overall.checked_add(&amt)?.checked_to_u128()?;

        context
            .update_token_balance(&to_script_key, to_balance, to_ptype)
            .map_err(Error::LedgerError)?;

        context
            .remove_transferable_asset(msg.old_satpoint, from_ptype)
            .map_err(Error::LedgerError)?;

        // update burned supply if transfer to op_return.
        match to_script_key {
            ScriptKey::ScriptHash { is_op_return, .. } if is_op_return => {
                let burned_amt = Into::<Decimal>::into(token_info.burned_supply)
                    .checked_add(&amt)?
                    .checked_to_u128()?;
                context
                    .update_burned_token_info(&tick, burned_amt, from_ptype)
                    .map_err(Error::LedgerError)?;
                out_msg = Some(format!(
                    "transfer to op_return, burned supply increased: {}",
                    amt
                ));
            }
            _ => (),
        }

        Ok(Event::Transfer(TransferEvent {
            msg: out_msg,
            tick: token_info.tick,
            amount: amt.checked_to_u128()?,
        }))
    }

    fn process_l2_deposit(
        context: &mut Context,
        msg: &ExecutionMessage,
        l2deposit: L2Deposit,
    ) -> anyhow::Result<Event> {
        let ptype = msg.op.p_type();

        let tick = l2deposit.tick.parse::<Tick>()?;

        let token_info = context
            .get_token_info(&tick, ptype)
            .map_err(Error::LedgerError)?
            .ok_or(BRC2XError::TickNotFound(tick.to_string()))?;

        let base = BIGDECIMAL_TEN.checked_powu(u64::from(token_info.decimal))?;

        let mut amt = Decimal::from_str(&l2deposit.amount)?;

        if amt.scale() > i64::from(token_info.decimal) {
            return Err(Error::BRC2XError(BRC2XError::AmountOverflow(amt.to_string())).into());
        }

        amt = amt.checked_mul(&base)?;
        if amt.sign() == Sign::NoSign || amt > Into::<Decimal>::into(token_info.supply) {
            return Err(Error::BRC2XError(BRC2XError::AmountOverflow(amt.to_string())).into());
        }

        // update from key balance.
        let mut from_balance = context
            .get_balance(&msg.from, &tick, ptype)
            .map_err(Error::LedgerError)?
            .map_or(Balance::new(&tick), |v| v);

        let from_overall = Into::<Decimal>::into(from_balance.overall_balance);
        let from_transferable = Into::<Decimal>::into(from_balance.transferable_balance);

        let from_overall = from_overall.checked_sub(&amt)?.checked_to_u128()?;
        let from_transferable = from_transferable.checked_sub(&amt)?.checked_to_u128()?;

        from_balance.overall_balance = from_overall;
        from_balance.transferable_balance = from_transferable;

        context
            .update_token_balance(&msg.from, from_balance, ptype)
            .map_err(Error::LedgerError)?;

        let holding_balance =
            context.get_brc21_deposits_holding_balance(l2deposit.l2id, tick.clone())?;
        context.update_brc21_deposits_holding_balance(
            l2deposit.l2id,
            &tick,
            holding_balance.checked_add(amt.checked_to_u128()?).unwrap(),
        )?;
        context.kv.append_l2_deposit(l2deposit.clone())?;

        Ok(Event::L2Deposit(L2DepositEvent {
            l2id: l2deposit.l2id,
            tick: l2deposit.tick,
            to: l2deposit.to,
            amount: l2deposit.amount,
        }))
    }

    fn process_l2_withdraw(
        context: &mut Context,
        msg: &ExecutionMessage,
        l2withdraw: L2WithdrawV1,
    ) -> anyhow::Result<Event> {
        let to_script_key = ScriptKey::Address(Address::from_str(&l2withdraw.to).unwrap());
        let ptype = msg.op.p_type();

        let tick = l2withdraw.tick.parse::<Tick>()?;

        let token_info = context
            .get_token_info(&tick, ptype)
            .map_err(Error::LedgerError)?
            .ok_or(BRC2XError::TickNotFound(tick.to_string()))?;

        let base = BIGDECIMAL_TEN.checked_powu(u64::from(token_info.decimal))?;

        let mut amt = Decimal::from_str(&l2withdraw.amount)?;

        if amt.scale() > i64::from(token_info.decimal) {
            return Err(Error::BRC2XError(BRC2XError::AmountOverflow(amt.to_string())).into());
        }

        amt = amt.checked_mul(&base)?;
        if amt.sign() == Sign::NoSign || amt > Into::<Decimal>::into(token_info.supply) {
            return Err(Error::BRC2XError(BRC2XError::AmountOverflow(amt.to_string())).into());
        }

        // update from key balance.
        let mut to_balance = context
            .get_balance(&to_script_key, &tick, ptype)
            .map_err(Error::LedgerError)?
            .map_or(Balance::new(&tick), |v| v);

        let to_overall = Into::<Decimal>::into(to_balance.overall_balance);
        let to_transferable = Into::<Decimal>::into(to_balance.transferable_balance);

        let to_overall = to_overall.checked_add(&amt)?.checked_to_u128()?;
        let to_transferable = to_transferable.checked_add(&amt)?.checked_to_u128()?;

        to_balance.overall_balance = to_overall;
        to_balance.transferable_balance = to_transferable;

        context
            .update_token_balance(&to_script_key, to_balance, ptype)
            .map_err(Error::LedgerError)?;

        let holding_balance =
            context.get_brc21_deposits_holding_balance(l2withdraw.l2id, tick.clone())?;
        context.update_brc21_deposits_holding_balance(
            l2withdraw.l2id,
            &tick,
            holding_balance.checked_sub(amt.checked_to_u128()?).unwrap(),
        )?;

        if Sha256Hasher::get_l2_withdraw_hash(&l2withdraw) != l2withdraw.proof.value {
            anyhow::bail!("proof value mismatch");
        }

        if !l2withdraw.proof.verify_marked_if::<Sha256Hasher>(false) {
            // TODO: check if root is valid
            anyhow::bail!("invalid proof");
        }

        Ok(Event::L2Withdraw(L2WithdrawEvent {
            l2id: l2withdraw.l2id,
            tick: l2withdraw.tick,
            to: l2withdraw.to,
            amount: l2withdraw.amount,
        }))
    }

    fn process_l2o_a_deploy(
        context: &mut Context,
        _msg: &ExecutionMessage,
        deploy: L2OADeployV1,
    ) -> anyhow::Result<Event> {
        let l2id = deploy.l2id;
        if context.kv.has_deployed_l2id(l2id)? {
            tracing::debug!("l2o {} already deployed", l2id);
            return Ok(Event::L2OADeploy);
        }
        if deploy.verifier_data.is_groth_16_bn_128() {
            anyhow::bail!("unsupported verifier type");
        };
        context.kv.report_deploy_inscription(deploy)?;
        tracing::info!("l2o {} deployed", l2id);
        Ok(Event::L2OADeploy)
    }

    fn process_l2o_a_block(
        context: &mut Context,
        _msg: &ExecutionMessage,
        block: L2OABlockV1,
    ) -> anyhow::Result<Event> {
        let l2id = block.l2id;
        if !context.kv.has_deployed_l2id(l2id)? {
            tracing::debug!("l2o {} not deployed yet", l2id);
            return Ok(Event::L2OABlock);
        }

        let deploy = context.kv.get_deploy_inscription(l2id)?;

        let block_proof = if deploy.verifier_data.is_groth_16_bn_128() {
            block
                .proof
                .clone()
                .try_as_groth_16_bn_128()
                .ok_or(anyhow::anyhow!("marformed proof"))?
        } else {
            anyhow::bail!("unsupported proof type");
        };

        let bitcoin_block_hash = Hash256::from_hex(
            &context
                .chain_ctx
                .bitcoin_rpc
                .get_block_hash(block.bitcoin_block_number)?
                .to_string(),
        )?;
        if bitcoin_block_hash != block.bitcoin_block_hash {
            anyhow::bail!("bitcoin block number mismatch");
        }

        let superchain_root = context
            .kv
            .get_superchainroot_at_block(block.bitcoin_block_number, deploy.hash_function)?;
        if superchain_root != block.superchain_root {
            anyhow::bail!("superchain root mismatch");
        }

        let last_public_key = if let Ok(last_block) = context.kv.get_last_block_inscription(l2id) {
            if block.l2_block_number != last_block.l2_block_number + 1 {
                anyhow::bail!("block must be consecutive");
            }

            if block.bitcoin_block_number <= last_block.bitcoin_block_number {
                anyhow::bail!("bitcoin block must be bigger than previous");
            }

            if block.start_state_root != last_block.end_state_root {
                anyhow::bail!("start state root must match the previous block's end state root")
            }

            if block.start_withdrawal_state_root != last_block.end_withdrawal_state_root {
                anyhow::bail!(
                    "start withdrawal root must match the previous block's end withdrawal root"
                )
            }

            last_block.public_key
        } else {
            if block.l2_block_number != 0 {
                anyhow::bail!("genesis block number must be zero");
            }
            if block.start_state_root != deploy.start_state_root {
                anyhow::bail!("genesis block state root must be equal to deploy start state root");
            }

            deploy.public_key
        };

        let mut uncompressed_bytes = Vec::new();
        block_proof.serialize_uncompressed(&mut uncompressed_bytes)?;

        let block_hash = if deploy.hash_function.is_sha_256() {
            Sha256Hasher::get_l2_block_hash(&block)
        } else if deploy.hash_function.is_blake_3() {
            Blake3Hasher::get_l2_block_hash(&block)
        } else if deploy.hash_function.is_keccak_256() {
            Keccak256Hasher::get_l2_block_hash(&block)
        } else if deploy.hash_function.is_poseidon_goldilocks() {
            PoseidonHasher::get_l2_block_hash(&block)
        } else {
            anyhow::bail!("unsupported hash function");
        };

        let public_inputs: [Fr; 2] = block_hash.into();
        if public_inputs.to_vec() != block_proof.public_inputs {
            anyhow::bail!("public inputs mismatch");
        }

        let vk = deploy
            .verifier_data
            .try_as_groth_16_bn_128()
            .ok_or(anyhow::anyhow!("marformed verifier"))?
            .0;

        let processed_vk = Groth16::<Bn254>::process_vk(&vk)?;

        assert!(Groth16::<Bn254>::verify_proof(
            &processed_vk,
            &block_proof.proof,
            &block_proof.public_inputs,
        )?);

        if !last_public_key.is_zero() {
            verify_sig(&last_public_key, &block.signature, &block_hash.0)?;
        }

        context.kv.set_last_block_inscription(block)?;
        tracing::info!("l2id {} block", l2id);

        return Ok(Event::L2OABlock);
    }
}
