use clap::Parser;
use miette::{IntoDiagnostic, Result};
use oxc_span::SourceType;
use std::path::PathBuf;
use react_compiler_rust::debug_hir;

/// React Compiler (Rust Edition)
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Input file to compile
    #[arg(short, long)]
    input: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let source_path = args.input;
    
    let source_text = std::fs::read_to_string(&source_path)
        .into_diagnostic()?;

    println!("Compiling: {}", source_path.display());

    let source_type = SourceType::from_path(&source_path).unwrap_or_default();
    
    let output = debug_hir(&source_text, source_type)?;
    
    println!("{}", output);

    Ok(())
}