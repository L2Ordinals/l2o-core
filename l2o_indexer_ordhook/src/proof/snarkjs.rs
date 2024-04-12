use std::str::FromStr;

use ark_bn254::Bn254;
use ark_bn254::Fq;
use ark_bn254::Fq2;
use ark_bn254::Fr;
use ark_bn254::G1Affine;
use ark_bn254::G1Projective;
use ark_bn254::G2Affine;
use ark_bn254::G2Projective;
use ark_ec::pairing::Pairing;
use ark_groth16::Proof;
use ark_groth16::VerifyingKey;
use ark_serialize::CanonicalDeserialize;
use ark_serialize::CanonicalSerialize;
use l2o_common::str_to_fq;
use l2o_common::str_to_fr;
use l2o_crypto::proof::groth16::bn128::proof_data::Groth16BN128ProofData;
use serde::Deserialize;
use serde::Serialize;

/// A proof in the Groth16 SNARK.
#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct ProofWithPublicInputs<E: Pairing> {
    /// The `A` element in `G1`.
    pub proof: Proof<E>,
    pub public_inputs: Vec<E::ScalarField>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ProofJson {
    pub pi_a: [String; 3],
    pub pi_b: [[String; 2]; 3],
    pub pi_c: [String; 3],
    pub public_inputs: Vec<String>,
}
impl ProofJson {
    pub fn to_proof_groth16_bn254(&self) -> anyhow::Result<Proof<Bn254>> {
        let a_g1 = G1Affine::from(G1Projective::new(
            str_to_fq(&self.pi_a[0])?,
            str_to_fq(&self.pi_a[1])?,
            str_to_fq(&self.pi_a[2])?,
        ));
        let b_g2 = G2Affine::from(G2Projective::new(
            // x
            Fq2::new(str_to_fq(&self.pi_b[0][0])?, str_to_fq(&self.pi_b[0][1])?),
            // y
            Fq2::new(str_to_fq(&self.pi_b[1][0])?, str_to_fq(&self.pi_b[1][1])?),
            // z,
            Fq2::new(str_to_fq(&self.pi_b[2][0])?, str_to_fq(&self.pi_b[2][1])?),
        ));

        let c_g1 = G1Affine::from(G1Projective::new(
            str_to_fq(&self.pi_c[0])?,
            str_to_fq(&self.pi_c[1])?,
            str_to_fq(&self.pi_c[2])?,
        ));

        Ok(Proof::<Bn254> {
            a: a_g1,
            b: b_g2,
            c: c_g1,
        })
    }
    pub fn to_proof_with_public_inputs_groth16_bn254(
        &self,
    ) -> anyhow::Result<ProofWithPublicInputs<Bn254>> {
        let proof = self.to_proof_groth16_bn254()?;
        let public_inputs = self
            .public_inputs
            .iter()
            .map(|s| str_to_fr(s))
            .collect::<l2o_common::Result<_>>()?;
        Ok(ProofWithPublicInputs::<Bn254> {
            proof: proof,
            public_inputs: public_inputs,
        })
    }
    pub fn from_proof_with_public_inputs_groth16_bn254(proof: &Groth16BN128ProofData) -> Self {
        let a_g1_projective = G1Projective::from(proof.proof.a);
        let b_g2_projective = G2Projective::from(proof.proof.b);
        let c_g1_projective = G1Projective::from(proof.proof.c);
        let public_inputs = proof
            .public_inputs
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>();
        Self {
            pi_a: [
                a_g1_projective.x.to_string(),
                a_g1_projective.y.to_string(),
                a_g1_projective.z.to_string(),
            ],
            pi_b: [
                [
                    b_g2_projective.x.c0.to_string(),
                    b_g2_projective.x.c1.to_string(),
                ],
                [
                    b_g2_projective.y.c0.to_string(),
                    b_g2_projective.y.c1.to_string(),
                ],
                [
                    b_g2_projective.z.c0.to_string(),
                    b_g2_projective.z.c1.to_string(),
                ],
            ],
            pi_c: [
                c_g1_projective.x.to_string(),
                c_g1_projective.y.to_string(),
                c_g1_projective.z.to_string(),
            ],
            public_inputs: public_inputs,
        }
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct VerifyingKeyJson {
    #[serde(rename = "IC")]
    pub ic: Vec<Vec<String>>,

    #[serde(rename = "nPublic")]
    pub n_public: u32,
    pub vk_alpha_1: Vec<String>,
    pub vk_beta_2: Vec<Vec<String>>,
    pub vk_gamma_2: Vec<Vec<String>>,
    pub vk_delta_2: Vec<Vec<String>>,
    pub vk_alphabeta_12: Vec<Vec<Vec<String>>>,
    pub curve: String,
    pub protocol: String,
}

impl VerifyingKeyJson {
    pub fn to_verifying_key_groth16_bn254(self) -> anyhow::Result<VerifyingKey<Bn254>> {
        let alpha_g1 = G1Affine::from(G1Projective::new(
            str_to_fq(&self.vk_alpha_1[0])?,
            str_to_fq(&self.vk_alpha_1[1])?,
            str_to_fq(&self.vk_alpha_1[2])?,
        ));
        let beta_g2 = G2Affine::from(G2Projective::new(
            // x
            Fq2::new(
                str_to_fq(&self.vk_beta_2[0][0])?,
                str_to_fq(&self.vk_beta_2[0][1])?,
            ),
            // y
            Fq2::new(
                str_to_fq(&self.vk_beta_2[1][0])?,
                str_to_fq(&self.vk_beta_2[1][1])?,
            ),
            // z,
            Fq2::new(
                str_to_fq(&self.vk_beta_2[2][0])?,
                str_to_fq(&self.vk_beta_2[2][1])?,
            ),
        ));

        let gamma_g2 = G2Affine::from(G2Projective::new(
            // x
            Fq2::new(
                str_to_fq(&self.vk_gamma_2[0][0])?,
                str_to_fq(&self.vk_gamma_2[0][1])?,
            ),
            // y
            Fq2::new(
                str_to_fq(&self.vk_gamma_2[1][0])?,
                str_to_fq(&self.vk_gamma_2[1][1])?,
            ),
            // z,
            Fq2::new(
                str_to_fq(&self.vk_gamma_2[2][0])?,
                str_to_fq(&self.vk_gamma_2[2][1])?,
            ),
        ));

        let delta_g2 = G2Affine::from(G2Projective::new(
            // x
            Fq2::new(
                str_to_fq(&self.vk_delta_2[0][0])?,
                str_to_fq(&self.vk_delta_2[0][1])?,
            ),
            // y
            Fq2::new(
                str_to_fq(&self.vk_delta_2[1][0])?,
                str_to_fq(&self.vk_delta_2[1][1])?,
            ),
            // z,
            Fq2::new(
                str_to_fq(&self.vk_delta_2[2][0])?,
                str_to_fq(&self.vk_delta_2[2][1])?,
            ),
        ));

        let gamma_abc_g1: Vec<G1Affine> = self
            .ic
            .iter()
            .map(|coords| {
                Ok({
                    G1Affine::from(G1Projective::new(
                        str_to_fq(&coords[0])?,
                        str_to_fq(&coords[1])?,
                        str_to_fq(&coords[2])?,
                    ))
                })
            })
            .collect::<l2o_common::Result<_>>()?;

        Ok(VerifyingKey::<Bn254> {
            alpha_g1: alpha_g1,
            beta_g2: beta_g2,
            gamma_g2: gamma_g2,
            delta_g2: delta_g2,
            gamma_abc_g1: gamma_abc_g1,
        })
    }
}

#[cfg(test)]
mod tests {
    use ark_bn254::Bn254;
    use ark_groth16::Groth16;
    use ark_groth16::VerifyingKey;
    use ark_serialize::CanonicalSerialize;
    use ark_snark::SNARK;

    use super::*;

    #[test]
    fn test_serialize_verify() {
        let vk_json = include_str!("../../assets/example.vkey.json");
        let proof_json = include_str!("../../assets/example_proof.json");
        let p: VerifyingKey<Bn254> = serde_json::from_str::<VerifyingKeyJson>(vk_json)
            .unwrap()
            .to_verifying_key_groth16_bn254()
            .unwrap();
        let proof: ProofWithPublicInputs<Bn254> = serde_json::from_str::<ProofJson>(proof_json)
            .unwrap()
            .to_proof_with_public_inputs_groth16_bn254()
            .unwrap();

        let p2 = Groth16::<Bn254>::process_vk(&p).unwrap();
        let mut uncompressed_bytes = Vec::new();
        p.serialize_uncompressed(&mut uncompressed_bytes).unwrap();
        tracing::info!("{:?}", p);
        tracing::info!("{}", uncompressed_bytes.len());
        let r = Groth16::<Bn254>::verify_proof(&p2, &proof.proof, &proof.public_inputs).unwrap();
        assert_eq!(r, true, "verify proof")
    }
}
