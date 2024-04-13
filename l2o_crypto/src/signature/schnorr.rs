use k256::schnorr::signature::Signer;
use k256::schnorr::signature::Verifier;
use k256::schnorr::Signature;
use k256::schnorr::SigningKey;
use l2o_common::common::data::signature::L2OCompactPublicKey;
use l2o_common::common::data::signature::L2OSignature512;

pub fn verify_sig(
    public_key: &L2OCompactPublicKey,
    sig: &L2OSignature512,
    msg: &[u8],
) -> l2o_common::Result<()> {
    let verifying_key = k256::schnorr::VerifyingKey::from_bytes(&public_key.0)?;
    verifying_key.verify(msg, &Signature::try_from(sig.0.as_ref())?)?;
    Ok(())
}

pub fn sign_msg(signing_key: &SigningKey, msg: &[u8]) -> l2o_common::Result<L2OSignature512> {
    Ok(L2OSignature512(signing_key.sign(msg).to_bytes()))
}
