//! Release signing tool for the auto-updater.
//!
//! ```bash
//! blogwriter-sign keygen <keyfile>          # generate a keypair, print the public key
//! blogwriter-sign pubkey <keyfile>          # print the public key for an existing key
//! blogwriter-sign sign <keyfile> <artifact> # print the base64 signature of an artifact
//! ```
//!
//! The private key is a base64 32-byte ed25519 seed. Keep it OUT of the
//! repository; the matching public key is embedded in `src/updater.rs`.

use base64::Engine;
use ed25519_dalek::{Signer, SigningKey};

fn b64() -> base64::engine::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

fn load_key(path: &str) -> SigningKey {
    let raw = std::fs::read_to_string(path).expect("read key file");
    let bytes = b64().decode(raw.trim()).expect("base64 decode key");
    let seed: [u8; 32] = bytes.as_slice().try_into().expect("key must be 32 bytes");
    SigningKey::from_bytes(&seed)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("keygen") => {
            let path = args.get(2).expect("usage: keygen <keyfile>");
            if std::path::Path::new(path).exists() {
                panic!("{path} already exists — refusing to overwrite");
            }
            let key = SigningKey::generate(&mut rand::rngs::OsRng);
            std::fs::write(path, b64().encode(key.to_bytes())).expect("write key");
            println!("private key written to {path} — keep it safe and OUT of git");
            println!("public key: {}", b64().encode(key.verifying_key().to_bytes()));
        }
        Some("pubkey") => {
            let key = load_key(args.get(2).expect("usage: pubkey <keyfile>"));
            println!("{}", b64().encode(key.verifying_key().to_bytes()));
        }
        Some("sign") => {
            let key = load_key(args.get(2).expect("usage: sign <keyfile> <artifact>"));
            let artifact = std::fs::read(args.get(3).expect("usage: sign <keyfile> <artifact>"))
                .expect("read artifact");
            println!("{}", b64().encode(key.sign(&artifact).to_bytes()));
        }
        _ => eprintln!("usage: blogwriter-sign keygen|pubkey|sign ..."),
    }
}
