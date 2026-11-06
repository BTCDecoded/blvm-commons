//! CLI tool for offline PR signing
//!
//! This tool allows maintainers to sign PRs offline using their private keys.
//! It generates the proper signature that can be posted as a comment on GitHub.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

use blvm_commons::crypto::signatures::{SignatureManager, governance_pr_message};
use blvm_sdk::governance::GovernanceKeypair;

#[derive(Parser)]
#[command(name = "sign-pr")]
#[command(about = "Sign a pull request for governance approval")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sign a pull request
    Sign {
        /// Private key file path
        #[arg(short, long)]
        key: String,

        /// Repository name (e.g., "btcdecoded/governance")
        #[arg(short, long)]
        repo: String,

        /// Pull request number
        #[arg(short, long)]
        pr: u64,

        /// Head commit the signature is bound to
        #[arg(long)]
        head: String,
    },
    /// Generate a new keypair
    Generate {
        /// Output directory for keys
        #[arg(short, long, default_value = "./keys")]
        output: String,

        /// Username for the key
        #[arg(short, long)]
        username: String,
    },
    /// Verify a signature
    Verify {
        /// Public key file path
        #[arg(short, long)]
        public_key: String,

        /// Message that was signed
        #[arg(short, long)]
        message: String,

        /// Signature to verify
        #[arg(short, long)]
        signature: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sign {
            key,
            repo,
            pr,
            head,
        } => {
            let _signature = sign_pr(&key, &repo, pr, &head)?;
        }
        Commands::Generate { output, username } => {
            generate_keypair(&output, &username)?;
        }
        Commands::Verify {
            public_key,
            message,
            signature,
        } => {
            verify_signature(&public_key, &message, &signature)?;
        }
    }

    Ok(())
}

fn sign_pr(
    key_path: &str,
    repo: &str,
    pr: u64,
    head: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("🔐 Signing PR #{pr} in {repo}");

    let private_key = fs::read_to_string(key_path)?;
    let secret = hex::decode(private_key.trim())?;
    let keypair = GovernanceKeypair::from_secret_key(&secret)?;

    let message = governance_pr_message(repo, pr, head);
    println!("📝 Message to sign: {message}");

    let signature_manager = SignatureManager::new();
    let signature = signature_manager.create_governance_signature(&message, &keypair)?;

    println!("✅ Signature generated successfully!");
    println!();
    println!("📋 Copy this command to post on GitHub:");
    println!("/governance-sign {signature}");
    println!();
    println!("🔍 Signature details:");
    println!("  Repository: {repo}");
    println!("  PR Number: {pr}");
    println!("  Message: {message}");
    println!("  Signature: {signature}");

    Ok(signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_pr_output_verifies() {
        let manager = SignatureManager::new();
        let keypair = manager.generate_keypair().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let key_path = dir.path().join("maintainer.hex");
        std::fs::write(&key_path, hex::encode(keypair.secret_key)).unwrap();

        let repo = "BTCDecoded/blvm-spec";
        let head = "0123456789abcdef";
        let signature = sign_pr(key_path.to_str().unwrap(), repo, 4, head).unwrap();
        let public_key = hex::encode(keypair.public_key);
        let message = governance_pr_message(repo, 4, head);
        assert_eq!(signature.len(), 128);
        assert!(
            manager
                .verify_governance_signature(&message, &signature, &public_key)
                .unwrap()
        );
        let other = governance_pr_message(repo, 4, "ffffffffffffffff");
        assert!(
            !manager
                .verify_governance_signature(&other, &signature, &public_key)
                .unwrap()
        );
    }
}

fn generate_keypair(output_dir: &str, username: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔑 Generating keypair for {username}");

    fs::create_dir_all(output_dir)?;

    let private_key_path = Path::new(output_dir).join(format!("{username}_private.hex"));
    let public_key_path = Path::new(output_dir).join(format!("{username}_public.hex"));

    let keypair = SignatureManager::new().generate_keypair()?;
    let secret_hex = hex::encode(keypair.secret_key);
    let public_key_hex = hex::encode(keypair.public_key);
    fs::write(&private_key_path, &secret_hex)?;
    fs::write(&public_key_path, &public_key_hex)?;

    println!("✅ Keypair generated successfully!");
    println!("📁 Private key: {}", private_key_path.display());
    println!("📁 Public key: {}", public_key_path.display());
    println!("🔑 Public key (hex): {public_key_hex}");
    println!();
    println!("💾 To add this maintainer to the database:");
    println!(
        "INSERT INTO maintainers (github_username, public_key, layer, active, last_updated) VALUES"
    );
    println!("('{username}', '{public_key_hex}', 1, true, CURRENT_TIMESTAMP);");

    Ok(())
}

fn verify_signature(
    public_key_path: &str,
    message: &str,
    signature: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Verifying signature...");

    // Load public key
    let public_key = fs::read_to_string(public_key_path)?;
    let public_key = public_key.trim();

    // Initialize signature manager
    let signature_manager = SignatureManager::new();

    // Verify signature
    let is_valid = signature_manager.verify_governance_signature(message, signature, public_key)?;

    if is_valid {
        println!("✅ Signature is VALID");
        println!("📝 Message: {message}");
        println!("🔑 Public key: {public_key_path}");
        println!("✍️  Signature: {signature}");
    } else {
        println!("❌ Signature is INVALID");
        println!("📝 Message: {message}");
        println!("🔑 Public key: {public_key_path}");
        println!("✍️  Signature: {signature}");
    }

    Ok(())
}
