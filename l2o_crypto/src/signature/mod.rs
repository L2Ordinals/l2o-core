#[cfg(test)]
mod tests {

    use musig2::secp::Point;

    #[test]
    fn test_blake3_two_to_one() -> anyhow::Result<()> {
      let signature_bytes = hex::decode("403B12B0D8555A344175EA7EC746566303321E5DBFA8BE6F091635163ECA79A8585ED3E3170807E7C03B720FC54C7B23897FCBA0E9D0B4A06894CFD249F22367")?;
      let mut public_key_bytes = [0u8; 32];
      hex::decode_to_slice("778CAA53B4393AC467774D09497A87224BF9FAB6F6E68B23086497324D6FD117", &mut public_key_bytes)?;

      let public_key = Point::lift_x(&public_key_bytes)?;
      let message_bytes = hex::decode("99999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999").unwrap();

      musig2::verify_single(public_key, &*signature_bytes, &message_bytes)?;
      /*let p: &[u8] = &signature_bytes;
      let signature = Signature::try_from(p)?;
      let public_key = Public::from_bytes(&public_key_bytes)?;
      public_key.verify(&message_bytes, &signature)?;*/
      /* s
        //
        // Signing
        //
        let signing_key = SigningKey::random(&mut OsRng); // serialize with `.to_bytes()`
        let verifying_key_bytes = signing_key.verifying_key().to_bytes(); // 32-bytes

        let message = b"Schnorr signatures prove knowledge of a secret in the random oracle model";
        let signature = signing_key.sign(message); // returns `k256::schnorr::Signature`
        signature.to_bytes()

        //
        // Verification
        //
        let verifying_key = VerifyingKey::from_bytes(&verifying_key_bytes)?;
        verifying_key.
        verifying_key.verify(message, &signature)?;*/


        Ok(())
    }
}
