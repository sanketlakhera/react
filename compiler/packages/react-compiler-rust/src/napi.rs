//! NAPI-RS Bindings for React Compiler Rust
//!
//! Exposes the Rust compiler as a native Node.js module.

#[cfg(feature = "napi")]
use napi_derive::napi;

#[cfg(feature = "napi")]
use oxc_span::SourceType;

/// Result from compiling JavaScript/TypeScript code
#[cfg_attr(feature = "napi", napi(object))]
pub struct CompileResult {
    /// The compiled output code
    pub code: String,
    /// Whether compilation was successful
    pub success: bool,
    /// Error message if compilation failed
    pub error: Option<String>,
}

/// Compile JavaScript/TypeScript source code to optimized JavaScript
/// with automatic memoization (useMemoCache patterns).
///
/// @param source - The source code to compile
/// @returns CompileResult with compiled code or error
#[cfg(feature = "napi")]
#[napi]
pub fn compile(source: String) -> CompileResult {
    compile_with_options(source, None)
}

/// Compile with options for file type
#[cfg(feature = "napi")]
#[napi]
pub fn compile_with_options(source: String, file_type: Option<String>) -> CompileResult {
    let source_type = match file_type.as_deref() {
        Some("ts") => SourceType::ts(),
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::mjs(),
    };

    match crate::compile(&source, source_type) {
        Ok(code) => CompileResult {
            code,
            success: true,
            error: None,
        },
        Err(e) => CompileResult {
            code: String::new(),
            success: false,
            error: Some(format!("{}", e)),
        },
    }
}

/// Get version information
#[cfg(feature = "napi")]
#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
