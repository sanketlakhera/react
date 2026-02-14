//! Reactive Function - Tree-structured IR for code generation
//!
//! This module converts the graph-based HIR (CFG) back into a tree structure
//! suitable for JavaScript code generation.

use crate::hir::scope::ScopeId;
use crate::hir::{
    BlockId, HIRFunction, Identifier, Instruction, InstructionValue, Terminal,
};
use crate::hir::reactive_scopes::ReactiveScopeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A tree-structured representation of a function for code generation.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReactiveFunction {
    pub name: Option<String>,
    pub params: Vec<Identifier>,
    pub body: Vec<ReactiveStatement>,
}

/// A statement in the reactive function tree.
#[derive(Debug, Serialize, Deserialize)]
pub enum ReactiveStatement {
    /// A single instruction (expression statement or declaration)
    Instruction(ReactiveInstruction),
    
    /// A reactive scope (memoization boundary)
    Scope {
        id: ScopeId,
        dependencies: Vec<Identifier>,
        declarations: Vec<Identifier>,
        body: Vec<ReactiveStatement>,
    },
    
    /// A conditional (if/else)
    If {
        test: Identifier,
        consequent: Vec<ReactiveStatement>,
        alternate: Vec<ReactiveStatement>,
    },
    
    /// A loop (while)
    While {
        test: Identifier,
        body: Vec<ReactiveStatement>,
    },

    /// A break statement
    Break,

    /// A continue statement
    Continue,

    /// A return statement
    Return(Option<Identifier>),
    
    /// A switch statement
    Switch {
        test: Identifier,
        cases: Vec<ReactiveSwitchCase>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactiveSwitchCase {
    pub label: Option<Identifier>, // None for default
    pub body: Vec<ReactiveStatement>,
}

/// A simplified instruction for codegen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactiveInstruction {
    pub lvalue: Identifier,
    pub value: ReactiveValue,
    pub scope: Option<ScopeId>,
}

/// Instruction values simplified for codegen
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReactiveValue {
    Constant(ConstantValue),
    BinaryOp { op: String, left: Identifier, right: Identifier },
    UnaryOp { op: String, operand: Identifier },
    Call { callee: Identifier, args: Vec<ReactiveArgument> },
    Object { properties: Vec<ReactiveObjectProperty> },
    Array { elements: Vec<ReactiveArrayElement> },
    PropertyLoad { object: Identifier, property: String },
    PropertyStore { object: Identifier, property: String, value: Identifier },
    ComputedLoad { object: Identifier, property: Identifier },
    ComputedStore { object: Identifier, property: Identifier, value: Identifier },
    LoadLocal(Identifier),
    Phi { operands: Vec<Identifier> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReactiveArgument {
    Regular(Identifier),
    Spread(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReactiveObjectProperty {
    KeyValue { key: ReactiveObjectKey, value: Identifier },
    Spread(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReactiveObjectKey {
    Identifier(String),
    Computed(Identifier),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReactiveArrayElement {
    Regular(Identifier),
    Spread(Identifier),
    Hole,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstantValue {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
}

/// Convert HIR (CFG) to ReactiveFunction (tree)
pub fn build_reactive_function(
    hir: &HIRFunction,
    scope_result: &ReactiveScopeResult,
) -> ReactiveFunction {
    let mut builder = TreeBuilder::new(hir, scope_result);
    builder.build()
}

struct TreeBuilder<'a> {
    hir: &'a HIRFunction,
    visited_blocks: HashSet<BlockId>,
    current_loops: HashSet<BlockId>,
    loop_stack: Vec<TreeLoopInfo>,
}

#[derive(Clone, Copy)]
struct TreeLoopInfo {
    header: BlockId,
    break_target: BlockId,
}

impl<'a> TreeBuilder<'a> {
    fn new(hir: &'a HIRFunction, scope_result: &'a ReactiveScopeResult) -> Self {
        Self {
            hir,
            visited_blocks: HashSet::new(),
            current_loops: HashSet::new(),
            loop_stack: Vec::new(),
        }
    }

    pub fn build(&mut self) -> ReactiveFunction {
        let body = self.build_block(self.hir.entry_block, None);
        ReactiveFunction {
            name: self.hir.name.clone(),
            params: self.hir.params.clone(),
            body,
        }
    }

    fn build_block(&mut self, block_id: BlockId, prev_id: Option<BlockId>) -> Vec<ReactiveStatement> {
        let mut statements = Vec::new();

        // 1. Handle Phis from predecessors
        if let Some(block) = self.hir.blocks.get(&block_id) {
            for instr in &block.instructions {
                if let InstructionValue::Phi { operands } = &instr.value {
                    if let Some(p_id) = prev_id {
                        for (pred_id, place) in operands {
                            if *pred_id == p_id {
                                statements.push(ReactiveStatement::Instruction(ReactiveInstruction {
                                    lvalue: instr.lvalue.identifier.clone(),
                                    value: ReactiveValue::LoadLocal(place.identifier.clone()),
                                    scope: None,
                                }));
                            }
                        }
                    }
                }
            }
        }

        // 2. Handle recursion and loops
        if self.visited_blocks.contains(&block_id) {
            return statements;
        }
        self.visited_blocks.insert(block_id);

        if let Some(block) = self.hir.blocks.get(&block_id) {
            // Check if this is a loop header
            let is_loop = self.is_loop_header(block_id);
            if is_loop {
                self.current_loops.insert(block_id);
                
                let mut loop_body = Vec::new();
                
                // Add header instructions (except Phis)
                for instr in &block.instructions {
                    if !matches!(instr.value, InstructionValue::Phi { .. }) {
                        loop_body.push(ReactiveStatement::Instruction(self.convert_instruction(instr)));
                    }
                }
                
                // Handle terminal of the loop header
                match &block.terminal {
                    Terminal::If { test, consequent, alternate } => {
                        // For a standard loop, consequent is the body, alternate is the exit.
                        // We use a "while(true)" style for now and internal break.
                        // This handles the re-evaluation of the test naturally.
                        
                        let test_id = test.identifier.clone();
                        
                        // if (!test) break;
                        loop_body.push(ReactiveStatement::If {
                            test: test_id,
                            consequent: vec![],
                            alternate: vec![ReactiveStatement::Break],
                        });
                        
                        // Body path
                        self.loop_stack.push(TreeLoopInfo {
                            header: block_id,
                            break_target: *alternate,
                        });
                        // println!("Loop start {:?}, break_target {:?}", block_id, *alternate);
                        loop_body.extend(self.build_block(*consequent, Some(block_id)));
                        self.loop_stack.pop();
                        // println!("Loop end {:?}", block_id);
                        
                        // Now we have the While statement
                        // We'll use "true" as a hacky literal identifier
                        let true_id = Identifier { name: "true".to_string(), id: 0 };
                        statements.push(ReactiveStatement::While {
                            test: true_id,
                            body: loop_body,
                        });
                        
                        self.current_loops.remove(&block_id);
                        
                        // Exit path (after the loop)
                        statements.extend(self.build_block(*alternate, Some(block_id)));
                    }
                    _ => {
                        // Unstructured loop? Fallback to normal
                        self.current_loops.remove(&block_id);
                    }
                }
                
                self.visited_blocks.remove(&block_id);
                return statements;
            }

            // Normal non-loop block
            for instr in &block.instructions {
                if matches!(instr.value, InstructionValue::Phi { .. }) {
                    continue;
                }
                let reactive_instr = self.convert_instruction(instr);
                statements.push(ReactiveStatement::Instruction(reactive_instr));
            }

            // Handle terminal
            match &block.terminal {
                Terminal::Return(place) => {
                    statements.push(ReactiveStatement::Return(
                        place.as_ref().map(|p| p.identifier.clone())
                    ));
                }
                Terminal::Goto(target) => {
                    // Check for break/continue
                    if let Some(loop_info) = self.loop_stack.last() {
                        if *target == loop_info.break_target {
                            statements.extend(self.emit_phi_assignments(*target, block_id));
                            statements.push(ReactiveStatement::Break);
                            self.visited_blocks.remove(&block_id);
                            return statements;
                        }
                        if *target == loop_info.header {
                            statements.extend(self.emit_phi_assignments(*target, block_id));
                            statements.push(ReactiveStatement::Continue);
                            self.visited_blocks.remove(&block_id);
                            return statements;
                        }
                    }

                    let next_stmts = self.build_block(*target, Some(block_id));
                    statements.extend(next_stmts);
                }
                Terminal::If { test, consequent, alternate } => {
                    let test_id = test.identifier.clone();
                    let then_stmts = self.build_block(*consequent, Some(block_id));
                    let else_stmts = self.build_block(*alternate, Some(block_id));
                    
                    statements.push(ReactiveStatement::If {
                        test: test_id,
                        consequent: then_stmts,
                        alternate: else_stmts,
                    });
                }
                Terminal::Switch { test, cases, default, merge_target } => {
                    let test_id = test.identifier.clone();
                    
                    if let Some(target) = merge_target {
                        self.loop_stack.push(TreeLoopInfo { header: block_id, break_target: *target });
                    }
                    
                    let mut reactive_cases = Vec::with_capacity(cases.len() + 1);
                    
                    // Specific cases
                    for (val, target) in cases {
                         let case_stmts = self.build_block(*target, Some(block_id));
                         reactive_cases.push(ReactiveSwitchCase {
                             label: Some(val.identifier.clone()),
                             body: case_stmts,
                         });
                    }
                    
                    // Default case
                    let default_stmts = self.build_block(*default, Some(block_id));
                    reactive_cases.push(ReactiveSwitchCase {
                        label: None,
                        body: default_stmts,
                    });
                    
                    if merge_target.is_some() {
                        self.loop_stack.pop();
                    }
                    
                    statements.push(ReactiveStatement::Switch {
                        test: test_id,
                        cases: reactive_cases,
                    });
                }
            }
        }

        self.visited_blocks.remove(&block_id);
        statements
    }

    fn is_loop_header(&self, block_id: BlockId) -> bool {
        self.hir.loop_headers.contains(&block_id)
    }
    
    fn emit_phi_assignments(&self, target_id: BlockId, current_id: BlockId) -> Vec<ReactiveStatement> {
        let mut statements = Vec::new();
        if let Some(block) = self.hir.blocks.get(&target_id) {
            for instr in &block.instructions {
                if let InstructionValue::Phi { operands } = &instr.value {
                    for (pred_id, place) in operands {
                        if *pred_id == current_id {
                            statements.push(ReactiveStatement::Instruction(ReactiveInstruction {
                                lvalue: instr.lvalue.identifier.clone(),
                                value: ReactiveValue::LoadLocal(place.identifier.clone()),
                                scope: None,
                            }));
                        }
                    }
                }
            }
        }
        statements
    }


    fn convert_instruction(&self, instr: &Instruction) -> ReactiveInstruction {
        let value = match &instr.value {
            InstructionValue::Constant(c) => {
                use crate::hir::Constant;
                let cv = match c {
                    Constant::Int(n) => ConstantValue::Number(*n as f64),
                    Constant::Float(n) => ConstantValue::Number(*n),
                    Constant::String(s) => ConstantValue::String(s.clone()),
                    Constant::Boolean(b) => ConstantValue::Boolean(*b),
                    Constant::Null => ConstantValue::Null,
                    Constant::Undefined => ConstantValue::Undefined,
                };
                ReactiveValue::Constant(cv)
            }
            InstructionValue::BinaryOp { op, left, right } => {
                use crate::hir::BinaryOperator;
                let op_str = match op {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Sub => "-",
                    BinaryOperator::Mul => "*",
                    BinaryOperator::Div => "/",
                    BinaryOperator::Mod => "%",
                    BinaryOperator::LessThan => "<",
                    BinaryOperator::LessThanEqual => "<=",
                    BinaryOperator::GreaterThan => ">",
                    BinaryOperator::GreaterThanEqual => ">=",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::StrictEqual => "===",
                    BinaryOperator::StrictNotEqual => "!==",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                    BinaryOperator::BitwiseAnd => "&",
                    BinaryOperator::BitwiseOr => "|",
                    BinaryOperator::BitwiseXor => "^",
                    BinaryOperator::LeftShift => "<<",
                    BinaryOperator::RightShift => ">>",
                    BinaryOperator::UnsignedRightShift => ">>>",
                    BinaryOperator::InstanceOf => "instanceof",
                    BinaryOperator::In => "in",
                };
                ReactiveValue::BinaryOp {
                    op: op_str.to_string(),
                    left: left.identifier.clone(),
                    right: right.identifier.clone(),
                }
            }
            InstructionValue::UnaryOp { op, operand } => {
                use crate::hir::UnaryOperator;
                let op_str = match op {
                    UnaryOperator::Not => "!",
                    UnaryOperator::Negate => "-",
                    UnaryOperator::Plus => "+",
                    UnaryOperator::BitwiseNot => "~",
                    UnaryOperator::TypeOf => "typeof ",
                    UnaryOperator::Void => "void ",
                    UnaryOperator::Delete => "delete ",
                    UnaryOperator::IsNullish => "__isNullish__", // Special marker for codegen
                };
                ReactiveValue::UnaryOp {
                    op: op_str.to_string(),
                    operand: operand.identifier.clone(),
                }
            }
            InstructionValue::Call { callee, args } => {
                ReactiveValue::Call {
                    callee: callee.identifier.clone(),
                    args: args.iter().map(|a| {
                        match a {
                            crate::hir::Argument::Regular(p) => ReactiveArgument::Regular(p.identifier.clone()),
                            crate::hir::Argument::Spread(p) => ReactiveArgument::Spread(p.identifier.clone()),
                        }
                    }).collect(),
                }
            }
            InstructionValue::Object { properties } => {
                ReactiveValue::Object {
                    properties: properties
                        .iter()
                        .map(|prop| {
                            match prop {
                                crate::hir::ObjectProperty::KeyValue { key, value } => {
                                    let reactive_key = match key {
                                        crate::hir::ObjectPropertyKey::Identifier(s) => ReactiveObjectKey::Identifier(s.clone()),
                                        crate::hir::ObjectPropertyKey::Computed(p) => ReactiveObjectKey::Computed(p.identifier.clone()),
                                    };
                                    ReactiveObjectProperty::KeyValue { key: reactive_key, value: value.identifier.clone() }
                                }
                                crate::hir::ObjectProperty::Spread(p) => ReactiveObjectProperty::Spread(p.identifier.clone()),
                            }
                        })
                        .collect(),
                }
            }
            InstructionValue::Array { elements } => {
                ReactiveValue::Array {
                    elements: elements.iter().map(|e| {
                        match e {
                            crate::hir::ArrayElement::Regular(p) => ReactiveArrayElement::Regular(p.identifier.clone()),
                            crate::hir::ArrayElement::Spread(p) => ReactiveArrayElement::Spread(p.identifier.clone()),
                            crate::hir::ArrayElement::Hole => ReactiveArrayElement::Hole,
                        }
                    }).collect(),
                }
            }
            InstructionValue::PropertyLoad { object, property } => {
                ReactiveValue::PropertyLoad {
                    object: object.identifier.clone(),
                    property: property.clone(),
                }
            }
            InstructionValue::PropertyStore { object, property, value } => {
                ReactiveValue::PropertyStore {
                    object: object.identifier.clone(),
                    property: property.clone(),
                    value: value.identifier.clone(),
                }
            }
            InstructionValue::ComputedLoad { object, property } => {
                ReactiveValue::ComputedLoad {
                    object: object.identifier.clone(),
                    property: property.identifier.clone(),
                }
            }
            InstructionValue::ComputedStore { object, property, value } => {
                ReactiveValue::ComputedStore {
                    object: object.identifier.clone(),
                    property: property.identifier.clone(),
                    value: value.identifier.clone(),
                }
            }
            InstructionValue::LoadLocal(place) => {
                ReactiveValue::LoadLocal(place.identifier.clone())
            }
            InstructionValue::StoreLocal(_, value) => {
                // StoreLocal becomes a LoadLocal (copy) after SSA
                ReactiveValue::LoadLocal(value.identifier.clone())
            }
            InstructionValue::Phi { operands } => {
                ReactiveValue::Phi {
                    operands: operands.iter().map(|(_, p)| p.identifier.clone()).collect(),
                }
            }
        };

        ReactiveInstruction {
            lvalue: instr.lvalue.identifier.clone(),
            value,
            scope: instr.scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reactive_value_serializes() {
        let val = ReactiveValue::Constant(ConstantValue::Number(42.0));
        let json = serde_json::to_string(&val).unwrap();
        assert!(json.contains("42"));
    }
}
