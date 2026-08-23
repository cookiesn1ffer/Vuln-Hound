//! Live check that dynamic model listing actually works against real
//! providers. `#[ignore]`d by default — run explicitly:
//!   cargo test --test live_models -- --ignored --nocapture

use std::path::PathBuf;

use vuln_hound_lib::llm::client::fetch_provider_models;

/// Loads `.env` from the repo root (one level up from `src-tauri`) so the
/// test env vars don't need to be exported by hand every run.
fn load_env() {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".env");
    let _ = dotenvy::from_path(env_path);
}

#[tokio::test]
#[ignore]
async fn lists_models_from_lmstudio() {
    let models = fetch_provider_models("http://localhost:1234/v1", None)
        .await
        .expect("LM Studio must be running with the server started");
    println!("LM Studio models: {models:?}");
    assert!(!models.is_empty());
}

#[tokio::test]
#[ignore]
async fn lists_models_from_opencode_zen() {
    let models = fetch_provider_models("https://opencode.ai/zen/v1", None)
        .await
        .expect("OpenCode Zen models list should be public");
    println!(
        "OpenCode Zen: {} models, first 5: {:?}",
        models.len(),
        &models[..5.min(models.len())]
    );
    assert!(!models.is_empty());
}

#[tokio::test]
#[ignore]
async fn lists_models_from_opencode_go() {
    let models = fetch_provider_models("https://opencode.ai/zen/go/v1", None)
        .await
        .expect("OpenCode Go models list should be public");
    println!(
        "OpenCode Go: {} models, first 5: {:?}",
        models.len(),
        &models[..5.min(models.len())]
    );
    assert!(!models.is_empty());
}

#[tokio::test]
#[ignore]
async fn lists_models_from_groq() {
    load_env();
    let key = std::env::var("VULN_HOUND_TEST_API_KEY").expect("set VULN_HOUND_TEST_API_KEY");
    let models = fetch_provider_models("https://api.groq.com/openai/v1", Some(&key))
        .await
        .expect("Groq models list requires a valid key");
    println!("Groq models: {models:?}");
    assert!(!models.is_empty());
}
