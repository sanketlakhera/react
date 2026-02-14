use crate::hir::dominators::DominatorTree;
use crate::hir::{
    BlockId, HIRFunction, Identifier, InstrId, Instruction, InstructionValue, Place,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub fn enter_ssa(mut func: HIRFunction) -> HIRFunction {
    // 0. Compute Predecessors
    // We need to rebuild predecessors because lowering doesn't populate them fully/correctly
    // or they might be stale.
    for block in func.blocks.values_mut() {
        block.preds.clear();
    }
    
    // We need to collect edges first to avoid double borrow
    let mut edges = Vec::new();
    for (id, block) in &func.blocks {
        for succ in block.successors() {
            edges.push((*id, succ));
        }
    }

    for (pred, succ) in edges {
        if let Some(block) = func.blocks.get_mut(&succ) {
            block.preds.push(pred);
        }
    }

    // 1. Compute Dominator Tree
    let dom_tree = DominatorTree::compute(&func);

    // 2. Collect Globals (Variables defined in multiple blocks or just all locals?)
    // For simplicity, we treat all identifiers used in StoreLocal/LoadLocal as variables to promote.
    // We filter out temporaries "t..." if we can, but lowering uses StoreLocal for user variables only.
    // Temporaries are just lvalues of other instructions.
    let mut globals = BTreeSet::new();
    let mut blocks_defining_global: BTreeMap<String, BTreeSet<BlockId>> = BTreeMap::new();

    for block in func.blocks.values() {
        for instr in &block.instructions {
            if let InstructionValue::StoreLocal(place, _) = &instr.value {
                let name = place.identifier.name.clone();
                globals.insert(name.clone());
                blocks_defining_global
                    .entry(name)
                    .or_insert_with(BTreeSet::new)
                    .insert(block.id);
            }
        }
    }

    // 3. Insert Phis
    // For each global, insert trivial Phis at IDF
    let mut phi_placements: BTreeMap<BlockId, Vec<(String, InstrId)>> = BTreeMap::new();
    // We need to generate IDs for Phis.
    let mut max_instr_id = 0;
    for block in func.blocks.values() {
        for instr in &block.instructions {
            max_instr_id = max_instr_id.max(instr.id.0);
        }
    }
    let mut next_instr_id = max_instr_id + 1;

    for var in &globals {
        let def_blocks = blocks_defining_global.get(var).unwrap();
        let mut worklist: Vec<BlockId> = def_blocks.iter().cloned().collect();
        let mut visited = HashSet::new(); // Blocks where we added Phi for this var
        let mut has_phi = HashSet::new();
        
        while let Some(b) = worklist.pop() {
            if let Some(df) = dom_tree.dominance_frontiers.get(&b) {
                for &d in df {
                    if !has_phi.contains(&d) {
                        // Insert Phi for `var` at `d`
                        // We need a generic Phi instruction.
                        // We assign a new ID.
                        let phi_id = InstrId(next_instr_id);
                        next_instr_id += 1;
                        
                        phi_placements
                            .entry(d)
                            .or_insert_with(Vec::new)
                            .push((var.clone(), phi_id));
                        
                        has_phi.insert(d);
                        if !visited.contains(&d) {
                            visited.insert(d);
                            worklist.push(d);
                        }
                    }
                }
            }
        }
    }

    // Actually insert the Phi instructions into the blocks
    for (block_id, phis) in phi_placements {
        if let Some(block) = func.blocks.get_mut(&block_id) {
            for (var_name, instr_id) in phis {
                // Phi instruction
                // lvalue: we use a placeholder version 0 for now, rename pass will fix it.
                // operands: empty for now
                let phi_instr = Instruction {
                    id: instr_id,
                    lvalue: Place {
                        identifier: Identifier {
                            name: var_name.clone(),
                            id: 0, 
                        },
                    },
                    value: InstructionValue::Phi {
                        operands: Vec::new(),
                    },
                    scope: None,
                };
                block.instructions.insert(0, phi_instr);
            }
        }
    }

    // 4. Rename
    let mut rename_ctx = RenameContext {
        stacks: HashMap::new(),
        counters: HashMap::new(),
        dom_tree: &dom_tree,
    };
    
    // Initialize stacks
    for var in &globals {
        rename_ctx.stacks.insert(var.clone(), vec![0]); // Version 0 is undefined/entry
        rename_ctx.counters.insert(var.clone(), 1);
    }

    rename_block(func.entry_block, &mut func, &mut rename_ctx);

    func
}

struct RenameContext<'a> {
    stacks: HashMap<String, Vec<usize>>,
    counters: HashMap<String, usize>,
    dom_tree: &'a DominatorTree,
}

impl<'a> RenameContext<'a> {
    fn new_version(&mut self, name: &str) -> usize {
        let counter = self.counters.entry(name.to_string()).or_insert(0);
        let v = *counter;
        *counter += 1;
        self.stacks.get_mut(name).unwrap().push(v);
        v
    }

    fn current_version(&self, name: &str) -> usize {
        *self.stacks.get(name).and_then(|s| s.last()).unwrap_or(&0)
    }

    fn pop_version(&mut self, name: &str) {
        if let Some(stack) = self.stacks.get_mut(name) {
            stack.pop();
        }
    }
}

fn rename_block(
    block_id: BlockId,
    func: &mut HIRFunction,
    ctx: &mut RenameContext,
) {
    // We must iterate instructions. 
    // We cannot easily borrow `func` mutably for the block and then recursively call.
    // So we extract the logic.
    
    // 1. Rewrite Instructions
    // We need to mutate the block.
    // Let's just clone the block instructions to iterate? No, expensive.
    // We can index.
    
    let block = func.blocks.get_mut(&block_id).unwrap();
    
    // Keep track of which variables got a new version in this block to pop them later
    let mut pushed_vars = Vec::new();

    for instr in &mut block.instructions {
        match &mut instr.value {
            InstructionValue::Phi { .. } => {
                // Definition: Rename lvalue
                let name = instr.lvalue.identifier.name.clone();
                let new_v = ctx.new_version(&name);
                instr.lvalue.identifier.id = new_v;
                pushed_vars.push(name);
            }
            InstructionValue::LoadLocal(place) => {
                // Use: Rename place
                // Note: LoadLocal argument is the place we are loading FROM.
                let name = place.identifier.name.clone();
                let v = ctx.current_version(&name);
                place.identifier.id = v;
            }
            InstructionValue::StoreLocal(target, _val) => {
                // Definition: Rename target
                // Also, convert StoreLocal to "Move" (LoadLocal as identity)
                // because StoreLocal is side-effectual but we want explicit def.
                // The value `_val` is being read (used).
                // Wait, `val` is a Place. It should have been renamed if it was a LoadLocal result?
                // No, `val` is an operand. If it's a variable, it should have been loaded by a previous instruction.
                // In our lowering, `StoreLocal(x, val_place)`: `val_place` is typically a temporary.
                // Temporaries don't need SSA renaming if they are single-def (which they are).
                // So we assume `val` is fine.
                
                let name = target.identifier.name.clone();
                let new_v = ctx.new_version(&name);
                
                // Transform instruction:
                // Old: lvalue=temp (unused), value=StoreLocal(x, val)
                // New: lvalue=x_new, value=LoadLocal(val) -- essentially a copy
                
                let val_clone = _val.clone(); // Clone the source place
                
                // Update the instruction
                instr.lvalue = Place {
                    identifier: Identifier {
                        name: name.clone(),
                        id: new_v,
                    }
                };
                instr.value = InstructionValue::LoadLocal(val_clone);
                
                pushed_vars.push(name);
            }
            // For other instructions (BinaryOp, Call, etc.), operands are Places.
            // If those Places refer to Promotable variables, we should rename them.
            // BUT, our lowering logic uses LoadLocal to read variables into temporaries.
            // So `BinaryOp(t0, t1)` uses temporaries.
            // Temporaries (t0, t1) are NOT in our `globals` set (they are local to block/linear).
            // So we don't rename them here.
            // Only LoadLocal/StoreLocal refer to the "variables" we are promoting.
            _ => {}
        }
    }

    // 2. Update Phis in successors
    // We need to look at successors.
    // Note: We can't access `func` while holding `block`.
    // So we finish with `block` first.
    let successors = block.successors(); 
    let block_id_copy = block.id;

    // Drop mutable borrow of func (by scope end or re-borrow)
    // Actually, we are in a function taking `&mut func`.
    // We can't hold `block` reference.
    // We iterate instructions by index or using a separate pass?
    // Renaming must be recursive.
    // The pattern is:
    //   rename_local(block)
    //   rename_successors(block)
    //   recurse children
    //   pop stacks
    
    // We did rename_local above. But we held `block` mutably.
    // Now we need to update successors.
    
    // Successors are in `func.blocks`.
    for succ_id in successors {
        if let Some(succ_block) = func.blocks.get_mut(&succ_id) {
            for instr in &mut succ_block.instructions {
                if let InstructionValue::Phi { operands } = &mut instr.value {
                    let name = instr.lvalue.identifier.name.clone();
                    // If this Phi is for one of our variables
                    // (It must be, if we inserted it)
                    // Check if we track this var
                    if ctx.stacks.contains_key(&name) {
                         let v = ctx.current_version(&name);
                         let place = Place {
                             identifier: Identifier {
                                 name: name,
                                 id: v,
                             }
                         };
                         operands.push((block_id_copy, place));
                    }
                } else {
                    // Phis are at the start.
                    break;
                }
            }
        }
    }

    // 3. Recurse into dominator tree children
    // We need the children of `block_id` in the dom tree.
    // `dom_tree.idoms` maps child -> parent.
    // We need parent -> children.
    // Optimization: Build the tree structure once.
    // For now, scan idoms (inefficient but works).
    
    // We have to be careful about borrowing `func` recursively.
    // Rust doesn't like passing `&mut func` recursively.
    // However, `rename_block` processes ONE block, then calls `rename_block` for others.
    // This is fine.
    
    // But we need to know the children.
    let mut children = Vec::new();
    for (&child, &parent) in &ctx.dom_tree.idoms {
        if parent == block_id && child != block_id {
            children.push(child);
        }
    }
    // Sort children for determinism? RPO would be better, but child order in dom tree doesn't strictly matter for correctness, just consistent output.
    children.sort(); 

    for child in children {
        rename_block(child, func, ctx);
    }

    // 4. Pop stacks
    for var in pushed_vars {
        ctx.pop_version(&var);
    }
}
