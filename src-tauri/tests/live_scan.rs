//! Live integration test that drives the real scan loop against a real local
//! LLM (Ollama/LM Studio) and the seeded fixture app. Not run in normal CI —
//! requires a locally running OpenAI-compatible server, so it's `#[ignore]`d
//! by default. Run explicitly with:
//!
//!   VULN_HOUND_TEST_BASE_URL=http://localhost:11434/v1 \
//!   VULN_HOUND_TEST_MODEL=huihui_ai/dolphin3-abliterated:8b \
//!   cargo test --test live_scan -- --ignored --nocapture

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tokio_util::sync::CancellationToken;

use vuln_hound_lib::agent::events::ScanEvent;
use vuln_hound_lib::agent::loop_::run_scan_session;
use vuln_hound_lib::llm::client::OpenAiCompatClient;
use vuln_hound_lib::playbook::load_playbook;
use vuln_hound_lib::security::mode::ScanMode;

#[tokio::test]
#[ignore]
async fn live_scan_finds_seeded_issues() {
    let base_url = std::env::var("VULN_HOUND_TEST_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
    let model = std::env::var("VULN_HOUND_TEST_MODEL")
        .unwrap_or_else(|_| "huihui_ai/dolphin3-abliterated:8b".to_string());
    let api_key = std::env::var("VULN_HOUND_TEST_API_KEY").ok();

    let client = OpenAiCompatClient::new(base_url, api_key, model);

    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("fixtures/vulnerable-sample-app");
    assert!(fixture_dir.exists(), "fixture dir must exist: {fixture_dir:?}");

    let playbook = load_playbook().expect("bundled playbook must parse");
    let group = playbook
        .group("injection-execution")
        .expect("injection-execution group must exist")
        .clone();

    let events: Arc<Mutex<Vec<ScanEvent>>> = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();
    let emit = move |event: ScanEvent| {
        println!("EVENT: {event:?}");
        events_clone.lock().unwrap().push(event);
    };

    run_scan_session(
        emit,
        "test-session".to_string(),
        fixture_dir,
        ScanMode::Test,
        group,
        client,
        CancellationToken::new(),
    )
    .await;

    let events = events.lock().unwrap();
    let findings: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            ScanEvent::Finding { finding, .. } => Some(finding),
            _ => None,
        })
        .collect();

    println!("\n=== {} findings reported ===", findings.len());
    for f in &findings {
        println!("- [{:?}] {} ({})", f.severity, f.title, f.category_id);
        for ev in &f.evidence {
            println!("    {}: {:?}", ev.file_path, ev.snippet);
        }
    }

    let final_status = events.iter().rev().find_map(|e| match e {
        ScanEvent::Status { status, .. } => Some(status.clone()),
        _ => None,
    });
    println!("\nfinal status: {final_status:?}");

    assert!(!findings.is_empty(), "expected the model to find at least one seeded issue");
    assert!(
        findings.iter().any(|f| f.category_id == "sql-injection"),
        "expected the seeded SQL injection in server.js to be found"
    );
}
