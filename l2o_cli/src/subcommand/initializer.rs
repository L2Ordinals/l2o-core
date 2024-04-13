use std::process::Command;

use ark_bn254::Bn254;
use ark_bn254::Fr;
use ark_crypto_primitives::snark::CircuitSpecificSetupSNARK;
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use ark_groth16::ProvingKey;
use ark_groth16::VerifyingKey;
use ark_std::rand::rngs::StdRng;
use ark_std::rand::SeedableRng;
use k256::schnorr::SigningKey;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::hash::L2OHash;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;
use l2o_common::InitializerArgs;
use l2o_crypto::hash::hash_functions::block_hasher::get_block_payload_bytes;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::traits::L2OBlockHasher;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16ProofSerializable;
use l2o_crypto::proof::groth16::bn128::verifier_data::Groth16VerifierSerializable;
use l2o_crypto::proof::L2OAVerifierSerializableData;
use l2o_crypto::signature::schnorr::sign_msg;
use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_indexer_ordhook::l2o::inscription::L2OInscription;
use serde_json::json;

use crate::circuits::BlockCircuit;

pub async fn run(
    _args: &InitializerArgs,
) -> anyhow::Result<(ProvingKey<Bn254>, VerifyingKey<Bn254>, StdRng, SigningKey)> {
    let deploy_json = include_str!("../../../l2o_indexer_ordhook/assets/deploy.json");
    let block_json = include_str!("../../../l2o_indexer_ordhook/assets/block.json");
    let deploy_data = serde_json::from_str::<L2OInscription>(deploy_json)?;
    let block_data = serde_json::from_str::<L2OInscription>(block_json)?;
    let mut deploy = match deploy_data {
        L2OInscription::Deploy(deploy) => deploy,
        _ => unreachable!(),
    };
    let mut block = match block_data {
        L2OInscription::Block(block) => block,
        _ => unreachable!(),
    };
    let signing_key = SigningKey::from_bytes(
        &hex::decode("60f0a76f41094bade9f7065da0fcb601dbd1c68a21f747e12691ccbe1cae9543").unwrap(),
    )
    .unwrap();

    let block_proof = block
        .proof
        .clone()
        .try_as_groth_16_proof_serializable()
        .unwrap()
        .to_proof_with_public_inputs_groth16_bn254()?;
    let mut block_inscription = L2OBlockInscriptionV1 {
        p: "l2o-a".to_string(),
        op: "Block".to_string(),

        l2id: block.l2id.into(),
        l2_block_number: block.block_parameters.block_number.into(),

        bitcoin_block_number: 0,
        bitcoin_block_hash: Hash256::zero(),

        public_key: L2OCompactPublicKey::from_hex(&block.block_parameters.public_key)?,

        start_state_root: Hash256::zero(),
        end_state_root: Hash256::from_hex(&deploy.start_state_root)?,

        deposit_state_root: Hash256::from_hex(&block.block_parameters.deposits_root)?,

        start_withdrawal_state_root: Hash256::zero(),
        end_withdrawal_state_root: Hash256::from_hex(&block.block_parameters.withdrawals_root)?,

        proof: L2OAProofData::Groth16BN128(Groth16BN128ProofData {
            proof: block_proof.proof,
            public_inputs: block_proof.public_inputs,
        }),

        superchain_root: Hash256::zero(),
        signature: L2OSignature512::from_hex(&block.signature)?,
    };

    let block_payload = get_block_payload_bytes(&block_inscription);
    let block_hash = Sha256Hasher::get_l2_block_hash(&block_inscription);
    let public_inputs: [Fr; 2] = block_hash.into();
    let signature = sign_msg(&signing_key, &block_hash.0)?;

    block_inscription.signature = signature.clone();
    block.signature = hex::encode(&signature.0);

    let block_circuit = BlockCircuit {
        block_hash: public_inputs,
        block_payload,
    };

    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(9365255816191338696);

    let (pk, vk) = Groth16::<Bn254>::setup(block_circuit.clone(), &mut rng)?;

    let vk_json = Groth16VerifierSerializable::from_vk(&vk);
    let vk_json_cloned = vk_json.clone();
    #[allow(irrefutable_let_patterns)]
    if let L2OAVerifierSerializableData::Groth16VerifierSerializable(ref mut vk) = deploy.vk {
        vk.vk_alpha_1 = vk_json.vk_alpha_1;
        vk.vk_beta_2 = vk_json.vk_beta_2;
        vk.vk_gamma_2 = vk_json.vk_gamma_2;
        vk.vk_delta_2 = vk_json.vk_delta_2;
        vk.ic = vk_json.ic;
    }

    let mut deploy_value = serde_json::to_value(&deploy)?;
    deploy_value["p"] = json!("l2o-a");
    deploy_value["op"] = json!("Deploy");
    std::fs::write(
        "./l2o_indexer_ordhook/assets/deploy.json",
        serde_json::to_string_pretty(&deploy_value)?,
    )?;

    assert!(Command::new("make")
        .args([
            "FILE=./l2o_indexer_ordhook/assets/deploy.json",
            "ord-inscribe",
        ])
        .status()
        .is_ok());

    let mut block_value = serde_json::to_value(&block)?;
    block_value["p"] = json!("l2o-a");
    block_value["op"] = json!("Block");
    let processed_vk = Groth16::<Bn254>::process_vk(&vk)?;
    let proof = Groth16::<Bn254>::prove(&pk, block_circuit.clone(), &mut rng)?;

    assert!(Groth16::<Bn254>::verify_with_processed_vk(
        &processed_vk,
        &block_circuit.block_hash,
        &proof
    )?);
    let proof_json = Groth16ProofSerializable::from_proof_with_public_inputs_groth16_bn254(
        &Groth16BN128ProofData {
            proof,
            public_inputs: public_inputs.to_vec(),
        },
    );

    let proof_deserialized = proof_json.to_proof_with_public_inputs_groth16_bn254()?;
    assert!(Groth16::<Bn254>::verify_proof(
        &Groth16::<Bn254>::process_vk(&vk_json_cloned.to_vk()?).unwrap(),
        &proof_deserialized.proof,
        &proof_deserialized.public_inputs,
    )?);

    block_value["proof"] = json!(proof_json);
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

    Ok((pk, vk, rng, signing_key))
}
