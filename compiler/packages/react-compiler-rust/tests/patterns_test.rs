//! Pattern-Based React Testing
//!
//! Tests focused fixtures covering common React patterns.

use react_compiler_rust::compile;
use oxc_span::SourceType;
use std::fs;
use std::path::PathBuf;

fn patterns_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/patterns")
}

fn test_pattern(filename: &str) -> Result<String, String> {
    let path = patterns_dir().join(filename);
    let source = fs::read_to_string(&path)
        .map_err(|e| format!("Read error: {}", e))?;
    
    let result = std::panic::catch_unwind(|| {
        compile(&source, SourceType::jsx())
    });
    
    match result {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(format!("{}", e)),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic".to_string()
            };
            Err(msg)
        }
    }
}

// ============ Hooks Patterns ============

#[test]
fn pattern_hooks_use_state() {
    let result = test_pattern("hooks_useState.js");
    assert!(result.is_ok(), "useState pattern failed: {:?}", result.err());
    println!("✓ hooks_useState.js");
}

#[test]
fn pattern_hooks_use_effect() {
    let result = test_pattern("hooks_useEffect.js");
    assert!(result.is_ok(), "useEffect pattern failed: {:?}", result.err());
    println!("✓ hooks_useEffect.js");
}

#[test]
fn pattern_hooks_use_memo() {
    let result = test_pattern("hooks_useMemo.js");
    assert!(result.is_ok(), "useMemo pattern failed: {:?}", result.err());
    println!("✓ hooks_useMemo.js");
}

#[test]
fn pattern_hooks_use_callback() {
    let result = test_pattern("hooks_useCallback.js");
    assert!(result.is_ok(), "useCallback pattern failed: {:?}", result.err());
    println!("✓ hooks_useCallback.js");
}

// ============ Props Patterns ============

#[test]
fn pattern_props_destructuring() {
    let result = test_pattern("props_destructuring.js");
    assert!(result.is_ok(), "Props destructuring failed: {:?}", result.err());
    println!("✓ props_destructuring.js");
}

#[test]
fn pattern_props_children() {
    let result = test_pattern("props_children.js");
    assert!(result.is_ok(), "Props children failed: {:?}", result.err());
    println!("✓ props_children.js");
}

// ============ Control Flow Patterns ============

#[test]
fn pattern_control_ternary() {
    let result = test_pattern("control_ternary.js");
    assert!(result.is_ok(), "Ternary pattern failed: {:?}", result.err());
    println!("✓ control_ternary.js");
}

#[test]
fn pattern_control_logical() {
    let result = test_pattern("control_logical.js");
    assert!(result.is_ok(), "Logical operators failed: {:?}", result.err());
    println!("✓ control_logical.js");
}

#[test]
fn pattern_control_early_return() {
    let result = test_pattern("control_earlyReturn.js");
    assert!(result.is_ok(), "Early return failed: {:?}", result.err());
    println!("✓ control_earlyReturn.js");
}

// ============ List Patterns ============

#[test]
fn pattern_list_map() {
    let result = test_pattern("list_map.js");
    assert!(result.is_ok(), "List map failed: {:?}", result.err());
    println!("✓ list_map.js");
}

#[test]
fn pattern_list_filter() {
    let result = test_pattern("list_filter.js");
    assert!(result.is_ok(), "List filter failed: {:?}", result.err());
    println!("✓ list_filter.js");
}

// ============ Event Patterns ============

#[test]
fn pattern_events_on_click() {
    let result = test_pattern("events_onClick.js");
    assert!(result.is_ok(), "onClick pattern failed: {:?}", result.err());
    println!("✓ events_onClick.js");
}

#[test]
fn pattern_events_on_change() {
    let result = test_pattern("events_onChange.js");
    assert!(result.is_ok(), "onChange pattern failed: {:?}", result.err());
    println!("✓ events_onChange.js");
}
