use crate::hir::{BasicBlock, BlockId, HIRFunction};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

pub struct DominatorTree {
    /// Map from a block to its immediate dominator.
    pub idoms: BTreeMap<BlockId, BlockId>,
    /// Map from a block to the set of blocks in its dominance frontier.
    pub dominance_frontiers: BTreeMap<BlockId, BTreeSet<BlockId>>,
}

impl DominatorTree {
    pub fn compute(func: &HIRFunction) -> Self {
        let entry = func.entry_block;
        let blocks = &func.blocks;
        let num_blocks = blocks.len();

        // 1. Compute Post Order Traversal (PO)
        // We use post-order for the iterative algorithm convergence speed.
        // We actually want "Reverse Post Order" (RPO) for the iteration, 
        // but the standard algorithm often describes just needing *some* traversal, RPO being best.
        let mut po = Vec::with_capacity(num_blocks);
        let mut visited = HashSet::new();
        post_order(entry, blocks, &mut visited, &mut po);
        
        // Reverse Post Order
        let rpo: Vec<BlockId> = po.iter().rev().cloned().collect();
        let rpo_indices: HashMap<BlockId, usize> = rpo.iter().enumerate().map(|(i, &b)| (b, i)).collect();
        
        // 2. Compute Immediate Dominators (Iterative Algorithm)
        // doms[entry] = entry
        let mut idoms: BTreeMap<BlockId, BlockId> = BTreeMap::new();
        idoms.insert(entry, entry);

        let mut changed = true;
        while changed {
            changed = false;
            for &b in &rpo {
                if b == entry {
                    continue;
                }

                // New idom is the intersection of idoms of all processed predecessors
                let mut new_idom: Option<BlockId> = None;
                
                let current_preds = &blocks[&b].preds;
                for &p in current_preds {
                    if idoms.contains_key(&p) {
                        if let Some(current) = new_idom {
                            new_idom = Some(intersect(&idoms, &rpo_indices, current, p));
                        } else {
                            new_idom = Some(p);
                        }
                    }
                }

                if let Some(new_idom) = new_idom {
                    if idoms.get(&b) != Some(&new_idom) {
                        idoms.insert(b, new_idom);
                        changed = true;
                    }
                }
            }
        }

        // 3. Compute Dominance Frontiers
        // DF(b) = { runner | runner is a predecessor of b, but runner is not strictly dominated by b }
        // Algorithm:
        // for each block b:
        //   if preds(b).len() >= 2:
        //     for each p in preds(b):
        //       runner = p
        //       while runner != idom(b):
        //         add b to DF(runner)
        //         runner = idom(runner)
        
        let mut dominance_frontiers: BTreeMap<BlockId, BTreeSet<BlockId>> = BTreeMap::new();
        // Initialize sets
        for b in blocks.keys() {
            dominance_frontiers.insert(*b, BTreeSet::new());
        }

        for (&b, block) in blocks {
            if block.preds.len() >= 2 {
                for &p in &block.preds {
                    let mut runner = p;
                    // We need to handle reachable blocks only. 
                    // If p is not reachable (no idom), skip.
                    if !idoms.contains_key(&runner) {
                        continue;
                    }
                    
                    let b_idom = match idoms.get(&b) {
                        Some(&id) => id,
                        None => continue, // b is unreachable
                    };

                    while runner != b_idom {
                        dominance_frontiers.get_mut(&runner).unwrap().insert(b);
                        
                        // Move up the dominator tree
                        if let Some(&next) = idoms.get(&runner) {
                             // prevent infinite loop if runner == next (entry) but entry != b_idom?
                             // entry idom is entry.
                             if runner == next { break; }
                             runner = next;
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        Self {
            idoms,
            dominance_frontiers,
        }
    }
}

fn post_order(
    current: BlockId,
    blocks: &BTreeMap<BlockId, BasicBlock>,
    visited: &mut HashSet<BlockId>,
    po: &mut Vec<BlockId>
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

// Intersect: Find the common dominator of b1 and b2
// We need post-order indices to check progress?
// Actually, standard intersect uses RPO index comparison.
// We need a map from BlockId -> RPO index.
fn intersect(
    idoms: &BTreeMap<BlockId, BlockId>,
    rpo_indices: &HashMap<BlockId, usize>,
    mut b1: BlockId,
    mut b2: BlockId
) -> BlockId {
    let mut idx1 = rpo_indices[&b1];
    let mut idx2 = rpo_indices[&b2];

    while idx1 != idx2 {
        while idx1 > idx2 {
            b1 = idoms[&b1];
            idx1 = rpo_indices[&b1];
        }
        while idx2 > idx1 {
            b2 = idoms[&b2];
            idx2 = rpo_indices[&b2];
        }
    }
    b1
}
