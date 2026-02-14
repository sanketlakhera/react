//! Error types for the React Compiler.

use miette::Diagnostic;
use thiserror::Error;

/// The main error type for the compiler.
#[derive(Error, Debug, Diagnostic)]
pub enum CompilerError {
    /// Error during parsing phase
    #[error("Parse error: {message}")]
    #[diagnostic(code(react_compiler::parse_error))]
    ParseError { message: String },

    /// Error during lowering (AST -> HIR)
    #[error("Lowering error: {message}")]
    #[diagnostic(code(react_compiler::lowering_error))]
    LoweringError { message: String },

    /// Unsupported JavaScript syntax
    #[error("Unsupported syntax: {syntax}")]
    #[diagnostic(code(react_compiler::unsupported_syntax), help("This syntax is not yet supported by the compiler"))]
    UnsupportedSyntax { syntax: String },

    /// IO errors
    #[error("IO error: {0}")]
    #[diagnostic(code(react_compiler::io_error))]
    IoError(#[from] std::io::Error),
}

/// Type alias for compiler results
pub type CompilerResult<T> = Result<T, CompilerError>;
