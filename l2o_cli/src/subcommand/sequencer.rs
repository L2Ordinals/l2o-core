use std::process;
use std::time::Duration;

use ark_crypto_primitives::crh::sha256::constraints::Sha256Gadget;
use ark_crypto_primitives::crh::sha256::Sha256;
use ark_crypto_primitives::crh::CRHSchemeGadget;
use ark_crypto_primitives::snark::CircuitSpecificSetupSNARK;
use ark_ff::PrimeField;
use ark_groth16::Groth16;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::uint8::UInt8;
use ark_r1cs_std::ToBitsGadget;
use ark_r1cs_std::ToBytesGadget;
use ark_relations::r1cs::ConstraintSynthesizer;
use ark_std::rand::RngCore;
use ark_std::rand::SeedableRng;
use ark_std::test_rng;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::hash::L2OHash;
use l2o_common::SequencerArgs;
use l2o_crypto::hash::hash_functions::block_hasher::get_block_payload_bytes;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_indexer_ordhook::l2o::inscription::L2OInscription;
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

pub struct BlockCircuit<F: PrimeField> {
    block_hash: [F; 2],
    block_payload: Vec<u8>,
}

impl<F: PrimeField> ConstraintSynthesizer<F> for BlockCircuit<F> {
    fn generate_constraints(
        self,
        cs: ark_relations::r1cs::ConstraintSystemRef<F>,
    ) -> ark_relations::r1cs::Result<()> {
        let sha256_parameter =
            <Sha256Gadget<F> as CRHSchemeGadget<Sha256, F>>::ParametersVar::new_constant(
                cs.clone(),
                (),
            )?;

        let hash_input = self
            .block_payload
            .into_iter()
            .map(|row| UInt8::new_witness(ark_relations::ns!(cs, "hash input byte"), || Ok(row)))
            .flatten()
            .collect::<Vec<UInt8<F>>>();

        let hash_result =
            Sha256Gadget::<F>::evaluate(&sha256_parameter, &hash_input)?.to_bytes()?;
        let low = Boolean::le_bits_to_fp_var(&hash_result[0..16].to_bits_le()?)?;
        let high = Boolean::le_bits_to_fp_var(&hash_result[16..32].to_bits_le()?)?;

        let low_expected = FpVar::new_input(cs.clone(), || Ok(self.block_hash[0]))?;
        let high_expected = FpVar::new_input(cs.clone(), || Ok(self.block_hash[1]))?;

        low.enforce_equal(&low_expected)?;
        high.enforce_equal(&high_expected)?;

        Ok(())
    }
}

pub async fn run(args: &SequencerArgs) -> anyhow::Result<()> {
    let mut rng = ark_std::rand::rngs::StdRng::seed_from_u64(1);
    // let (pk, vk) = {
    //     let c = BlockCircuit {
    //         block_hash: ,
    //         block_payload: ,
    //     };
    // Groth16::<ark_bn254::Bn254>::setup(c, &mut rng).unwrap()
    // };
    //     let block_json =
    // include_str!("../../../l2o_indexer_ordhook/assets/block.json");     let p
    // = serde_json::from_str::<L2OInscription>(block_json).unwrap();
    // let block = match p {
    //     L2OInscription::Block(block) => block,
    //     _ => unreachable!()
    // };
    //     tracing::info!("{:?}", p);
    // let client = Client::new();
    // loop {
    //     if let Err(err) = (|| async {
    //         let response = client
    //             .post(&args.indexer_url)
    //             .json(&RpcRequest {
    //                 jsonrpc: Version::V2,
    //                 request:
    // RequestParams::L2OGetLastBlockInscription(args.l2oid),                 
    // id: Id::Number(1),             })
    //             .send()
    //             .await?
    //             .json::<Value>()
    //             .await?;
    //
    //         let last_block =
    //             
    // serde_json::from_value::<L2OBlockInscriptionV1>(response["result"].clone())?;
    //
    //         let new_block = L2OInscriptionBlock {
    //             l2id: last_block.l2id as u32,
    //             block_parameters: L2OInscriptionBlockParameters {
    //                 state_root: Hash256::rand().to_hex(),
    //                 public_key: last_block.public_key.to_hex(),
    //                 deposits_root: last_block.deposit_state_root.to_hex(),
    //                 withdrawals_root:
    // last_block.end_withdrawal_state_root.to_hex(),                 
    // block_number: (last_block.l2_block_number + 1) as u32,             },
    //             proof: ProofJson::from_proof_with_public_inputs_groth16_bn254(
    //                 &last_block.proof.as_groth16_bn128(),
    //             ),
    //             signature: Hash256::zero().to_hex(),
    //         };
    //         let mut value = serde_json::to_value(&new_block).unwrap();
    //         value["p"] = json!("l2o");
    //         value["op"] = json!("Block");
    //         std::fs::write(
    //             "./l2o_indexer_ordhook/assets/block.json",
    //             serde_json::to_string(&value).unwrap(),
    //         )?;
    //
    //         process::Command::new("make")
    //             .args([
    //                 "FILE=./l2o_indexer_ordhook/assets/block.json",
    //                 "ord-inscribe",
    //             ])
    //             .spawn()
    //             .expect("failed to execute process");
    //
    //         Ok::<_, anyhow::Error>(())
    //     })()
    //     .await
    //     {
    //         tracing::error!("{}", err);
    //     }
    //     tokio::time::sleep(Duration::from_secs(3)).await;
    // }
    Ok(())
}
