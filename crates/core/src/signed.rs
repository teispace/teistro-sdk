//! The check every signed pack passes (`docs/03-design/zone-packs.md` §2,
//! `published-overrides.md` §5): a detached Ed25519 signature over a pack's
//! exact bytes, against a key the caller fixes at its own build. The SDK
//! embeds none.

use crate::error::Error;

/// Whether `signature` is `key`'s over `pack`.
///
/// ```
/// use teistro_core::signed::verify;
///
/// // A key whose y-coordinate, 2, has no curve point is refused before
/// // any signature is checked.
/// let mut key = [0_u8; 32];
/// key[0] = 2;
/// assert_eq!(verify(b"{}", &[0; 64], &key).unwrap_err().field(), Some("key"));
/// ```
///
/// # Errors
///
/// `INVALID_ARG` on `key` for one that is not 32 bytes of a curve point,
/// and on `signature` for one that is not 64 bytes or does not verify (the
/// pack was altered, or signed by another key).
pub fn verify(pack: &[u8], signature: &[u8], key: &[u8]) -> Result<(), Error> {
    let key = <[u8; ed25519_dalek::PUBLIC_KEY_LENGTH]>::try_from(key)
        .ok()
        .and_then(|bytes| ed25519_dalek::VerifyingKey::from_bytes(&bytes).ok())
        .ok_or_else(|| {
            Error::invalid_arg(format!(
                "the publisher's key is {} bytes of an Ed25519 point",
                ed25519_dalek::PUBLIC_KEY_LENGTH
            ))
            .with_field("key")
        })?;
    let signature = <[u8; ed25519_dalek::SIGNATURE_LENGTH]>::try_from(signature)
        .map(|bytes| ed25519_dalek::Signature::from_bytes(&bytes))
        .map_err(|_| {
            Error::invalid_arg(format!(
                "a detached Ed25519 signature is {} bytes, not {}",
                ed25519_dalek::SIGNATURE_LENGTH,
                signature.len()
            ))
            .with_field("signature")
        })?;
    key.verify_strict(pack, &signature).map_err(|_| {
        Error::invalid_arg(
            "the signature does not verify: the pack was altered, or signed by another key",
        )
        .with_field("signature")
    })
}
