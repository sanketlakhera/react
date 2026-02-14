//! E2E Tests for React Components
//!
//! These tests run React components in a simulated DOM environment
//! via Node.js and verify behavior including state updates and interactions.

use std::path::PathBuf;
use std::process::Command;

fn get_e2e_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/e2e")
}

fn run_e2e_test(fixture_name: &str) -> (bool, String) {
    let e2e_dir = get_e2e_dir();
    let fixture_path = e2e_dir.join("fixtures").join(fixture_name);
    
    let output = Command::new("node")
        .current_dir(&e2e_dir)
        .arg("runner.js")
        .arg(&fixture_path)
        .output()
        .expect("Failed to execute Node.js");
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    if !stderr.is_empty() {
        return (false, format!("stderr: {}", stderr));
    }
    
    // Parse the JSON result
    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let success = result["success"].as_bool().unwrap_or(false);
        (success, stdout)
    } else {
        (false, format!("Failed to parse output: {}", stdout))
    }
}

#[test]
#[ignore] // Ignore by default - requires npm install
fn test_counter_component() {
    let (success, output) = run_e2e_test("counter.js");
    assert!(success, "Counter E2E test failed: {}", output);
}

#[test]
fn test_e2e_infrastructure_exists() {
    let e2e_dir = get_e2e_dir();
    assert!(e2e_dir.join("runner.js").exists(), "runner.js should exist");
    assert!(e2e_dir.join("package.json").exists(), "package.json should exist");
    assert!(e2e_dir.join("fixtures/counter.js").exists(), "counter.js fixture should exist");
}
