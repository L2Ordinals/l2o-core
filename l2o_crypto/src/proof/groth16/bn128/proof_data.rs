use ark_bn254::Bn254;
use ark_bn254::Fq2;
use ark_bn254::Fr;
use ark_bn254::G1Affine;
use ark_bn254::G1Projective;
use ark_bn254::G2Affine;
use ark_bn254::G2Projective;
use ark_groth16::Proof;
use ark_serialize::CanonicalDeserialize;
use ark_serialize::CanonicalSerialize;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

use super::verifier_data::str_to_fq;
use super::verifier_data::str_to_fr;

#[derive(Clone, Debug, PartialEq, CanonicalSerialize, CanonicalDeserialize)]
pub struct Groth16BN128ProofData {
    pub proof: Proof<Bn254>,
    pub public_inputs: Vec<Fr>,
}

impl Serialize for Groth16BN128ProofData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw = Groth16ProofSerializable::from_proof_with_public_inputs_groth16_bn254(&self);

        raw.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Groth16BN128ProofData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let raw = Groth16ProofSerializable::deserialize(deserializer)?;
        let proof = raw.to_proof_with_public_inputs_groth16_bn254();

        if proof.is_ok() {
            Ok(proof.unwrap())
        } else {
            Err(Error::custom("invalid Groth16BN128ProofData JSON"))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
struct Groth16ProofSerializable {
    pub pi_a: [String; 3],
    pub pi_b: [[String; 2]; 3],
    pub pi_c: [String; 3],
    pub public_inputs: Vec<String>,
}

impl Groth16ProofSerializable {
    pub fn to_proof_groth16_bn254(&self) -> anyhow::Result<Proof<Bn254>, ()> {
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
    ) -> anyhow::Result<Groth16BN128ProofData, ()> {
        let proof = self.to_proof_groth16_bn254()?;
        let mut public_inputs: Vec<Fr> = Vec::new();
        for pi in self.public_inputs.iter() {
            let v = str_to_fr(pi)?;
            public_inputs.push(v);
        }
        Ok(Groth16BN128ProofData {
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
