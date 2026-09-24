use std::env;
use std::fs;

use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

pub struct Provider;

impl Provider {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_args(&self) {
        let args: Vec<String> = env::args().collect();
        for arg in args {
            match arg.as_str() {
                "generate-keys" => self.generate_keys(),
                other => eprintln!("Unknown argument: {}", other)
            }
        } 
    }

    fn generate_keys(&self) {
        let mut csprng = OsRng{};
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);

        let veryfying_key: VerifyingKey = signing_key.verifying_key();

        println!("{:#?}", signing_key.to_bytes());
        fs::write("/etc/lnos/private.key", signing_key.to_bytes()).expect("Couldn't save private.key file!");
        fs::write("/etc/lnos/public.key", veryfying_key.to_bytes()).expect("Couldn't save public.key file!");

    }
}