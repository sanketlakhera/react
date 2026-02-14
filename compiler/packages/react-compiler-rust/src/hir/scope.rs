use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScopeId(pub usize);

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactiveScope {
    pub id: ScopeId,
    // Range of instruction IDs covered by this scope (inclusive start, exclusive end?)
    // In React Compiler, it's roughly (start, end) based on instruction IDs.
    pub range: (usize, usize),
    
    // Dependencies (inputs) and Declarations (outputs)
    pub dependencies: Vec<Dependency>,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Dependency {
    pub place: crate::hir::Place,
    // path?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Declaration {
    pub place: crate::hir::Place,
}
