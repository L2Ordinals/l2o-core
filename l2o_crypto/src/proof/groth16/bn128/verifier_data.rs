use ark_bn254::Bn254;
use ark_bn254::Fq2;
use ark_bn254::G1Affine;
use ark_bn254::G1Projective;
use ark_bn254::G2Affine;
use ark_bn254::G2Projective;
use ark_groth16::VerifyingKey;
use l2o_common::str_to_fq;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

#[derive(Clone, PartialEq, Debug)]
pub struct Groth16BN128VerifierData(pub VerifyingKey<Bn254>);

impl Serialize for Groth16BN128VerifierData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw = Groth16VerifierDataSerializable::from_vk(&self.0);

        raw.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Groth16BN128VerifierData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let raw = Groth16VerifierDataSerializable::deserialize(deserializer)?;

        if let Ok(vk) = raw.to_vk() {
            Ok(Groth16BN128VerifierData(vk))
        } else {
            Err(Error::custom("invalid Groth16BN128VerifierData JSON"))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Groth16VerifierDataSerializable {
    pub vk_alpha_1: [String; 3],
    pub vk_beta_2: [[String; 2]; 3],
    pub vk_gamma_2: [[String; 2]; 3],
    pub vk_delta_2: [[String; 2]; 3],
    //    pub vk_alphabeta_12: [[[String; 2]; 3]; 2],
    pub ic: [[String; 3]; 3],
}

impl Groth16VerifierDataSerializable {
    pub fn to_vk(&self) -> l2o_common::Result<VerifyingKey<Bn254>> {
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

        let mut gamma_abc_g1: Vec<G1Affine> = Vec::new();
        for coords in self.ic.iter() {
            let c = G1Affine::from(G1Projective::new(
                str_to_fq(&coords[0])?,
                str_to_fq(&coords[1])?,
                str_to_fq(&coords[2])?,
            ));
            gamma_abc_g1.push(c);
        }

        Ok(VerifyingKey::<Bn254> {
            alpha_g1: alpha_g1,
            beta_g2: beta_g2,
            gamma_g2: gamma_g2,
            delta_g2: delta_g2,
            gamma_abc_g1: gamma_abc_g1,
        })
    }

    pub fn from_vk(vk: &VerifyingKey<Bn254>) -> Self {
        let vk_alpha_1_projective = G1Projective::from(vk.alpha_g1);
        let beta_g2_projective = G2Projective::from(vk.beta_g2);
        let gamma_g2_projective = G2Projective::from(vk.gamma_g2);
        let delta_g2_projective = G2Projective::from(vk.delta_g2);
        let ic_projective = vk
            .gamma_abc_g1
            .iter()
            .map(|x| G1Projective::from(*x))
            .collect::<Vec<_>>();

        Self {
            vk_alpha_1: [
                vk_alpha_1_projective.x.to_string(),
                vk_alpha_1_projective.y.to_string(),
                vk_alpha_1_projective.z.to_string(),
            ],
            vk_beta_2: [
                [
                    beta_g2_projective.x.c0.to_string(),
                    beta_g2_projective.x.c1.to_string(),
                ],
                [
                    beta_g2_projective.y.c0.to_string(),
                    beta_g2_projective.y.c1.to_string(),
                ],
                [
                    beta_g2_projective.z.c0.to_string(),
                    beta_g2_projective.z.c1.to_string(),
                ],
            ],
            vk_gamma_2: [
                [
                    gamma_g2_projective.x.c0.to_string(),
                    gamma_g2_projective.x.c1.to_string(),
                ],
                [
                    gamma_g2_projective.y.c0.to_string(),
                    gamma_g2_projective.y.c1.to_string(),
                ],
                [
                    gamma_g2_projective.z.c0.to_string(),
                    gamma_g2_projective.z.c1.to_string(),
                ],
            ],
            vk_delta_2: [
                [
                    delta_g2_projective.x.c0.to_string(),
                    delta_g2_projective.x.c1.to_string(),
                ],
                [
                    delta_g2_projective.y.c0.to_string(),
                    delta_g2_projective.y.c1.to_string(),
                ],
                [
                    delta_g2_projective.z.c0.to_string(),
                    delta_g2_projective.z.c1.to_string(),
                ],
            ],
            ic: [
                [
                    ic_projective[0].x.to_string(),
                    ic_projective[0].y.to_string(),
                    ic_projective[0].z.to_string(),
                ],
                [
                    ic_projective[1].x.to_string(),
                    ic_projective[1].y.to_string(),
                    ic_projective[1].z.to_string(),
                ],
                [
                    ic_projective[2].x.to_string(),
                    ic_projective[2].y.to_string(),
                    ic_projective[2].z.to_string(),
                ],
            ],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Groth16ProofSerializable {
    pub pi_a: [String; 3],
    pub pi_b: [String; 3],
    pub pi_c: [String; 3],
    pub public_inputs: Vec<String>,
}
