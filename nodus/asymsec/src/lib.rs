mod ed25519;

pub const SIG_LEN: usize = 64;
pub const PUB_KEY_LEN: usize = 32;
pub trait AsymSecHandler {
    fn sign(&self, data: &[u8]) -> [u8; SIG_LEN];
    fn verify(&self, data: &[u8], sig_bytes: &[u8; SIG_LEN]) -> bool;
    fn get_public_key(&self) -> [u8; PUB_KEY_LEN];
}

// not sure if this is the right way to go about this... letting the lib decide what impl to use
pub fn get_implementation() -> Box<dyn AsymSecHandler> {
    Box::new(ed25519::Ed25519AsymSecHandler::new())
}
