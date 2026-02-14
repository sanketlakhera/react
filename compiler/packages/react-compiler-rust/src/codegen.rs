//! JavaScript Code Generation
//!
//! This module generates JavaScript code from the ReactiveFunction tree,
//! emitting useMemoCache patterns for reactive scopes.

use crate::hir::Identifier;
use crate::hir::reactive_function::{
    ConstantValue, ReactiveArgument, ReactiveArrayElement, ReactiveFunction, ReactiveInstruction,
    ReactiveObjectKey, ReactiveObjectProperty, ReactiveStatement, ReactiveValue,
};
use crate::hir::reactive_scopes::ReactiveScopeResult;
use crate::hir::scope::ScopeId;
use std::collections::HashSet;
use std::fmt::Write;

/// Generate JavaScript code from a ReactiveFunction
pub fn generate_code(func: &ReactiveFunction, scopes: &ReactiveScopeResult) -> String {
    let mut codegen = CodeGenerator::new(scopes);
    codegen.generate_function(func)
}

struct CodeGenerator<'a> {
    output: String,
    indent: usize,
    scopes: &'a ReactiveScopeResult,
    cache_size: usize,
    declared: HashSet<String>,
    declared_base_names: HashSet<String>,
    params: HashSet<String>,
}

impl<'a> CodeGenerator<'a> {
    fn new(scopes: &'a ReactiveScopeResult) -> Self {
        // Calculate total cache size needed
        let cache_size = scopes.scopes.iter()
            .map(|s| s.dependencies.len() + s.declarations.len())
            .sum::<usize>()
            .max(1);
            
        Self {
            output: String::new(),
            indent: 0,
            scopes,
            cache_size,
            declared: HashSet::new(),
            declared_base_names: HashSet::new(),
            params: HashSet::new(),
        }
    }

    fn generate_function(&mut self, func: &ReactiveFunction) -> String {
        let name = func.name.as_deref().unwrap_or("anonymous");
        
        // Function header
        self.params = func.params.iter().map(|p| p.name.clone()).collect();
        let params_str: Vec<_> = func.params.iter().map(|p| self.identifier_name(p)).collect();
        writeln!(self.output, "function {}({}) {{", name, params_str.join(", ")).unwrap();
        self.indent += 1;
        
        // Add cache initialization if we have scopes
        if !self.scopes.scopes.is_empty() {
            self.write_indent();
            writeln!(self.output, "const $ = _c({});", self.cache_size).unwrap();
        }

        // Hoist declarations
        // Hoist declarations
        for stmt in &func.body {
            Self::collect_declarations(stmt, &mut self.declared, &mut self.declared_base_names);
        }
        
        // Filter out params from declared to avoid re-declaration
        for p in &self.params {
            self.declared.remove(p);
        }
        
        if !self.declared.is_empty() {
            let mut sorted_vars: Vec<_> = self.declared.iter().cloned().collect();
            sorted_vars.sort();
            self.write_indent();
            writeln!(self.output, "let {};", sorted_vars.join(", ")).unwrap();
        }
        
        // Generate body
        for stmt in &func.body {
            self.generate_statement(stmt);
        }
        
        self.indent -= 1;
        writeln!(self.output, "}}").unwrap();
        
        self.output.clone()
    }

    fn generate_statement(&mut self, stmt: &ReactiveStatement) {
        match stmt {
            ReactiveStatement::Instruction(instr) => {
                self.generate_instruction(instr);
            }
            ReactiveStatement::Scope { id, dependencies, declarations, body } => {
                self.generate_scope(*id, dependencies, declarations, body);
            }
            ReactiveStatement::If { test, consequent, alternate } => {
                self.write_indent();
                writeln!(self.output, "if ({}) {{", self.identifier_name(test)).unwrap();
                self.indent += 1;
                for s in consequent {
                    self.generate_statement(s);
                }
                self.indent -= 1;
                
                if !alternate.is_empty() {
                    self.write_indent();
                    writeln!(self.output, "}} else {{").unwrap();
                    self.indent += 1;
                    for s in alternate {
                        self.generate_statement(s);
                    }
                    self.indent -= 1;
                }
                
                self.write_indent();
                writeln!(self.output, "}}").unwrap();
            }
            ReactiveStatement::While { test, body } => {
                self.write_indent();
                writeln!(self.output, "while ({}) {{", self.identifier_name(test)).unwrap();
                self.indent += 1;
                for s in body {
                    self.generate_statement(s);
                }
                self.indent -= 1;
                self.write_indent();
                writeln!(self.output, "}}").unwrap();
            }
            ReactiveStatement::Break => {
                self.write_indent();
                writeln!(self.output, "break;").unwrap();
            }
            ReactiveStatement::Continue => {
                self.write_indent();
                writeln!(self.output, "continue;").unwrap();
            }
            ReactiveStatement::Return(place) => {
                self.write_indent();
                if let Some(id) = place {
                    writeln!(self.output, "return {};", self.identifier_name(id)).unwrap();
                } else {
                    writeln!(self.output, "return;").unwrap();
                }
            }
            ReactiveStatement::Switch { test, cases } => {
                self.write_indent();
                writeln!(self.output, "switch ({}) {{", self.identifier_name(test)).unwrap();
                self.indent += 1;
                
                for case in cases {
                    self.write_indent();
                    if let Some(label) = &case.label {
                        writeln!(self.output, "case {}: {{", self.identifier_name(label)).unwrap();
                    } else {
                        writeln!(self.output, "default: {{").unwrap();
                    }
                    
                    self.indent += 1;
                    for s in &case.body {
                        self.generate_statement(s);
                    }
                    self.indent -= 1;
                    
                    self.write_indent();
                    writeln!(self.output, "}}").unwrap();
                }
                
                self.indent -= 1;
                self.write_indent();
                writeln!(self.output, "}}").unwrap();
            }
        }
    }

    fn generate_instruction(&mut self, instr: &ReactiveInstruction) {
        let lvalue = self.identifier_name(&instr.lvalue);
        let rvalue = self.generate_value(&instr.value);
        
        // Skip trivial assignments (LoadLocal where source == dest name)
        if let ReactiveValue::LoadLocal(src) = &instr.value {
            if self.identifier_name(src) == lvalue {
                return;
            }
        }
        
        self.write_indent();
        
        // Use let for declarations, assignment for updates/temporaries
        let is_temp = instr.lvalue.name.starts_with('t') && instr.lvalue.name[1..].chars().all(|c| c.is_ascii_digit());
        let is_reserved = matches!(instr.lvalue.name.as_str(), "true" | "false" | "null" | "undefined");
        
        if is_temp || is_reserved {
            writeln!(self.output, "const {} = {};", lvalue, rvalue).unwrap();
        } else if self.declared.contains(&lvalue) {
            writeln!(self.output, "{} = {};", lvalue, rvalue).unwrap();
        } else {
            self.declared.insert(lvalue.clone());
            writeln!(self.output, "let {} = {};", lvalue, rvalue).unwrap();
        }
    }

    fn generate_value(&self, value: &ReactiveValue) -> String {
        match value {
            ReactiveValue::Constant(c) => match c {
                ConstantValue::Number(n) => {
                    if n.fract() == 0.0 {
                        format!("{}", *n as i64)
                    } else {
                        format!("{}", n)
                    }
                }
                ConstantValue::String(s) => {
                    let escaped = s
                        .replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t")
                        .replace('\0', "\\0");
                    format!("\"{}\"", escaped)
                },
                ConstantValue::Boolean(b) => format!("{}", b),
                ConstantValue::Null => "null".to_string(),
                ConstantValue::Undefined => "undefined".to_string(),
            },
            ReactiveValue::BinaryOp { op, left, right } => {
                format!("{} {} {}", self.identifier_name(left), op, self.identifier_name(right))
            }
            ReactiveValue::UnaryOp { op, operand } => {
                if op == "__isNullish__" {
                    // Generate: (x == null) which checks for both null and undefined
                    format!("({} == null)", self.identifier_name(operand))
                } else {
                    format!("{}{}", op, self.identifier_name(operand))
                }
            }
            ReactiveValue::Call { callee, args } => {
                let args_str: Vec<_> = args.iter().map(|a| {
                    match a {
                        ReactiveArgument::Regular(id) => self.identifier_name(id),
                        ReactiveArgument::Spread(id) => format!("...{}", self.identifier_name(id)),
                    }
                }).collect();
                format!("{}({})", self.identifier_name(callee), args_str.join(", "))
            }
            ReactiveValue::Object { properties } => {
                let props: Vec<_> = properties
                    .iter()
                    .map(|prop| {
                        match prop {
                            ReactiveObjectProperty::KeyValue { key, value } => {
                                let key_str = match key {
                                    ReactiveObjectKey::Identifier(s) => s.clone(),
                                    ReactiveObjectKey::Computed(id) => format!("[{}]", self.identifier_name(id)),
                                };
                                format!("{}: {}", key_str, self.identifier_name(value))
                            }
                            ReactiveObjectProperty::Spread(id) => format!("...{}", self.identifier_name(id)),
                        }
                    })
                    .collect();
                format!("{{ {} }}", props.join(", "))
            }
            ReactiveValue::Array { elements } => {
                let elems: Vec<_> = elements.iter().map(|e| {
                    match e {
                        ReactiveArrayElement::Regular(id) => self.identifier_name(id),
                        ReactiveArrayElement::Spread(id) => format!("...{}", self.identifier_name(id)),
                        ReactiveArrayElement::Hole => String::new(),
                    }
                }).collect();
                format!("[{}]", elems.join(", "))
            }
            ReactiveValue::PropertyLoad { object, property } => {
                format!("{}.{}", self.identifier_name(object), property)
            }
            ReactiveValue::PropertyStore { object, property, value } => {
                format!("{}.{} = {}", self.identifier_name(object), property, self.identifier_name(value))
            }
            ReactiveValue::ComputedLoad { object, property } => {
                format!("{}[{}]", self.identifier_name(object), self.identifier_name(property))
            }
            ReactiveValue::ComputedStore { object, property, value } => {
                format!("{}[{}] = {}", self.identifier_name(object), self.identifier_name(property), self.identifier_name(value))
            }
            ReactiveValue::LoadLocal(id) => {
                self.identifier_name(id)
            }
            ReactiveValue::Phi { operands } => {
                // Phi nodes shouldn't appear in codegen, but handle gracefully
                if let Some(first) = operands.first() {
                    self.identifier_name(first)
                } else {
                    "undefined".to_string()
                }
            }
        }
    }

    fn generate_scope(
        &mut self,
        _id: ScopeId,
        dependencies: &[Identifier],
        declarations: &[Identifier],
        body: &[ReactiveStatement],
    ) {
        // Generate useMemoCache pattern:
        // if ($[0] !== dep1 || $[1] !== dep2) {
        //   // body
        //   $[0] = dep1; $[1] = dep2; $[2] = result;
        // }
        // const result = $[2];
        
        if dependencies.is_empty() && body.is_empty() {
            return;
        }

        let dep_count = dependencies.len();
        
        // Generate condition
        self.write_indent();
        if dependencies.is_empty() {
            writeln!(self.output, "if ($[0] === Symbol.for(\"react.memo_cache_sentinel\")) {{").unwrap();
        } else {
            let conditions: Vec<_> = dependencies
                .iter()
                .enumerate()
                .map(|(i, d)| format!("$[{}] !== {}", i, self.identifier_name(d)))
                .collect();
            writeln!(self.output, "if ({}) {{", conditions.join(" || ")).unwrap();
        }
        
        self.indent += 1;
        
        // Generate body
        for stmt in body {
            self.generate_statement(stmt);
        }
        
        // Store dependencies
        for (i, dep) in dependencies.iter().enumerate() {
            self.write_indent();
            writeln!(self.output, "$[{}] = {};", i, self.identifier_name(dep)).unwrap();
        }
        
        // Store declarations
        for (i, decl) in declarations.iter().enumerate() {
            self.write_indent();
            writeln!(self.output, "$[{}] = {};", dep_count + i, self.identifier_name(decl)).unwrap();
        }
        
        self.indent -= 1;
        self.write_indent();
        writeln!(self.output, "}}").unwrap();
        
        // Read cached declarations
        for (i, decl) in declarations.iter().enumerate() {
            self.write_indent();
            writeln!(self.output, "const {} = $[{}];", self.identifier_name(decl), dep_count + i).unwrap();
        }
    }

    fn get_canonical_name(id: &Identifier) -> String {
        let is_temp = id.name.starts_with('t') && id.name.len() > 1 && id.name[1..].chars().all(|c| c.is_ascii_digit());
        let is_reserved = matches!(id.name.as_str(), "true" | "false" | "null" | "undefined");
        if is_temp || is_reserved {
            id.name.clone()
        } else {
            format!("{}_{}", id.name, id.id)
        }
    }

    fn identifier_name(&self, id: &Identifier) -> String {
        let canonical = Self::get_canonical_name(id);
        
        if self.params.contains(&id.name) {
            return id.name.clone();
        }

        // Check for globals (id=0 and not declared locally)
        if id.id == 0 {
             // If base name IS declared locally (e.g. j_1 exists), then j_0 is Uninitialized Local -> undefined.
             if self.declared_base_names.contains(&id.name) {
                 return "undefined".to_string();
             }
             // If base name is NOT declared locally, it must be Global.
             return id.name.clone();
        }
        
        canonical
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent {
            write!(self.output, "  ").unwrap();
        }
    }

    fn collect_declarations(stmt: &ReactiveStatement, vars: &mut HashSet<String>, base_names: &mut HashSet<String>) {
        match stmt {
            ReactiveStatement::Instruction(instr) => {
                let name = Self::get_canonical_name(&instr.lvalue);
                // Only hoist user variables (not temps starting with 't' followed by digit)
                let is_temp = instr.lvalue.name.starts_with('t') && instr.lvalue.name.len() > 1 && instr.lvalue.name[1..].chars().all(|c| c.is_ascii_digit());
                let is_reserved = matches!(instr.lvalue.name.as_str(), "true" | "false" | "null" | "undefined");
                
                if !is_temp && !is_reserved && !vars.contains(&name) {
                    vars.insert(name);
                    base_names.insert(instr.lvalue.name.clone());
                }
            }
            ReactiveStatement::If { consequent, alternate, .. } => {
                for s in consequent {
                    Self::collect_declarations(s, vars, base_names);
                }
                for s in alternate {
                    Self::collect_declarations(s, vars, base_names);
                }
            }
            ReactiveStatement::While { body, .. } => {
                for s in body {
                    Self::collect_declarations(s, vars, base_names);
                }
            }
            ReactiveStatement::Scope { body, .. } => {
                for s in body {
                    Self::collect_declarations(s, vars, base_names);
                }
            }
            ReactiveStatement::Switch { cases, .. } => {
                for case in cases {
                    for s in &case.body {
                        Self::collect_declarations(s, vars, base_names);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_generation() {
        let generator = CodeGenerator {
            output: String::new(),
            indent: 0,
            scopes: &ReactiveScopeResult {
                scopes: vec![],
                instruction_scopes: std::collections::HashMap::new(),
            },
            cache_size: 0,
            declared: HashSet::new(),
            declared_base_names: HashSet::new(),
            params: HashSet::new(),
        };
        
        assert_eq!(generator.generate_value(&ReactiveValue::Constant(ConstantValue::Number(42.0))), "42");
        assert_eq!(generator.generate_value(&ReactiveValue::Constant(ConstantValue::Boolean(true))), "true");
        assert_eq!(generator.generate_value(&ReactiveValue::Constant(ConstantValue::Null)), "null");
    }
}
