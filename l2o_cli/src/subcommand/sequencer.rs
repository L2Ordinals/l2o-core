use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use ark_bn254::Bn254;
use ark_bn254::Fr;
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use ark_groth16::ProvingKey;
use ark_groth16::VerifyingKey;
use ark_std::rand::rngs::StdRng;
use bitcoincore_rpc::RpcApi;
use k256::schnorr::SigningKey;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;
use l2o_common::standards::l2o_a::supported_crypto::L2OAHashFunction;
use l2o_common::InitializerArgs;
use l2o_common::SequencerArgs;
use l2o_crypto::hash::hash_functions::block_hasher::get_block_payload_bytes;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::traits::L2OBlockHasher;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16ProofSerializable;
use l2o_crypto::signature::schnorr::sign_msg;
use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::L2OABlockInscriptionV1;
use l2o_indexer_ordhook::standards::l2o_a::inscription::L2OAInscriptionBlock;
use l2o_indexer_ordhook::standards::l2o_a::inscription::L2OAInscriptionBlockParameters;
use l2o_rpc_provider::L2OAProvider;
use l2o_rpc_provider::Provider;
use serde_json::json;

use crate::circuits::BlockCircuit;
use crate::subcommand::initializer;

async fn execute_single(
    args: &SequencerArgs,
    pk: &ProvingKey<Bn254>,
    _vk: &VerifyingKey<Bn254>,
    rng: &mut StdRng,
    signing_key: &SigningKey,
    bitcoincore_rpc: Arc<bitcoincore_rpc::Client>,
    rpc: Arc<Provider>,
) -> anyhow::Result<()> {
    let prev_block = rpc.get_last_block_inscription(args.l2oid).await?;
    let bitcoin_block_number = prev_block.bitcoin_block_number + 1;
    let bitcoin_block_hash = bitcoincore_rpc.get_block_hash(bitcoin_block_number)?;
    let superchain_root = rpc
        .get_superchainroot_at_block(bitcoin_block_number, L2OAHashFunction::Sha256)
        .await?;

    let mut block = L2OAInscriptionBlock {
        l2id: prev_block.l2id as u32,
        block_parameters: L2OAInscriptionBlockParameters {
            state_root: Hash256::rand().to_hex(),
            public_key: prev_block.public_key.to_hex(),
            deposits_root: Hash256::rand().to_hex(),
            withdrawals_root: Hash256::rand().to_hex(),
            block_number: (prev_block.l2_block_number + 1) as u32,
        },
        bitcoin_block_number: bitcoin_block_number,
        bitcoin_block_hash: bitcoin_block_hash.to_string().trim_start_matches("0x").to_string(),
        superchain_root: superchain_root.to_hex(),
        proof: Groth16ProofSerializable::from_proof_with_public_inputs_groth16_bn254(
            &prev_block.proof.try_as_groth_16_bn_128().unwrap(),
        )
        .into(),
        signature: "aa1a18a79d73e2d7d0c636317b9ffc6d9492cdab3cc9872a15bd3c866d2cf132c7bb8bd90eb69e20e88372eab927e9b09897835edd81d3450a458c725ed581c0".to_string(),
    };

    let proof = block
        .proof
        .clone()
        .try_as_groth_16_proof_serializable()
        .unwrap()
        .to_proof_with_public_inputs_groth16_bn254()?;

    let mut block_inscription = L2OABlockInscriptionV1 {
        p: "l2o-a".to_string(),
        op: "Block".to_string(),

        l2id: block.l2id.into(),
        l2_block_number: block.block_parameters.block_number.into(),

        bitcoin_block_number,
        bitcoin_block_hash: Hash256::from_hex(&block.bitcoin_block_hash)?,

        public_key: L2OCompactPublicKey::from_hex(&block.block_parameters.public_key)?,

        start_state_root: prev_block.end_state_root.clone(),
        end_state_root: Hash256::from_hex(&block.block_parameters.state_root)?,

        deposit_state_root: Hash256::from_hex(&block.block_parameters.deposits_root)?,

        start_withdrawal_state_root: prev_block.end_withdrawal_state_root.clone(),
        end_withdrawal_state_root: Hash256::from_hex(&block.block_parameters.withdrawals_root)?,

        proof: L2OAProofData::Groth16BN128(Groth16BN128ProofData {
            proof: proof.proof,
            public_inputs: proof.public_inputs,
        }),

        superchain_root: superchain_root,
        signature: L2OSignature512::from_hex(&block.signature)?,
    };
    let block_payload = get_block_payload_bytes(&block_inscription);
    let block_hash = Sha256Hasher::get_l2_block_hash(&block_inscription);
    let signature = sign_msg(signing_key, &block_hash.0)?;
    block_inscription.signature = signature.clone();
    block.signature = hex::encode(&signature.0);
    let public_inputs: [Fr; 2] = block_hash.into();
    let block_circuit = BlockCircuit {
        block_hash: public_inputs,
        block_payload,
    };
    let proof = Groth16::<Bn254>::prove(&pk, block_circuit, rng)?;
    let proof_json = Groth16ProofSerializable::from_proof_with_public_inputs_groth16_bn254(
        &Groth16BN128ProofData {
            proof,
            public_inputs: public_inputs.to_vec(),
        },
    );
    let mut block_value = serde_json::to_value(&block)?;
    block_value["proof"] = json!(proof_json);

    block_value["p"] = json!("l2o-a");
    block_value["op"] = json!("Block");
    block_value["bitcoin_block_number"] = json!(block_inscription.bitcoin_block_number);
    block_value["bitcoin_block_hash"] = json!(block_inscription.bitcoin_block_hash.to_hex());
    block_value["superchain_root"] = json!(block_inscription.superchain_root.to_hex());
    std::fs::write(
        "./l2o_indexer_ordhook/assets/block.json",
        serde_json::to_string_pretty(&block_value)?,
    )?;

    assert!(Command::new("make")
        .args([
            "FILE=./l2o_indexer_ordhook/assets/block.json",
            "ord-inscribe",
        ])
        .status()
        .is_ok());

    Ok::<_, anyhow::Error>(())
}

pub async fn run(args: &SequencerArgs) -> anyhow::Result<()> {
    let (pk, vk, mut rng, signing_key, bitcoincore_rpc, rpc) = initializer::run(&InitializerArgs {
        indexer_url: args.indexer_url.to_string(),
        bitcoin_rpc: args.bitcoin_rpc.to_string(),
        bitcoin_rpcuser: args.bitcoin_rpcuser.to_string(),
        bitcoin_rpcpassword: args.bitcoin_rpcpassword.to_string(),
        l2oid: args.l2oid,
    })
    .await?;

    loop {
        if let Err(err) = execute_single(
            args,
            &pk,
            &vk,
            &mut rng,
            &signing_key,
            bitcoincore_rpc.clone(),
            rpc.clone(),
        )
        .await
        {
            tracing::error!("{}", err);
        }
        tokio::time::sleep(Duration::from_secs(15)).await;
    }
}
