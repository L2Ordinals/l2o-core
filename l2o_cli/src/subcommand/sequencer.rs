use std::time::Duration;

use ark_bn254::Bn254;
use ark_bn254::Fr;
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use ark_groth16::ProvingKey;
use ark_groth16::VerifyingKey;
use ark_std::rand::rngs::StdRng;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::hash::L2OHash;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;
use l2o_common::InitializerArgs;
use l2o_common::SequencerArgs;
use l2o_crypto::hash::hash_functions::block_hasher::get_block_payload_bytes;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::traits::L2OBlockHasher;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_indexer_ordhook::l2o::inscription::L2OInscriptionBlock;
use l2o_indexer_ordhook::l2o::inscription::L2OInscriptionBlockParameters;
use l2o_indexer_ordhook::proof::snarkjs::ProofJson;
use l2o_indexer_ordhook::rpc::request::Id;
use l2o_indexer_ordhook::rpc::request::RequestParams;
use l2o_indexer_ordhook::rpc::request::RpcRequest;
use l2o_indexer_ordhook::rpc::request::Version;
use reqwest::Client;
use serde_json::json;
use serde_json::Value;

use crate::circuits::BlockCircuit;
use crate::subcommand::initializer;

async fn execute_single(
    client: &Client,
    args: &SequencerArgs,
    pk: &ProvingKey<Bn254>,
    _vk: &VerifyingKey<Bn254>,
    rng: &mut StdRng,
) -> anyhow::Result<()> {
    let response = client
        .post(&args.indexer_url)
        .json(&RpcRequest {
            jsonrpc: Version::V2,
            request: RequestParams::L2OGetLastBlockInscription(args.l2oid),
            id: Id::Number(1),
        })
        .send()
        .await?
        .json::<Value>()
        .await?;

    let prev_block = serde_json::from_value::<L2OBlockInscriptionV1>(response["result"].clone())?;

    let next_block = L2OInscriptionBlock {
        l2id: prev_block.l2id as u32,
        block_parameters: L2OInscriptionBlockParameters {
            state_root: Hash256::rand().to_hex(),
            public_key: prev_block.public_key.to_hex(),
            deposits_root: Hash256::rand().to_hex(),
            withdrawals_root: Hash256::rand().to_hex(),
            block_number: (prev_block.l2_block_number + 1) as u32,
        },
        proof: ProofJson::from_proof_with_public_inputs_groth16_bn254(
            &prev_block.proof.as_groth16_bn128(),
        ),
        signature: prev_block.signature.to_hex(),
    };

    let mock_proof = next_block
        .proof
        .to_proof_with_public_inputs_groth16_bn254()?;

    let block_inscription = L2OBlockInscriptionV1 {
        p: "l2o-a".to_string(),
        op: "Block".to_string(),

        l2id: next_block.l2id.into(),
        l2_block_number: next_block.block_parameters.block_number.into(),

        bitcoin_block_number: 0,
        bitcoin_block_hash: Hash256::zero(),

        public_key: L2OCompactPublicKey::from_hex(&next_block.block_parameters.public_key)?,

        start_state_root: prev_block.end_state_root.clone(),
        end_state_root: Hash256::from_hex(&next_block.block_parameters.state_root)?,

        deposit_state_root: Hash256::from_hex(&next_block.block_parameters.deposits_root)?,

        start_withdrawal_state_root: prev_block.end_withdrawal_state_root.clone(),
        end_withdrawal_state_root: Hash256::from_hex(
            &next_block.block_parameters.withdrawals_root,
        )?,

        proof: L2OAProofData::Groth16BN128(Groth16BN128ProofData {
            proof: mock_proof.proof,
            public_inputs: mock_proof.public_inputs,
        }),

        superchain_root: Hash256::zero(),
        signature: L2OSignature512::from_hex(&next_block.signature)?,
    };
    let block_payload = get_block_payload_bytes(&block_inscription);
    let block_hash: [Fr; 2] = Sha256Hasher::get_l2_block_hash(&block_inscription).into();
    let block_circuit = BlockCircuit {
        block_hash,
        block_payload,
    };
    let proof = Groth16::<Bn254>::prove(&pk, block_circuit, rng)?;
    let proof_json =
        ProofJson::from_proof_with_public_inputs_groth16_bn254(&Groth16BN128ProofData {
            proof,
            public_inputs: block_hash.to_vec(),
        });
    let mut block_value = serde_json::to_value(&next_block)?;
    block_value["proof"] = json!(proof_json);

    block_value["p"] = json!("l2o");
    block_value["op"] = json!("Block");
    std::fs::write(
        "./l2o_indexer_ordhook/assets/block.json",
        serde_json::to_string_pretty(&block_value)?,
    )?;

    std::process::Command::new("make")
        .args([
            "FILE=./l2o_indexer_ordhook/assets/block.json",
            "ord-inscribe",
        ])
        .spawn()
        .expect("failed to execute process");

    Ok::<_, anyhow::Error>(())
}

pub async fn run(args: &SequencerArgs) -> anyhow::Result<()> {
    let (pk, vk, mut rng) = initializer::run(&InitializerArgs {}).await?;

    let client = Client::new();
    loop {
        if let Err(err) = execute_single(&client, args, &pk, &vk, &mut rng).await {
            tracing::error!("{}", err);
        }
        tokio::time::sleep(Duration::from_secs(15)).await;
    }
}
