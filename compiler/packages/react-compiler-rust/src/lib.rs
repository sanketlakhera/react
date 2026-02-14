pub mod codegen;
pub mod error;
pub mod hir;
pub mod napi;
pub mod sprout;

pub use error::{CompilerError, CompilerResult};

use codegen::generate_code;
use hir::inference::infer_liveness;
use hir::lowering::LoweringContext;
use hir::reactive_function::build_reactive_function;
use hir::reactive_scopes::construct_reactive_scopes;
use hir::ssa::enter_ssa;
use miette::Result;
use oxc_allocator::Allocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;

/// Compile JavaScript/TypeScript source code to optimized JavaScript with memoization.
pub fn compile(source_text: &str, source_type: SourceType) -> Result<String> {
    let allocator = Allocator::default();

    let ret = OxcParser::new(&allocator, source_text, source_type)
        .parse();

    if !ret.errors.is_empty() {
        use std::fmt::Write;
        let mut err_msg = String::new();
        writeln!(&mut err_msg, "Parse Errors:").unwrap();
        for error in ret.errors {
            writeln!(&mut err_msg, "{:?}", error).unwrap();
        }
        return Ok(err_msg);
    }

    let mut output = String::new();

    for stmt in &ret.program.body {
        if let oxc_ast::ast::Statement::FunctionDeclaration(func) = stmt {
            // Phase 1-2: Lower AST to HIR
            let ctx = LoweringContext::default();
            let hir = ctx.build(func);

            // Phase 3: SSA transformation
            let ssa_hir = enter_ssa(hir);

            // Phase 4: Liveness analysis and scope construction
            let liveness = infer_liveness(&ssa_hir);
            let scope_result = construct_reactive_scopes(&ssa_hir, &liveness);

            // Phase 5: Build reactive function tree and generate code
            let reactive_func = build_reactive_function(&ssa_hir, &scope_result);
            let code = generate_code(&reactive_func, &scope_result);

            output.push_str(&code);
            output.push('\n');
        }
    }

    Ok(output)
}

/// Debug function that shows intermediate representations.
pub fn debug_hir(source_text: &str, source_type: SourceType) -> Result<String> {
    let allocator = Allocator::default();

    let ret = OxcParser::new(&allocator, source_text, source_type)
        .parse();

    if !ret.errors.is_empty() {
        use std::fmt::Write;
        let mut err_msg = String::new();
        writeln!(&mut err_msg, "Parse Errors:").unwrap();
        for error in ret.errors {
            writeln!(&mut err_msg, "{:?}", error).unwrap();
        }
        return Ok(err_msg);
    }

    let mut output = String::new();

    for stmt in &ret.program.body {
        if let oxc_ast::ast::Statement::FunctionDeclaration(func) = stmt {
             let ctx = LoweringContext::default();
             let hir = ctx.build(func);
             let ssa_hir = enter_ssa(hir);

             let liveness = infer_liveness(&ssa_hir);
             let scope_result = construct_reactive_scopes(&ssa_hir, &liveness);

             use std::fmt::Write;
             writeln!(&mut output, "=== HIR (SSA) ===").unwrap();
             write!(&mut output, "{:#?}\n", ssa_hir).unwrap();

             if !scope_result.scopes.is_empty() {
                 writeln!(&mut output, "\n=== Reactive Scopes ===").unwrap();
                 for scope in &scope_result.scopes {
                     writeln!(&mut output, "Scope {:?}: range {:?}", scope.id, scope.range).unwrap();
                     if !scope.dependencies.is_empty() {
                         write!(&mut output, "  Dependencies: ").unwrap();
                         for dep in &scope.dependencies {
                             write!(&mut output, "{} ", dep.place.identifier.name).unwrap();
                         }
                         writeln!(&mut output).unwrap();
                     }
                     if !scope.declarations.is_empty() {
                         write!(&mut output, "  Declarations: ").unwrap();
                         for decl in &scope.declarations {
                             write!(&mut output, "{} ", decl.place.identifier.name).unwrap();
                         }
                         writeln!(&mut output).unwrap();
                     }
                 }
             }

             // Also show generated code
             let reactive_func = build_reactive_function(&ssa_hir, &scope_result);
             let code = generate_code(&reactive_func, &scope_result);
             writeln!(&mut output, "\n=== Generated Code ===").unwrap();
             write!(&mut output, "{}", code).unwrap();
        }
    }

    Ok(output)
}
