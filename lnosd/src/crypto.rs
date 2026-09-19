use std::fs;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use hex;

pub struct Crypto {
    pub public_key: String,
    private_key: String
}

impl Crypto {
    pub fn new() -> Self {
        let public_key = fs::read("/etc/lnos/public.key")
            .expect("An error occured while opening public.key!");

        let private_key = fs::read("/etc/lnos/private.key")
            .expect("An error occured while opening private.key!");

        Self {
            public_key: hex::encode(public_key),
            private_key: hex::encode(private_key)
        }
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        let key_bytes = hex::decode(self.private_key.clone()).expect("Couldn't decode hex key!");
        
        let fixed_bytes: [u8; 32] = key_bytes.try_into().map_err(|_| "An error occured while parsing the private key into fixed bytes").unwrap();

        let signing_key = SigningKey::from_bytes(&fixed_bytes);

        signing_key.sign(message)
    }

    pub fn verify(&self, public_key: &[u8], raw_signature: &[u8], message: &[u8]) -> bool {
        let public_vec = hex::decode(public_key).unwrap();
        let public_arr: [u8; 32] = public_vec.try_into().expect("Public key must be 32 bytes long!");
        let verifying_key = VerifyingKey::from_bytes(&public_arr).unwrap();
        let signature = Signature::from_bytes(raw_signature.try_into().unwrap());

        verifying_key.verify(message, &signature).is_ok()
    }
}