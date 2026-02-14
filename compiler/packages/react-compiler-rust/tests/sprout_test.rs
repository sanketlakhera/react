//! Sprout Runtime Verification Tests
//!
//! Tests that compiled code produces the same output as the original.
//! Uses Node.js to execute both versions and compares results.

use react_compiler_rust::compile;
use react_compiler_rust::sprout::verify_fixture;
use oxc_span::SourceType;
use std::fs;
use std::path::PathBuf;

fn sprout_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/sprout")
}

/// Extract the FIXTURE_ENTRYPOINT from source code
fn extract_fixture_entrypoint(source: &str) -> Option<String> {
    if let Some(start_idx) = source.find("const FIXTURE_ENTRYPOINT") {
        let rest = &source[start_idx..];
        if let Some(end_idx) = rest.find("};") {
            return Some(rest[..end_idx + 2].to_string());
        }
    }
    None
}

/// Run sprout verification for a fixture
fn run_sprout_test(filename: &str) -> Result<(), String> {
    let path = sprout_dir().join(filename);
    let original_code = fs::read_to_string(&path)
        .map_err(|e| format!("Read error: {}", e))?;
    
    let fixture_entrypoint = extract_fixture_entrypoint(&original_code)
        .ok_or_else(|| "No FIXTURE_ENTRYPOINT found".to_string())?;
    
    let compiled_result = std::panic::catch_unwind(|| {
        compile(&original_code, SourceType::mjs())
    });
    
    let mut compiled_code = match compiled_result {
        Ok(Ok(code)) => code,
        Ok(Err(e)) => return Err(format!("Compile error: {}", e)),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            return Err(format!("Compile panic: {}", msg));
        }
    };
    
    // Mock _c function and append entrypoint
    let mock_cache = "function _c(size) { return new Array(size).fill(undefined); }";
    compiled_code = format!("{}\n{}\n\n{}", mock_cache, compiled_code, fixture_entrypoint);
    
    let result = verify_fixture(&original_code, &compiled_code);
    
    if result.passed {
        println!("✓ {} - Output: {}", filename, result.original_output.trim());
        Ok(())
    } else {
        Err(format!(
            "Runtime mismatch for {}\n  Original: {}\n  Compiled: {}\n  Original error: {:?}\n  Compiled error: {:?}",
            filename,
            result.original_output.trim(),
            result.compiled_output.trim(),
            result.original_error,
            result.compiled_error
        ))
    }
}

#[test]
fn sprout_pure_arithmetic() {
    let result = run_sprout_test("pure_arithmetic.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_object_access() {
    let result = run_sprout_test("object_access.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_conditionals() {
    let result = run_sprout_test("conditionals.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_binary_ops() {
    let result = run_sprout_test("binary_ops.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_while_loop() {
    let result = run_sprout_test("while_loop.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_for_loop_basic() {
    let result = run_sprout_test("for_loop_basic.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_operators_comprehensive() {
    let result = run_sprout_test("operators_comprehensive.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_update_expressions() {
    let result = run_sprout_test("update_expressions.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_break_continue() {
    let result = run_sprout_test("break_continue.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_simple_switch() {
    let result = run_sprout_test("simple_switch.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn sprout_switch() {
    let result = run_sprout_test("switch.js");
    assert!(result.is_ok(), "{}", result.unwrap_err());
}
