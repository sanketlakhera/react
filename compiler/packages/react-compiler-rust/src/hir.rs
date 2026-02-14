pub mod lowering;
pub mod dominators;
pub mod ssa;
pub mod scope;
pub mod inference;
pub mod reactive_scopes;
pub mod reactive_function;

use scope::ScopeId;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// A unique identifier for a basic block within a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct BlockId(pub usize);

/// A unique identifier for an instruction/value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InstrId(pub usize);

/// A High-Level Intermediate Representation (HIR) of a function.
/// It is represented as a Control Flow Graph (CFG) of BasicBlocks.
#[derive(Debug, Serialize, Deserialize)]
pub struct HIRFunction {
    /// The name of the function (if any).
    pub name: Option<String>,
    /// The parameters of the function.
    pub params: Vec<Identifier>,
    /// The entry block of the function.
    pub entry_block: BlockId,
    /// All basic blocks in the function, indexed by their ID.
    /// All basic blocks in the function, indexed by their ID.
    pub blocks: BTreeMap<BlockId, BasicBlock>,
    /// Set of blocks that are loop headers.
    pub loop_headers: HashSet<BlockId>,
}

/// A BasicBlock contains a linear sequence of instructions that ends with a terminal.
#[derive(Debug, Serialize, Deserialize)]
pub struct BasicBlock {
    pub id: BlockId,
    /// The list of instructions in this block (excluding the terminal).
    pub instructions: Vec<Instruction>,
    /// The terminal instruction that determines control flow out of this block.
    pub terminal: Terminal,
    /// Predecessor blocks (control flow enters from these blocks).
    pub preds: Vec<BlockId>,
}

impl BasicBlock {
    pub fn successors(&self) -> Vec<BlockId> {
        match &self.terminal {
            Terminal::Goto(target) => vec![*target],
            Terminal::If { consequent, alternate, .. } => vec![*consequent, *alternate],
            Terminal::Return(_) => vec![],
            Terminal::Switch { cases, default, .. } => {
                let mut succs = Vec::with_capacity(cases.len() + 1);
                for (_, target) in cases {
                    succs.push(*target);
                }
                succs.push(*default);
                succs
            }
        }
    }
}

/// A single instruction in the HIR: `lvalue = opcode operands`.
#[derive(Debug, Serialize, Deserialize)]
pub struct Instruction {
    pub id: InstrId,
    pub lvalue: Place,
    pub value: InstructionValue,
    /// The reactive scope this instruction belongs to (if any).
    pub scope: Option<ScopeId>,
}

/// Represents a location where a value is stored (e.g., a variable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Place {
    pub identifier: Identifier,
    // TODO: Add effect/kind (e.g., read, mutate)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub id: usize, // Unique ID for this specific identifier instance
}

/// Represents an argument in a function call or array/object element.
/// Can be a regular value or a spread expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argument {
    /// Regular argument: value
    Regular(Place),
    /// Spread argument: ...value
    Spread(Place),
}

/// Represents a property in an object literal.
/// Can be a key-value pair or a spread expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectProperty {
    /// Regular property: key: value
    KeyValue { key: ObjectPropertyKey, value: Place },
    /// Spread property: ...value
    Spread(Place),
}

/// Represents an object property key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectPropertyKey {
    /// Static identifier key: { foo: value }
    Identifier(String),
    /// Computed key: { [expr]: value }
    Computed(Place),
}

/// Represents an element in an array literal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArrayElement {
    /// Regular element
    Regular(Place),
    /// Spread element: ...value
    Spread(Place),
    /// Hole (elision): [1, , 3]
    Hole,
}

/// The operation being performed in an instruction.
#[derive(Debug, Serialize, Deserialize)]
pub enum InstructionValue {
    /// A constant value (number, string, etc.)
    Constant(Constant),
    /// A binary operation (e.g., a + b)
    BinaryOp {
        op: BinaryOperator,
        left: Place,
        right: Place,
    },
    /// A unary operation (e.g., !a, -a)
    UnaryOp {
        op: UnaryOperator,
        operand: Place,
    },
    /// A function call
    Call {
        callee: Place,
        args: Vec<Argument>,
    },
    /// Create an object literal: { key: value, ... }
    Object {
        properties: Vec<ObjectProperty>,
    },
    /// Create an array literal: [value, ...]
    Array {
        elements: Vec<ArrayElement>,
    },
    /// Read a static property: object.property
    PropertyLoad {
        object: Place,
        property: String,
    },
    /// Write to a static property: object.property = value
    PropertyStore {
        object: Place,
        property: String,
        value: Place,
    },
    /// Read a computed property: object[property]
    ComputedLoad {
        object: Place,
        property: Place,
    },
    /// Write to a computed property: object[property] = value
    ComputedStore {
        object: Place,
        property: Place,
        value: Place,
    },
    /// Load a value from a local variable/binding
    LoadLocal(Place),
    /// Store a value into a local variable/binding (lvalue, value)
    StoreLocal(Place, Place),
    /// Phi node: merges values from predecessor blocks.
    Phi {
        operands: Vec<(BlockId, Place)>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Constant {
    Int(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    And,
    Or,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    InstanceOf,
    In,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UnaryOperator {
    /// Logical NOT: !x
    Not,
    /// Numeric negation: -x
    Negate,
    /// Unary plus: +x
    Plus,
    /// Bitwise NOT: ~x
    BitwiseNot,
    /// Typeof: typeof x
    TypeOf,
    /// Void operator: void x
    Void,
    /// Delete operator: delete x
    Delete,
    /// Check if value is null or undefined (for ?? operator)
    IsNullish,
}

/// Terminal instructions determine how control flow leaves a block.
#[derive(Debug, Serialize, Deserialize)]
pub enum Terminal {
    /// Jump to a single target block.
    Goto(BlockId),
    /// Conditional jump.
    If {
        test: Place,
        consequent: BlockId,
        alternate: BlockId,
    },
    /// Return from the function.
    Return(Option<Place>),
    /// A switch statement (switch test { case val: goto target; ... default: goto default })
    Switch {
        test: Place,
        cases: Vec<(Place, BlockId)>,
        default: BlockId,
        merge_target: Option<BlockId>, // Target for 'break'
    },
    // Throw, etc.
}

