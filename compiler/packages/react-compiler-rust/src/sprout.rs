//! Sprout Runtime Verification
//!
//! Verifies semantic equivalence by executing both original and compiled
//! JavaScript via Node.js and comparing outputs.

use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;

/// Result of running Sprout verification
#[derive(Debug)]
pub struct SproutResult {
    pub original_output: String,
    pub compiled_output: String,
    pub original_error: Option<String>,
    pub compiled_error: Option<String>,
    pub passed: bool,
}

/// Execute JavaScript code via Node.js and capture output
fn execute_js(code: &str) -> Result<(String, Option<String>), std::io::Error> {
    // Create a temporary file with the JS code
    let mut temp_file = NamedTempFile::with_suffix(".mjs")?;
    temp_file.write_all(code.as_bytes())?;
    temp_file.flush()?;

    // Execute with Node.js
    let output = Command::new("node")
        .arg(temp_file.path())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok((stdout, None))
    } else {
        Ok((stdout, Some(stderr)))
    }
}

/// Generate runner code that executes the fixture and captures results
fn generate_runner(fixture_code: &str) -> String {
    format!(
        r#"
{fixture_code}

// Execute the fixture entrypoint
if (typeof FIXTURE_ENTRYPOINT !== 'undefined') {{
    const {{ fn, params }} = FIXTURE_ENTRYPOINT;
    try {{
        const result = fn(...params);
        console.log(JSON.stringify({{ success: true, result }}));
    }} catch (error) {{
        console.log(JSON.stringify({{ success: false, error: error.message }}));
    }}
}} else {{
    console.log(JSON.stringify({{ success: false, error: "No FIXTURE_ENTRYPOINT defined" }}));
}}
"#
    )
}

/// Verify a fixture by comparing original and compiled outputs
pub fn verify_fixture(original_code: &str, compiled_code: &str) -> SproutResult {
    // Generate runner code for both versions
    let original_runner = generate_runner(original_code);
    let compiled_runner = generate_runner(compiled_code);

    // Execute both
    let (original_output, original_error) = execute_js(&original_runner)
        .unwrap_or_else(|e| (String::new(), Some(e.to_string())));
    
    let (compiled_output, compiled_error) = execute_js(&compiled_runner)
        .unwrap_or_else(|e| (String::new(), Some(e.to_string())));

    // Compare results
    let passed = original_output.trim() == compiled_output.trim()
        && original_error.is_none()
        && compiled_error.is_none();

    SproutResult {
        original_output,
        compiled_output,
        original_error,
        compiled_error,
        passed,
    }
}

/// Run sprout verification on a fixture file
pub fn verify_fixture_file(fixture_path: &Path, compile_fn: impl Fn(&str) -> String) -> SproutResult {
    let original_code = std::fs::read_to_string(fixture_path)
        .expect("Failed to read fixture file");
    
    let compiled_code = compile_fn(&original_code);
    
    verify_fixture(&original_code, &compiled_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple_js() {
        let code = r#"console.log(JSON.stringify({ success: true, result: 42 }));"#;
        let (output, error) = execute_js(code).unwrap();
        
        assert!(error.is_none());
        assert!(output.contains("42"));
    }

    #[test]
    fn test_verify_identical_code() {
        let code = r#"
function add(a, b) {
    return a + b;
}

const FIXTURE_ENTRYPOINT = {
    fn: add,
    params: [1, 2],
};
"#;
        
        let result = verify_fixture(code, code);
        assert!(result.passed, "Identical code should pass: {:?}", result);
    }

    #[test]
    fn test_verify_semantically_equivalent() {
        let original = r#"
function add(a, b) {
    return a + b;
}

const FIXTURE_ENTRYPOINT = {
    fn: add,
    params: [5, 3],
};
"#;
        
        // Different style but same semantics
        let compiled = r#"
function add(a, b) {
    const result = a + b;
    return result;
}

const FIXTURE_ENTRYPOINT = {
    fn: add,
    params: [5, 3],
};
"#;
        
        let result = verify_fixture(original, compiled);
        assert!(result.passed, "Semantically equivalent code should pass: {:?}", result);
    }

    #[test]
    fn test_verify_different_results_fails() {
        let original = r#"
function getValue() {
    return 42;
}

const FIXTURE_ENTRYPOINT = {
    fn: getValue,
    params: [],
};
"#;
        
        let compiled = r#"
function getValue() {
    return 100;  // Wrong!
}

const FIXTURE_ENTRYPOINT = {
    fn: getValue,
    params: [],
};
"#;
        
        let result = verify_fixture(original, compiled);
        assert!(!result.passed, "Different results should fail");
    }
}
