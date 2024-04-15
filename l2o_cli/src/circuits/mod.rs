use ark_crypto_primitives::crh::sha256::constraints::Sha256Gadget;
use ark_crypto_primitives::crh::sha256::Sha256;
use ark_crypto_primitives::crh::CRHSchemeGadget;
use ark_ff::PrimeField;
use ark_r1cs_std::alloc::AllocVar;
use ark_r1cs_std::boolean::Boolean;
use ark_r1cs_std::eq::EqGadget;
use ark_r1cs_std::fields::fp::FpVar;
use ark_r1cs_std::uint8::UInt8;
use ark_r1cs_std::ToBitsGadget;
use ark_r1cs_std::ToBytesGadget;
use ark_relations::r1cs::ConstraintSynthesizer;

#[derive(Clone)]
pub struct BlockCircuit<F: PrimeField> {
    pub block_hash: [F; 2],
    pub block_payload: Vec<u8>,
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
