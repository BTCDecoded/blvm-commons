//! M4: the tree has no committed RSA private key, and boot refuses a missing
//! or placeholder webhook secret and GitHub App id.

use blvm_commons::config::AppConfig;
use blvm_commons::github::client::GitHubClient;
use rsa::RsaPrivateKey;
use rsa::pkcs8::EncodePrivateKey;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn set_boot_env(app_id: &str, secret: &str) {
    // SAFETY: this test binary is the only process touching these vars, and
    // ENV_LOCK keeps the writes single-threaded.
    unsafe {
        std::env::set_var("GITHUB_APP_ID", app_id);
        std::env::set_var("GITHUB_WEBHOOK_SECRET", secret);
    }
}

fn clear_boot_env() {
    unsafe {
        std::env::remove_var("GITHUB_APP_ID");
        std::env::remove_var("GITHUB_WEBHOOK_SECRET");
    }
}

#[test]
fn committed_rsa_private_key_is_gone() {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("test_fixtures/test_rsa_key.pem");
    assert!(
        !path.exists(),
        "test_fixtures/test_rsa_key.pem must not be committed"
    );
    let production = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config/production.toml"),
    )
    .unwrap();
    assert!(!production.contains("your-webhook-secret-here"));
    assert!(!production.contains("app_id = 12345"));
    let compose = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("deployment/docker-compose.yml"),
    )
    .unwrap();
    assert!(!compose.contains(":-123456"));
    assert!(compose.contains("GITHUB_WEBHOOK_SECRET:?"));
}

#[test]
fn missing_webhook_secret_does_not_load() {
    let _guard = ENV_LOCK.lock().unwrap();
    clear_boot_env();
    unsafe {
        std::env::set_var("GITHUB_APP_ID", "424242");
        std::env::remove_var("GITHUB_WEBHOOK_SECRET");
    }
    let err = AppConfig::load().expect_err("missing secret");
    assert!(err.to_string().contains("GITHUB_WEBHOOK_SECRET"));
    set_boot_env("424242", "   ");
    let err = AppConfig::load().expect_err("blank secret");
    assert!(err.to_string().contains("placeholder"));
    clear_boot_env();
}

#[test]
fn placeholder_webhook_secret_does_not_load() {
    let _guard = ENV_LOCK.lock().unwrap();
    set_boot_env("424242", "your_webhook_secret_here");
    let err = AppConfig::load().expect_err("placeholder secret");
    assert!(err.to_string().contains("placeholder"));
    set_boot_env("424242", "your-webhook-secret-here");
    let err = AppConfig::load().expect_err("dashed placeholder");
    assert!(err.to_string().contains("placeholder"));
    clear_boot_env();
}

#[test]
fn placeholder_app_id_does_not_load() {
    let _guard = ENV_LOCK.lock().unwrap();
    set_boot_env("123456", "a-real-webhook-secret");
    let err = AppConfig::load().expect_err("placeholder app id");
    assert!(err.to_string().contains("GITHUB_APP_ID"));
    clear_boot_env();
}

#[test]
fn configured_secret_and_app_id_load() {
    let _guard = ENV_LOCK.lock().unwrap();
    set_boot_env("424242", "a-real-webhook-secret");
    let config = AppConfig::load().expect("configured boot values load");
    assert_eq!(config.github_app_id, 424242);
    assert_eq!(config.github_webhook_secret, "a-real-webhook-secret");
    clear_boot_env();
}

#[tokio::test]
async fn generated_rsa_key_parses_as_a_github_app_key() {
    let temp = tempfile::tempdir().unwrap();
    let mut rng = rsa::rand_core::OsRng;
    let key = RsaPrivateKey::new(&mut rng, 2048).expect("generate");
    let pem = key.to_pkcs8_pem(rsa::pkcs8::LineEnding::LF).expect("pem");
    let path = temp.path().join("app.pem");
    std::fs::write(&path, pem.as_bytes()).unwrap();
    GitHubClient::new(424242, path.to_str().unwrap()).expect("generated key is a usable app key");
}
