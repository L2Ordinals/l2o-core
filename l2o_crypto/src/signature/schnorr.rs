use k256::schnorr::signature::Verifier;
use k256::schnorr::Signature;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;

pub fn verify(
    public_key: &L2OCompactPublicKey,
    sig: &L2OSignature512,
    msg: &[u8],
) -> anyhow::Result<()> {
    let verifying_key = k256::schnorr::VerifyingKey::from_bytes(&public_key.0)?;
    verifying_key.verify(msg, &Signature::try_from(sig.0.as_ref()).unwrap())?;
    Ok(())
}
