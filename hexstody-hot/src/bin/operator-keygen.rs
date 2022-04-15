use anyhow::{bail, Context, Result};
use clap::Parser;
use p256::{SecretKey};
use pkcs8::{EncodePrivateKey, EncodePublicKey};
use rand::rngs::OsRng;
use rpassword;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{PathBuf};

/// Program to generate NIST P-256 keypair and store them to files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The path to the output file
    #[clap(short, long, parse(from_os_str), default_value = "operator-key")]
    output: PathBuf,
    /// Flag that enables password encryption
    #[clap(short, long)]
    password: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Request password
    let password = if args.password {
        let password_entered = rpassword::prompt_password("Enter password: ")?;
        let password_repeated = rpassword::prompt_password("Repeat password: ")?;
        if password_entered != password_repeated {
            bail!("Passwords do not match");
        };
        Some(password_entered)
    } else {
        None
    };

    // Generate secret key
    let secret_key = SecretKey::random(&mut OsRng);
    let encoded_secret_key = match password {
        Some(password) => {
            secret_key.to_pkcs8_encrypted_pem(&mut OsRng, password, Default::default())?
        }
        None => secret_key.to_pkcs8_pem(Default::default())?,
    };

    // Write secret key to file
    let mut prv_key_path = args.output.clone();
    prv_key_path.set_extension("prv");
    let path = prv_key_path.clone();
    let mut prv_key_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(prv_key_path)
        .with_context(|| format!("Failed to open file {}", path.display()))?;
    prv_key_file.write_all(encoded_secret_key.as_bytes())?;

    // Generate public key
    let public_key = secret_key.public_key();
    let encoded_public_key = public_key.to_public_key_pem(Default::default())?;

    // Write public key to file
    let mut pub_key_path = args.output.clone();
    pub_key_path.set_extension("pub");
    let path = pub_key_path.clone();
    let mut pub_key_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(pub_key_path)
        .with_context(|| format!("Failed to open file {}", path.display()))?;
    pub_key_file.write_all(encoded_public_key.as_bytes())?;

    Ok(())
}
