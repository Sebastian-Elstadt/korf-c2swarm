use crate::{AsymSecHandler, PUB_KEY_LEN, SIG_LEN};
use ed25519_dalek::{Signature, Signer, SigningKey};
use rand::rngs::OsRng;

pub struct Ed25519AsymSecHandler {
    signing_key: SigningKey,
}

impl Ed25519AsymSecHandler {
    pub fn new() -> Self {
        let mut csprng = OsRng;
        Ed25519AsymSecHandler {
            signing_key: SigningKey::generate(&mut csprng),
        }
    }
}

impl AsymSecHandler for Ed25519AsymSecHandler {
    fn sign(&self, data: &[u8]) -> [u8; SIG_LEN] {
        self.signing_key.sign(data).to_bytes()
    }

    fn verify(&self, data: &[u8], sig_bytes: &[u8; SIG_LEN]) -> bool {
        let sig = Signature::from_bytes(sig_bytes);
        self.signing_key.verify(data, &sig).is_ok()
    }

    fn get_public_key(&self) -> [u8; PUB_KEY_LEN] {
        self.signing_key.verifying_key().to_bytes()
    }
}
