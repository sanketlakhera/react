use crate::hir::{
    BasicBlock, BlockId, HIRFunction, Identifier, InstructionValue, Place,
};
use std::collections::{HashMap, HashSet};

/// Maps each Identifier (SSA version) to its live range [start, end) in the linearized instruction stream.
pub struct LivenessResult {
    pub ranges: HashMap<Identifier, (usize, usize)>,
    pub aliases: DisjointSet,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentifierHash(String, usize); // Helper for using Identifier as key if needed, but Identifier is Hash

impl From<&Identifier> for IdentifierHash {
    fn from(id: &Identifier) -> Self {
        IdentifierHash(id.name.clone(), id.id)
    }
}

pub fn infer_liveness(func: &HIRFunction) -> LivenessResult {
    // 1. Linearize CFG (Reverse Post Order)
    // We reuse the logic from DominatorTree or just re-compute RPO.
    // For range analysis, we need a specific order. RPO is standard.
    
    // We'll compute RPO manually here to get the instruction list.
    let entry = func.entry_block;
    let mut po = Vec::new();
    let mut visited = HashSet::new();
    post_order(entry, &func.blocks, &mut visited, &mut po);
    let rpo: Vec<BlockId> = po.into_iter().rev().collect();

    // 2. Build Instruction Index and Alias Map
    // We assign a linear index to each instruction.
    // Also build DisjointSet for aliasing (Phi, LoadLocal).
    
    let mut instr_indices = HashMap::new(); // InstrId -> usize
    let mut current_index = 0;
    let mut aliases = DisjointSet::new();
    
    // We also need to map Identifier -> Definition Index (Start of range)
    let mut ranges: HashMap<Identifier, (usize, usize)> = HashMap::new();

    // Helper to register usage
    // We defer usage processing to a backward pass or just update 'end' in place?
    // Forward pass: set 'start'.
    // Usage updates 'end'.
    // Since we visit in RPO, we might see a use before def (backedge)?
    // Yes, in loops.
    // So 'start' is the definition index.
    // 'end' is the max usage index.
    
    // Linear scan to assign indices and process Definitions + Aliases
    for &block_id in &rpo {
        if let Some(block) = func.blocks.get(&block_id) {
            for instr in &block.instructions {
                instr_indices.insert(instr.id, current_index);
                
                // Definitions (LValue) start range here
                let lvalue = &instr.lvalue;
                // Initialize range [index, index + 1)
                ranges.insert(lvalue.identifier.clone(), (current_index, current_index + 1));
                
                // Handle Aliasing
                match &instr.value {
                    InstructionValue::LoadLocal(src) => {
                        aliases.union(&lvalue.identifier, &src.identifier);
                    }
                    InstructionValue::Phi { operands } => {
                        for (_, src) in operands {
                            aliases.union(&lvalue.identifier, &src.identifier);
                        }
                    }
                    _ => {}
                }
                
                current_index += 1;
            }
        }
    }

    // 3. Process Usages to extend ranges
    for &block_id in &rpo {
        if let Some(block) = func.blocks.get(&block_id) {
            for instr in &block.instructions {
                let idx = instr_indices[&instr.id];
                
                let mut mark_use = |place: &Place| {
                    if let Some(range) = ranges.get_mut(&place.identifier) {
                        range.1 = range.1.max(idx + 1);
                    }
                };

                match &instr.value {
                    InstructionValue::BinaryOp { left, right, .. } => {
                        mark_use(left);
                        mark_use(right);
                    }
                    InstructionValue::UnaryOp { operand, .. } => {
                        mark_use(operand);
                    }
                    InstructionValue::Call { callee, args } => {
                        mark_use(callee);
                        for arg in args {
                            match arg {
                                crate::hir::Argument::Regular(p) => mark_use(p),
                                crate::hir::Argument::Spread(p) => mark_use(p),
                            }
                        }
                    }
                    InstructionValue::PropertyStore { object, value, .. } => {
                        mark_use(object);
                        mark_use(value);
                    }
                    InstructionValue::ComputedStore { object, property, value } => {
                        mark_use(object);
                        mark_use(property);
                        mark_use(value);
                    }
                    InstructionValue::PropertyLoad { object, .. } => {
                        mark_use(object);
                    }
                    InstructionValue::ComputedLoad { object, property } => {
                        mark_use(object);
                        mark_use(property);
                    }
                    InstructionValue::Object { properties } => {
                        for prop in properties {
                            match prop {
                                crate::hir::ObjectProperty::KeyValue { key, value } => {
                                    if let crate::hir::ObjectPropertyKey::Computed(k) = key {
                                        mark_use(k);
                                    }
                                    mark_use(value);
                                }
                                crate::hir::ObjectProperty::Spread(p) => mark_use(p),
                            }
                        }
                    }
                    InstructionValue::Array { elements } => {
                        for elem in elements {
                            match elem {
                                crate::hir::ArrayElement::Regular(p) => mark_use(p),
                                crate::hir::ArrayElement::Spread(p) => mark_use(p),
                                crate::hir::ArrayElement::Hole => {}
                            }
                        }
                    }
                    InstructionValue::StoreLocal(_, val) => {
                        mark_use(val);
                    }
                    InstructionValue::LoadLocal(src) => {
                        mark_use(src);
                    }
                    InstructionValue::Phi { operands } => {
                        for (_, val) in operands {
                            mark_use(val);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 4. Merge Ranges based on Aliases
    let mut merged_ranges: HashMap<Identifier, (usize, usize)> = HashMap::new();
    
    // Identify roots and merge
    for (id, range) in &ranges {
        let root = aliases.find(id);
        let entry = merged_ranges.entry(root).or_insert(*range);
        entry.0 = entry.0.min(range.0);
        entry.1 = entry.1.max(range.1);
    }
    
    // Assign merged range to all identifiers
    let mut final_ranges = HashMap::new();
    for id in ranges.keys() {
        let root = aliases.find(id);
        if let Some(range) = merged_ranges.get(&root) {
            final_ranges.insert(id.clone(), *range);
        }
    }

    LivenessResult {
        ranges: final_ranges,
        aliases,
    }
}

fn post_order(
    current: BlockId,
    blocks: &std::collections::BTreeMap<BlockId, BasicBlock>,
    visited: &mut HashSet<BlockId>,
    po: &mut Vec<BlockId>
) {
    if !visited.insert(current) { return; }
    if let Some(block) = blocks.get(&current) {
        for succ in block.successors() {
            post_order(succ, blocks, visited, po);
        }
    }
    po.push(current);
}

// Simple Union-Find for Identifiers
pub struct DisjointSet {
    parents: HashMap<Identifier, Identifier>,
}

impl DisjointSet {
    pub fn new() -> Self {
        Self { parents: HashMap::new() }
    }

    pub fn find(&mut self, id: &Identifier) -> Identifier {
        if !self.parents.contains_key(id) {
            self.parents.insert(id.clone(), id.clone());
            return id.clone();
        }
        
        let parent = self.parents.get(id).unwrap().clone();
        if parent == *id {
            return parent;
        }
        
        let root = self.find(&parent);
        self.parents.insert(id.clone(), root.clone());
        root
    }

    pub fn union(&mut self, a: &Identifier, b: &Identifier) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a != root_b {
            self.parents.insert(root_a, root_b);
        }
    }
}
