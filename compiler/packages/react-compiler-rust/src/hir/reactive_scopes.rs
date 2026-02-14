//! Reactive Scope Construction
//!
//! This module implements the core memoization logic of the React Compiler.
//! It groups instructions into reactive scopes that can be memoized using `useMemo`.
//!
//! The algorithm:
//! 1. Infer initial scopes based on liveness ranges (values that need memoization)
//! 2. Align scopes to safe boundaries (statement boundaries)
//! 3. Merge overlapping scopes when dependencies are entangled
//! 4. Propagate dependencies (inputs) for each scope

use crate::hir::inference::LivenessResult;
use crate::hir::scope::{Declaration, Dependency, ReactiveScope, ScopeId};
use crate::hir::{
    BasicBlock, BlockId, HIRFunction, Identifier, Instruction, InstructionValue, Place,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

/// Result of reactive scope construction
#[derive(Debug)]
pub struct ReactiveScopeResult {
    /// All reactive scopes in the function
    pub scopes: Vec<ReactiveScope>,
    /// Mapping from instruction index to scope ID (if any)
    pub instruction_scopes: HashMap<usize, ScopeId>,
}

/// Context for scope inference
struct ScopeInferenceContext {
    next_scope_id: usize,
    /// Maps identifier to the scope it defines
    identifier_to_scope: HashMap<Identifier, ScopeId>,
}

impl ScopeInferenceContext {
    fn new() -> Self {
        Self {
            next_scope_id: 0,
            identifier_to_scope: HashMap::new(),
        }
    }

    fn create_scope(&mut self, range: (usize, usize)) -> ReactiveScope {
        let id = ScopeId(self.next_scope_id);
        self.next_scope_id += 1;
        ReactiveScope {
            id,
            range,
            dependencies: Vec::new(),
            declarations: Vec::new(),
        }
    }
}

/// Main entry point: construct reactive scopes for a function
pub fn construct_reactive_scopes(
    func: &HIRFunction,
    liveness: &LivenessResult,
) -> ReactiveScopeResult {
    // Step 1: Infer initial scopes based on liveness
    let mut scopes = infer_scopes(func, liveness);

    // Step 2: Align scopes to statement boundaries
    align_scopes(&mut scopes, func);

    // Step 3: Merge overlapping scopes
    let scopes = merge_scopes(scopes);

    // Step 4: Propagate dependencies
    let scopes = propagate_dependencies(func, scopes, liveness);

    // Build instruction -> scope mapping
    let mut instruction_scopes = HashMap::new();
    for scope in &scopes {
        for idx in scope.range.0..scope.range.1 {
            instruction_scopes.insert(idx, scope.id);
        }
    }

    ReactiveScopeResult {
        scopes,
        instruction_scopes,
    }
}

/// Step 1: Infer scopes based on liveness ranges
///
/// Each value with a non-trivial live range (used beyond its definition point)
/// is a candidate for memoization. We create a scope that covers its live range.
fn infer_scopes(_func: &HIRFunction, liveness: &LivenessResult) -> Vec<ReactiveScope> {
    let mut ctx = ScopeInferenceContext::new();
    let mut scopes = Vec::new();

    // Collect and sort ranges for deterministic iteration
    let mut sorted_ranges: Vec<_> = liveness
        .ranges
        .iter()
        .filter(|(id, range)| {
            let (start, end) = **range;
            // Skip trivial ranges (single instruction, not memoizable)
            if end - start <= 1 {
                return false;
            }
            // Skip temporaries (t0, t1, etc.) - they're internal
            if id.name.starts_with('t') && id.name[1..].chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
            true
        })
        .collect();

    // Sort by (start, name, id) for deterministic ordering
    sorted_ranges.sort_by_key(|(id, range)| (range.0, id.name.clone(), id.id));

    for (id, range) in sorted_ranges {
        let (start, end) = *range;
        let scope = ctx.create_scope((start, end));
        ctx.identifier_to_scope.insert(id.clone(), scope.id);
        scopes.push(scope);
    }

    scopes
}

/// Step 2: Align scopes to statement boundaries
///
/// Scopes should start and end at clean statement boundaries,
/// not in the middle of expressions.
fn align_scopes(scopes: &mut Vec<ReactiveScope>, _func: &HIRFunction) {
    // For now, we use a simple alignment: scopes stay as-is
    // since our instruction indices already correspond to statement-level operations.
    // In a more advanced implementation, we'd analyze the CFG to find
    // safe insertion points for useMemo.

    // Sort scopes by start position
    scopes.sort_by_key(|s| s.range.0);
}

/// Step 3: Merge overlapping scopes
///
/// If two scopes overlap, they must be merged because:
/// - They have entangled dependencies
/// - Separating them would break the memoization invariant
fn merge_scopes(mut scopes: Vec<ReactiveScope>) -> Vec<ReactiveScope> {
    if scopes.is_empty() {
        return scopes;
    }

    // Sort by start position
    scopes.sort_by_key(|s| s.range.0);

    let mut merged: Vec<ReactiveScope> = Vec::new();

    for scope in scopes {
        if let Some(last) = merged.last_mut() {
            // Check for overlap: if current scope starts before last scope ends
            if scope.range.0 < last.range.1 {
                // Merge: extend end to cover both
                last.range.1 = last.range.1.max(scope.range.1);
                // Merge declarations and dependencies will be done in propagation step
            } else {
                merged.push(scope);
            }
        } else {
            merged.push(scope);
        }
    }

    merged
}

/// Step 4: Propagate dependencies for each scope
///
/// A dependency is a value that:
/// - Is used inside the scope
/// - Is defined outside the scope
fn propagate_dependencies(
    func: &HIRFunction,
    mut scopes: Vec<ReactiveScope>,
    liveness: &LivenessResult,
) -> Vec<ReactiveScope> {
    // Linearize instructions (RPO order)
    let (instructions, _) = linearize_instructions(func);

    for scope in &mut scopes {
        let mut deps: BTreeSet<(String, usize)> = BTreeSet::new();
        let mut decls: BTreeSet<(String, usize)> = BTreeSet::new();

        // Collect all uses and definitions within the scope
        for idx in scope.range.0..scope.range.1 {
            if idx >= instructions.len() {
                break;
            }

            let instr = &instructions[idx];

            // Record definition (lvalue)
            let id = &instr.lvalue.identifier;
            decls.insert((id.name.clone(), id.id));

            // Record uses (operands)
            for used in get_operand_identifiers(&instr.value) {
                // If this use is defined outside the scope, it's a dependency
                if let Some(&(def_start, _)) = liveness.ranges.get(&used) {
                    if def_start < scope.range.0 {
                        deps.insert((used.name.clone(), used.id));
                    }
                }
            }
        }

        // Convert to Dependency/Declaration structs (sorted for deterministic output)
        scope.dependencies = deps
            .into_iter()
            .map(|(name, id)| Dependency {
                place: Place {
                    identifier: Identifier { name, id },
                },
            })
            .collect();

        scope.declarations = decls
            .into_iter()
            .map(|(name, id)| Declaration {
                place: Place {
                    identifier: Identifier { name, id },
                },
            })
            .collect();
    }

    scopes
}

/// Linearize instructions in Reverse Post Order (same as liveness analysis)
fn linearize_instructions(func: &HIRFunction) -> (Vec<&Instruction>, Vec<BlockId>) {
    let entry = func.entry_block;
    let mut po = Vec::new();
    let mut visited = HashSet::new();
    post_order(entry, &func.blocks, &mut visited, &mut po);
    let rpo: Vec<BlockId> = po.into_iter().rev().collect();

    let mut instructions = Vec::new();
    for &block_id in &rpo {
        if let Some(block) = func.blocks.get(&block_id) {
            for instr in &block.instructions {
                instructions.push(instr);
            }
        }
    }

    (instructions, rpo)
}

fn post_order(
    current: BlockId,
    blocks: &BTreeMap<BlockId, BasicBlock>,
    visited: &mut HashSet<BlockId>,
    po: &mut Vec<BlockId>,
) {
    if !visited.insert(current) {
        return;
    }
    if let Some(block) = blocks.get(&current) {
        for succ in block.successors() {
            post_order(succ, blocks, visited, po);
        }
    }
    po.push(current);
}

/// Extract identifiers used as operands in an instruction
fn get_operand_identifiers(value: &InstructionValue) -> Vec<Identifier> {
    let mut result = Vec::new();

    match value {
        InstructionValue::BinaryOp { left, right, .. } => {
            result.push(left.identifier.clone());
            result.push(right.identifier.clone());
        }
        InstructionValue::UnaryOp { operand, .. } => {
            result.push(operand.identifier.clone());
        }
        InstructionValue::Call { callee, args } => {
            result.push(callee.identifier.clone());
            for arg in args {
                match arg {
                    crate::hir::Argument::Regular(p) => result.push(p.identifier.clone()),
                    crate::hir::Argument::Spread(p) => result.push(p.identifier.clone()),
                }
            }
        }
        InstructionValue::Object { properties } => {
            for prop in properties {
                match prop {
                    crate::hir::ObjectProperty::KeyValue { key, value } => {
                        if let crate::hir::ObjectPropertyKey::Computed(k) = key {
                            result.push(k.identifier.clone());
                        }
                        result.push(value.identifier.clone());
                    }
                    crate::hir::ObjectProperty::Spread(p) => result.push(p.identifier.clone()),
                }
            }
        }
        InstructionValue::Array { elements } => {
            for elem in elements {
                match elem {
                    crate::hir::ArrayElement::Regular(p) => result.push(p.identifier.clone()),
                    crate::hir::ArrayElement::Spread(p) => result.push(p.identifier.clone()),
                    crate::hir::ArrayElement::Hole => {}
                }
            }
        }
        InstructionValue::PropertyLoad { object, .. } => {
            result.push(object.identifier.clone());
        }
        InstructionValue::PropertyStore { object, value, .. } => {
            result.push(object.identifier.clone());
            result.push(value.identifier.clone());
        }
        InstructionValue::ComputedLoad { object, property } => {
            result.push(object.identifier.clone());
            result.push(property.identifier.clone());
        }
        InstructionValue::ComputedStore {
            object,
            property,
            value,
        } => {
            result.push(object.identifier.clone());
            result.push(property.identifier.clone());
            result.push(value.identifier.clone());
        }
        InstructionValue::LoadLocal(place) => {
            result.push(place.identifier.clone());
        }
        InstructionValue::StoreLocal(_, val) => {
            result.push(val.identifier.clone());
        }
        InstructionValue::Phi { operands } => {
            for (_, place) in operands {
                result.push(place.identifier.clone());
            }
        }
        InstructionValue::Constant(_) => {}
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_overlapping_scopes() {
        let scopes = vec![
            ReactiveScope {
                id: ScopeId(0),
                range: (0, 5),
                dependencies: vec![],
                declarations: vec![],
            },
            ReactiveScope {
                id: ScopeId(1),
                range: (3, 8),
                dependencies: vec![],
                declarations: vec![],
            },
            ReactiveScope {
                id: ScopeId(2),
                range: (10, 15),
                dependencies: vec![],
                declarations: vec![],
            },
        ];

        let merged = merge_scopes(scopes);

        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].range, (0, 8)); // First two merged
        assert_eq!(merged[1].range, (10, 15)); // Third unchanged
    }
}
