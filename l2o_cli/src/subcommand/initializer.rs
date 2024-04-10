use ark_bn254::Fr;
use l2o_common::common::data::hash::Hash256;
use l2o_common::common::data::hash::L2OHash;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;
use l2o_common::InitializerArgs;
use l2o_crypto::hash::hash_functions::sha256::Sha256Hasher;
use l2o_crypto::hash::traits::L2OBlockHasher;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use l2o_crypto::standards::l2o_a::proof::L2OAProofData;
use l2o_crypto::standards::l2o_a::L2OBlockInscriptionV1;
use l2o_indexer_ordhook::l2o::inscription::L2OInscription;

pub async fn run(args: &InitializerArgs) -> anyhow::Result<()> {
    let deploy_json = include_str!("../../../l2o_indexer_ordhook/assets/deploy.json");
    let block_json = include_str!("../../../l2o_indexer_ordhook/assets/block.json");
    let deploy_data = serde_json::from_str::<L2OInscription>(deploy_json).unwrap();
    let block_data = serde_json::from_str::<L2OInscription>(block_json).unwrap();
    let deploy = match deploy_data {
        L2OInscription::Deploy(deploy) => deploy,
        _ => unreachable!(),
    };
    let block = match block_data {
        L2OInscription::Block(block) => block,
        _ => unreachable!(),
    };

    let block_proof = block.proof.to_proof_groth16_bn254();
    let mut block_inscription = L2OBlockInscriptionV1 {
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
            proof: block_proof,
            public_inputs: vec![],
        }),

        superchain_root: Hash256::zero(),
        signature: L2OSignature512::from_hex(&block.signature).unwrap(),
    };

    let public_inputs: [Fr; 2] = Sha256Hasher::get_l2_block_hash(&block_inscription).into();
    match block_inscription.proof {
        L2OAProofData::Groth16BN128(ref mut p) => {
            p.public_inputs.extend(public_inputs.into_iter());
        }
        _ => {
            unreachable!()
        }
    }

    Ok(())
}
