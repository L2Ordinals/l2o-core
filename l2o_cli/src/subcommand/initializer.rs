use ark_bn254::Bn254;
use ark_bn254::Fr;
use ark_crypto_primitives::snark::CircuitSpecificSetupSNARK;
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use ark_groth16::ProvingKey;
use ark_groth16::VerifyingKey;
use ark_std::rand::rngs::StdRng;
use ark_std::rand::RngCore;
use ark_std::rand::SeedableRng;
use ark_std::test_rng;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::hash::L2OHash;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;
use l2o_common::InitializerArgs;
use l2o_crypto::hash::hash_functions::block_hasher::get_block_payload_bytes;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::traits::L2OBlockHasher;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use l2o_crypto::proof::groth16::bn128::verifier_data::Groth16VerifierDataSerializable;
use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_indexer_ordhook::l2o::inscription::L2OInscription;
use l2o_indexer_ordhook::proof::snarkjs::ProofJson;
use serde_json::json;

use crate::circuits::BlockCircuit;

pub async fn run(
    _args: &InitializerArgs,
) -> anyhow::Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>, StdRng)> {
    let deploy_json = include_str!("../../../l2o_indexer_ordhook/assets/deploy.json");
    let block_json = include_str!("../../../l2o_indexer_ordhook/assets/block.json");
    let deploy_data = serde_json::from_str::<L2OInscription>(deploy_json).unwrap();
    let block_data = serde_json::from_str::<L2OInscription>(block_json).unwrap();
    let mut deploy = match deploy_data {
        L2OInscription::Deploy(deploy) => deploy,
        _ => unreachable!(),
    };
    let block = match block_data {
        L2OInscription::Block(block) => block,
        _ => unreachable!(),
    };

    let block_proof = block.proof.to_proof_with_public_inputs_groth16_bn254();
    let block_inscription = L2OBlockInscriptionV1 {
        p: "l2o-a".to_string(),
        op: "Block".to_string(),

        l2id: block.l2id.into(),
        l2_block_number: block.block_parameters.block_number.into(),

        bitcoin_block_number: 0,
        bitcoin_block_hash: Hash256::zero(),

        public_key: L2OCompactPublicKey::from_hex(&block.block_parameters.public_key).unwrap(),

        start_state_root: Hash256::zero(),
        end_state_root: Hash256::from_hex(&deploy.start_state_root).unwrap(),

        deposit_state_root: Hash256::from_hex(&block.block_parameters.deposits_root).unwrap(),

        start_withdrawal_state_root: Hash256::zero(),
        end_withdrawal_state_root: Hash256::from_hex(&block.block_parameters.withdrawals_root)
            .unwrap(),

        proof: L2OAProofData::Groth16BN128(Groth16BN128ProofData {
            proof: block_proof.proof,
            public_inputs: block_proof.public_inputs,
        }),

        superchain_root: Hash256::zero(),
        signature: L2OSignature512::from_hex(&block.signature).unwrap(),
    };

    let block_payload = get_block_payload_bytes(&block_inscription);
    let block_hash: [Fr; 2] = Sha256Hasher::get_l2_block_hash(&block_inscription).into();

    let block_circuit = BlockCircuit {
        block_hash,
        block_payload,
    };

    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(test_rng().next_u64());

    let (pk, vk) = Groth16::<Bn254>::setup(block_circuit.clone(), &mut rng).unwrap();

    let vk_json = Groth16VerifierDataSerializable::from_vk(&vk);
    deploy.vk.ic = vk_json.ic.into_iter().map(|x| x.into()).collect();
    deploy.vk.vk_alpha_1 = vk_json.vk_alpha_1.to_vec();
    deploy.vk.vk_beta_2 = vk_json.vk_beta_2.into_iter().map(|x| x.into()).collect();
    deploy.vk.vk_gamma_2 = vk_json.vk_gamma_2.into_iter().map(|x| x.into()).collect();
    deploy.vk.vk_delta_2 = vk_json.vk_delta_2.into_iter().map(|x| x.into()).collect();

    let mut deploy_value = serde_json::to_value(&deploy).unwrap();
    deploy_value["p"] = json!("l2o");
    deploy_value["op"] = json!("Deploy");
    std::fs::write(
        "./l2o_indexer_ordhook/assets/deploy.json",
        serde_json::to_string_pretty(&deploy_value).unwrap(),
    )
    .unwrap();

    std::process::Command::new("make")
        .args([
            "FILE=./l2o_indexer_ordhook/assets/deploy.json",
            "ord-inscribe",
        ])
        .spawn()
        .expect("failed to execute process");

    let mut block_value = serde_json::to_value(&block).unwrap();
    block_value["p"] = json!("l2o");
    block_value["op"] = json!("Block");
    let processed_vk = Groth16::<Bn254>::process_vk(&vk).unwrap();
    let proof = Groth16::<Bn254>::prove(&pk, block_circuit.clone(), &mut rng).unwrap();

    Groth16::<Bn254>::verify_with_processed_vk(&processed_vk, &block_circuit.block_hash, &proof)
        .unwrap();
    let proof_json =
        ProofJson::from_proof_with_public_inputs_groth16_bn254(&Groth16BN128ProofData {
            proof,
            public_inputs: block_hash.to_vec(),
        });
    block_value["proof"] = json!(proof_json);
    std::fs::write(
        "./l2o_indexer_ordhook/assets/block.json",
        serde_json::to_string_pretty(&block_value).unwrap(),
    )
    .unwrap();

    std::process::Command::new("make")
        .args([
            "FILE=./l2o_indexer_ordhook/assets/block.json",
            "ord-inscribe",
        ])
        .spawn()
        .expect("failed to execute process");

    Ok((pk, vk, rng))
}
