// Ed25519 helpers built on [`ed25519_dalek`]. Shared by kommander and nodus for aligned dependency versions.

pub use ed25519_dalek::{
    Signature, Signer, SigningKey, Verifier, VerifyingKey,
};

pub const SIGNATURE_LENGTH: usize = 64;
pub const PUBLIC_KEY_LENGTH: usize = 32;

/// Verify a detached Ed25519 signature.
#[must_use]
pub fn verify_signature(
    verifying_key_bytes: &[u8; PUBLIC_KEY_LENGTH],
    message: &[u8],
    signature_bytes: &[u8; SIGNATURE_LENGTH],
) -> bool {
    let Ok(verifying_key) = VerifyingKey::from_bytes(verifying_key_bytes) else {
        return false;
    };
    let Ok(signature) = Signature::from_slice(signature_bytes) else {
        return false;
    };
    verifying_key.verify_strict(message, &signature).is_ok()
}
