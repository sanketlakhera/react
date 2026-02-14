# React Compiler Rust - Remaining Work

This document outlines all the pending features and improvements needed in the React Compiler Rust project. Each section includes detailed explanations, examples, and implementation hints to help developers understand and implement these features.

---

## Table of Contents

1. [Control Flow Statements](#1-control-flow-statements)
   - [For Loops](#11-for-loops)
   - [For-In Loops](#12-for-in-loops)
   - [For-Of Loops](#13-for-of-loops)
   - [Switch Statements](#14-switch-statements)
   - [Try-Catch-Finally](#15-try-catch-finally)
   - [Break and Continue](#16-break-and-continue)
2. [Operators](#2-operators)
   - [Bitwise Operators](#21-bitwise-operators)
   - [instanceof Operator](#22-instanceof-operator)
   - [in Operator](#23-in-operator)
   - [Unary Operators](#24-unary-operators)
   - [Update Expressions](#25-update-expressions)
3. [Functions and Classes](#3-functions-and-classes)
   - [Arrow Functions](#31-arrow-functions)
   - [Class Declarations](#32-class-declarations)
   - [Method Definitions](#33-method-definitions)
   - [Getters and Setters](#34-getters-and-setters)
4. [Advanced Patterns](#4-advanced-patterns)
   - [Nested Destructuring](#41-nested-destructuring)
   - [Default Values in Destructuring](#42-default-values-in-destructuring)
   - [Rest Patterns](#43-rest-patterns)
   - [Optional Chaining](#44-optional-chaining)
5. [Expressions](#5-expressions)
   - [Template Literals](#51-template-literals)
   - [Tagged Templates](#52-tagged-templates)
   - [New Expressions](#53-new-expressions)
   - [Sequence Expressions](#54-sequence-expressions)
   - [Conditional (Ternary) Expressions](#55-conditional-ternary-expressions)
   - [Await Expressions](#56-await-expressions)
   - [Yield Expressions](#57-yield-expressions)
6. [Code Quality](#6-code-quality)
   - [Control Flow Reconstruction Issues](#61-control-flow-reconstruction-issues)
   - [Unused Variable Elimination](#62-unused-variable-elimination)
   - [Better Error Messages](#63-better-error-messages)

---

## 1. Control Flow Statements

### 1.1 For Loops

**Current Status:** Not implemented - silently ignored

**Location:** `src/hir/lowering.rs` in `lower_statement()` function (around line 151)

**What it should do:**
Transform JavaScript `for` loops into HIR control flow graph with proper loop structure.

**Example that currently doesn't work:**
```javascript
function sum() {
  let total = 0;
  for (let i = 0; i < 10; i++) {
    total += i;
  }
  return total;
}
```

**Expected HIR structure:**
```
Block 0 (entry):
  - Initialize: let total = 0
  - Initialize: let i = 0
  - Goto Block 1 (header)

Block 1 (header/test):
  - Test: i < 10
  - If true -> Block 2 (body)
  - If false -> Block 3 (exit)

Block 2 (body):
  - total += i
  - Goto Block 4 (update)

Block 4 (update):
  - i++
  - Goto Block 1 (header)

Block 3 (exit):
  - return total
```

**Implementation hints:**
1. Find the `Statement::ForStatement` case in `lower_statement()`
2. A for loop has 4 parts: `init`, `test`, `update`, `body`
3. Create 4 blocks: current (init), header (test), body, update
4. Lower the `init` in the current block
5. Create header block, lower the `test`, terminate with `If` terminal
6. Create body block, lower the `body` statement
7. Create update block, lower the `update` expression
8. Connect: init -> header, header (true) -> body, body -> update, update -> header, header (false) -> exit

**Code template:**
```rust
Statement::ForStatement(for_stmt) => {
    // 1. Lower initialization in current block
    if let Some(init) = &for_stmt.init {
        match init {
            ast::ForStatementInit::VariableDeclaration(decl) => {
                // Handle variable declaration
                self.lower_statement(&Statement::VariableDeclaration(decl.clone()));
            }
            ast::ForStatementInit::Expression(expr) => {
                self.lower_expression(expr);
            }
            _ => {}
        }
    }

    // 2. Create block IDs
    let header_block_id = self.next_block_id();
    let body_block_id = self.next_block_id();
    let update_block_id = self.next_block_id();
    let exit_block_id = self.next_block_id();

    // 3. Jump to header from current
    self.terminate_block(Terminal::Goto(header_block_id));

    // 4. Header block - test condition
    self.start_block(header_block_id);
    let test = if let Some(test_expr) = &for_stmt.test {
        self.lower_expression(test_expr)
    } else {
        // No test means infinite loop (for(;;))
        self.push_instruction(InstructionValue::Constant(Constant::Boolean(true)))
    };
    self.terminate_block(Terminal::If {
        test,
        consequent: body_block_id,
        alternate: exit_block_id,
    });

    // 5. Body block
    self.start_block(body_block_id);
    self.lower_statement(&for_stmt.body);
    if !self.is_block_terminated(body_block_id) {
        self.terminate_block(Terminal::Goto(update_block_id));
    }

    // 6. Update block
    self.start_block(update_block_id);
    if let Some(update_expr) = &for_stmt.update {
        self.lower_expression(update_expr);
    }
    self.terminate_block(Terminal::Goto(header_block_id));

    // 7. Exit block
    self.start_block(exit_block_id);
}
```

---

### 1.2 For-In Loops

**Current Status:** Not implemented

**Location:** `src/hir/lowering.rs` in `lower_statement()`

**What it should do:**
Transform `for (key in object)` loops. These iterate over enumerable property names.

**Example:**
```javascript
function logKeys(obj) {
  for (const key in obj) {
    console.log(key);
  }
}
```

**Implementation hints:**
1. For-in loops need a special iterator instruction
2. You may need to add new HIR instructions like `ForInInit` and `ForInNext`
3. The simpler approach: treat as unsupported and skip the body (current behavior)
4. Full implementation requires runtime support for property enumeration

**Suggested new HIR instructions:**
```rust
/// Initialize a for-in iterator
ForInInit { object: Place },

/// Get next key from for-in iterator, returns (done: bool, key: string)
ForInNext { iterator: Place },
```

---

### 1.3 For-Of Loops

**Current Status:** Not implemented

**Location:** `src/hir/lowering.rs` in `lower_statement()`

**What it should do:**
Transform `for (item of iterable)` loops. These iterate over iterable values.

**Example:**
```javascript
function sumArray(arr) {
  let total = 0;
  for (const num of arr) {
    total += num;
  }
  return total;
}
```

**Implementation hints:**
Similar to for-in, but iterates over values instead of keys. Needs iterator protocol support.

---

### 1.4 Switch Statements

**Current Status:** Not implemented

**Location:** `src/hir/lowering.rs` in `lower_statement()`

**What it should do:**
Transform switch statements into a series of conditional jumps.

**Example:**
```javascript
function getDayName(day) {
  switch (day) {
    case 0:
      return "Sunday";
    case 1:
      return "Monday";
    default:
      return "Unknown";
  }
}
```

**Expected HIR structure:**
```
Block 0:
  - discriminant = day
  - t0 = discriminant === 0
  - If t0 -> Block 1, else -> Block 2

Block 1 (case 0):
  - return "Sunday"

Block 2:
  - t1 = discriminant === 1
  - If t1 -> Block 3, else -> Block 4

Block 3 (case 1):
  - return "Monday"

Block 4 (default):
  - return "Unknown"
```

**Implementation hints:**
1. Lower the discriminant expression first
2. For each case, create a comparison and conditional jump
3. Handle `default` case as the final fallthrough
4. Handle `break` statements (need to track the exit block)
5. Handle fallthrough (when no break between cases)

**Code template:**
```rust
Statement::SwitchStatement(switch_stmt) => {
    let discriminant = self.lower_expression(&switch_stmt.discriminant);
    let exit_block_id = self.next_block_id();

    // Track case blocks for fallthrough
    let mut case_blocks: Vec<BlockId> = Vec::new();
    let mut default_block: Option<BlockId> = None;

    // First pass: create all case blocks
    for case in &switch_stmt.cases {
        let case_block = self.next_block_id();
        case_blocks.push(case_block);
        if case.test.is_none() {
            default_block = Some(case_block);
        }
    }

    // Second pass: create test chain
    let mut current_test_block = self.current_block_id;
    for (i, case) in switch_stmt.cases.iter().enumerate() {
        if let Some(test) = &case.test {
            // Create comparison: discriminant === test
            let test_val = self.lower_expression(test);
            let cmp = self.push_instruction(InstructionValue::BinaryOp {
                op: BinaryOperator::StrictEqual,
                left: discriminant.clone(),
                right: test_val,
            });

            let next_test_block = if i + 1 < switch_stmt.cases.len() {
                self.next_block_id()
            } else {
                default_block.unwrap_or(exit_block_id)
            };

            self.terminate_block(Terminal::If {
                test: cmp,
                consequent: case_blocks[i],
                alternate: next_test_block,
            });

            self.start_block(next_test_block);
            current_test_block = next_test_block;
        }
    }

    // Third pass: lower case bodies
    for (i, case) in switch_stmt.cases.iter().enumerate() {
        self.start_block(case_blocks[i]);
        for stmt in &case.consequent {
            self.lower_statement(stmt);
        }
        // Fallthrough to next case or exit
        let next = if i + 1 < case_blocks.len() {
            case_blocks[i + 1]
        } else {
            exit_block_id
        };
        if !self.is_block_terminated(case_blocks[i]) {
            self.terminate_block(Terminal::Goto(next));
        }
    }

    self.start_block(exit_block_id);
}
```

---

### 1.5 Try-Catch-Finally

**Current Status:** Not implemented

**Location:** `src/hir/lowering.rs` in `lower_statement()`

**What it should do:**
Handle exception handling blocks.

**Example:**
```javascript
function safeParse(json) {
  try {
    return JSON.parse(json);
  } catch (e) {
    return null;
  } finally {
    console.log("done");
  }
}
```

**Implementation hints:**
1. This requires adding new terminal types to the HIR
2. Need to track exception handlers in the CFG
3. Consider adding:
   - `Terminal::TryStart { handler: BlockId, finally: Option<BlockId> }`
   - `Terminal::Throw(Place)`

**Complexity:** HIGH - requires significant HIR changes

**Suggested approach for MVP:**
Just lower the try block and ignore catch/finally:
```rust
Statement::TryStatement(try_stmt) => {
    // For now, just lower the try block
    self.lower_statement(&try_stmt.block);
    // TODO: Handle catch and finally
}
```

---

### 1.6 Break and Continue

**Current Status:** Not implemented

**Location:** `src/hir/lowering.rs` in `lower_statement()`

**What it should do:**
Jump to loop exit (break) or loop header (continue).

**Example:**
```javascript
function findFirst(arr, target) {
  for (let i = 0; i < arr.length; i++) {
    if (arr[i] === target) {
      break;
    }
  }
}
```

**Implementation hints:**
1. Need to track the current loop's exit block (for break) and header block (for continue)
2. Add fields to `LoweringContext`:
   ```rust
   struct LoweringContext {
       // ... existing fields ...
       loop_exit_stack: Vec<BlockId>,    // for break
       loop_header_stack: Vec<BlockId>,  // for continue
   }
   ```
3. Push to stack when entering a loop, pop when exiting
4. Handle labeled statements for labeled break/continue

---

## 2. Operators

### 2.1 Bitwise Operators

**Current Status:** Falls back to `Add` operator (incorrect)

**Location:** `src/hir/lowering.rs` around line 177, and `src/hir.rs` `BinaryOperator` enum

**What needs to be done:**

1. Add bitwise operators to the `BinaryOperator` enum in `src/hir.rs`:
```rust
pub enum BinaryOperator {
    // ... existing operators ...

    // Bitwise operators
    BitwiseAnd,      // &
    BitwiseOr,       // |
    BitwiseXor,      // ^
    LeftShift,       // <<
    RightShift,      // >>
    UnsignedRightShift, // >>>
}
```

2. Update the match in `lower_expression()`:
```rust
ast::BinaryOperator::BitwiseAnd => BinaryOperator::BitwiseAnd,
ast::BinaryOperator::BitwiseOR => BinaryOperator::BitwiseOr,
ast::BinaryOperator::BitwiseXOR => BinaryOperator::BitwiseXor,
ast::BinaryOperator::ShiftLeft => BinaryOperator::LeftShift,
ast::BinaryOperator::ShiftRight => BinaryOperator::RightShift,
ast::BinaryOperator::ShiftRightZeroFill => BinaryOperator::UnsignedRightShift,
```

3. Update `reactive_function.rs` to convert these to strings:
```rust
BinaryOperator::BitwiseAnd => "&",
BinaryOperator::BitwiseOr => "|",
BinaryOperator::BitwiseXor => "^",
BinaryOperator::LeftShift => "<<",
BinaryOperator::RightShift => ">>",
BinaryOperator::UnsignedRightShift => ">>>",
```

---

### 2.2 instanceof Operator

**Current Status:** Falls back to `Add` (incorrect)

**Location:** Same as bitwise operators

**What it should do:**
Check if an object is an instance of a constructor.

**Example:**
```javascript
function isArray(x) {
  return x instanceof Array;
}
```

**Implementation:**
1. Add to `BinaryOperator`:
```rust
InstanceOf,  // instanceof
```

2. Map in lowering:
```rust
ast::BinaryOperator::Instanceof => BinaryOperator::InstanceOf,
```

3. Convert to string:
```rust
BinaryOperator::InstanceOf => "instanceof",
```

---

### 2.3 in Operator

**Current Status:** Falls back to `Add` (incorrect)

**What it should do:**
Check if a property exists in an object.

**Example:**
```javascript
function hasName(obj) {
  return "name" in obj;
}
```

**Implementation:**
Same pattern as instanceof:
```rust
In,  // in

ast::BinaryOperator::In => BinaryOperator::In,

BinaryOperator::In => "in",
```

---

### 2.4 Unary Operators

**Current Status:** Partially implemented (only Not, Negate, TypeOf, IsNullish)

**Location:** `src/hir.rs` `UnaryOperator` enum, `src/hir/lowering.rs`

**Missing unary operators:**
- `+` (unary plus - converts to number)
- `~` (bitwise NOT)
- `void`
- `delete`

**What needs to be done:**

1. Add to `UnaryOperator` enum:
```rust
pub enum UnaryOperator {
    Not,        // !
    Negate,     // -
    Plus,       // + (unary)
    BitwiseNot, // ~
    TypeOf,     // typeof
    Void,       // void
    Delete,     // delete
    IsNullish,  // internal for ??
}
```

2. Handle `Expression::UnaryExpression` in lowering:
```rust
Expression::UnaryExpression(unary) => {
    let operand = self.lower_expression(&unary.argument);
    let op = match unary.operator {
        ast::UnaryOperator::LogicalNot => UnaryOperator::Not,
        ast::UnaryOperator::UnaryNegation => UnaryOperator::Negate,
        ast::UnaryOperator::UnaryPlus => UnaryOperator::Plus,
        ast::UnaryOperator::BitwiseNot => UnaryOperator::BitwiseNot,
        ast::UnaryOperator::Typeof => UnaryOperator::TypeOf,
        ast::UnaryOperator::Void => UnaryOperator::Void,
        ast::UnaryOperator::Delete => UnaryOperator::Delete,
    };
    self.push_instruction(InstructionValue::UnaryOp { op, operand })
}
```

3. Update codegen to handle new operators:
```rust
UnaryOperator::Plus => "+",
UnaryOperator::BitwiseNot => "~",
UnaryOperator::Void => "void ",
UnaryOperator::Delete => "delete ",
```

---

### 2.5 Update Expressions

**Current Status:** Not implemented

**Location:** `src/hir/lowering.rs` in `lower_expression()`

**What it should do:**
Handle `++` and `--` operators (both prefix and postfix).

**Examples:**
```javascript
let x = 5;
x++;      // postfix increment
++x;      // prefix increment
x--;      // postfix decrement
--x;      // prefix decrement
```

**Implementation hints:**
1. Prefix: increment/decrement first, then return new value
2. Postfix: save original value, increment/decrement, return original

**Code template:**
```rust
Expression::UpdateExpression(update) => {
    let arg_place = match &update.argument {
        ast::SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
            Place {
                identifier: Identifier {
                    name: id.name.to_string(),
                    id: 0,
                },
            }
        }
        _ => return self.create_temp(), // Complex targets not supported
    };

    // Load current value
    let current = self.push_instruction(InstructionValue::LoadLocal(arg_place.clone()));

    // Create 1 constant
    let one = self.push_instruction(InstructionValue::Constant(Constant::Int(1)));

    // Compute new value
    let op = if update.operator == ast::UpdateOperator::Increment {
        BinaryOperator::Add
    } else {
        BinaryOperator::Sub
    };
    let new_value = self.push_instruction(InstructionValue::BinaryOp {
        op,
        left: current.clone(),
        right: one,
    });

    // Store new value
    self.push_instruction(InstructionValue::StoreLocal(arg_place, new_value.clone()));

    // Return appropriate value
    if update.prefix {
        new_value  // ++x returns new value
    } else {
        current    // x++ returns old value
    }
}
```

---

## 3. Functions and Classes

### 3.1 Arrow Functions

**Current Status:** Not implemented (only function declarations work)

**Location:** `src/lib.rs` and `src/hir/lowering.rs`

**What it should do:**
Handle arrow function expressions like `(x) => x * 2`.

**Examples:**
```javascript
const double = (x) => x * 2;
const greet = name => `Hello, ${name}`;
const add = (a, b) => {
  return a + b;
};
```

**Implementation hints:**

1. In `src/lib.rs`, the main `compile()` function only handles `FunctionDeclaration`:
```rust
if let oxc_ast::ast::Statement::FunctionDeclaration(func) = stmt {
    // ...
}
```

2. Arrow functions appear as expressions, so you need to:
   - Handle `Expression::ArrowFunctionExpression` in `lower_expression()`
   - Create a nested `HIRFunction` for the arrow function
   - Generate code for it as a separate function or inline

3. Challenges:
   - Arrow functions capture `this` from enclosing scope
   - They can have expression bodies (implicit return) or block bodies
   - They need to be lowered to a function value

**Suggested approach:**
For now, create a placeholder that returns the arrow function as-is:
```rust
Expression::ArrowFunctionExpression(arrow) => {
    // For now, just create a temp representing the function
    // Full implementation would lower the function body
    self.create_temp()
}
```

---

### 3.2 Class Declarations

**Current Status:** Not implemented

**Location:** `src/lib.rs` and `src/hir/lowering.rs`

**What it should do:**
Handle ES6 class declarations.

**Example:**
```javascript
class Counter {
  constructor(initial) {
    this.count = initial;
  }

  increment() {
    this.count++;
  }
}
```

**Implementation hints:**
1. Classes are syntactic sugar for constructor functions + prototypes
2. Each method becomes a function on the prototype
3. The constructor is the main function
4. Static methods go on the class itself

**Complexity:** HIGH - significant work required

---

### 3.3 Method Definitions

**Current Status:** Not implemented

**Location:** Would be part of class/object handling

**What it should do:**
Handle shorthand method definitions in objects.

**Example:**
```javascript
const obj = {
  greet() {
    return "Hello";
  },
  async fetch() {
    // ...
  },
  *generator() {
    yield 1;
  }
};
```

---

### 3.4 Getters and Setters

**Current Status:** Not implemented

**What it should do:**
Handle getter and setter definitions.

**Example:**
```javascript
const obj = {
  get name() {
    return this._name;
  },
  set name(value) {
    this._name = value;
  }
};
```

---

## 4. Advanced Patterns

### 4.1 Nested Destructuring

**Current Status:** Partially implemented (only top-level)

**Location:** `src/hir/lowering.rs` in assignment target handling

**What's missing:**
Nested patterns like `const { a: { b } } = obj`

**Example that needs work:**
```javascript
function test() {
  const obj = { inner: { value: 42 } };
  const { inner: { value } } = obj;
  return value;
}
```

**Implementation hints:**
The current implementation handles:
- `{ a, b } = obj` ✓
- `[a, b] = arr` ✓

But not:
- `{ a: { b } } = obj` (nested object destructuring)
- `[a, [b, c]] = arr` (nested array destructuring)

To implement, you need recursive destructuring:
```rust
fn lower_destructuring_target(&mut self, target: &AssignmentTarget, source: Place) {
    match target {
        AssignmentTarget::ObjectAssignmentTarget(obj) => {
            for prop in &obj.properties {
                // Get property value
                let prop_value = self.push_instruction(InstructionValue::PropertyLoad {
                    object: source.clone(),
                    property: prop_name,
                });

                // Recursively handle the binding
                match &prop.binding {
                    AssignmentTargetMaybeDefault::AssignmentTargetIdentifier(id) => {
                        // Simple: just store
                    }
                    AssignmentTargetMaybeDefault::AssignmentTarget(nested) => {
                        // Recursive: destructure further
                        self.lower_destructuring_target(nested, prop_value);
                    }
                    // ...
                }
            }
        }
        // ... similar for arrays
    }
}
```

---

### 4.2 Default Values in Destructuring

**Current Status:** Not implemented

**What it should do:**
Handle default values in destructuring patterns.

**Example:**
```javascript
const { name = "Anonymous" } = user;
const [first = 0, second = 0] = numbers;
```

**Implementation hints:**
After extracting the value, check if it's undefined and use default:
```rust
// Pseudocode:
let value = obj.name;
let is_undefined = value === undefined;
if (is_undefined) {
    value = "Anonymous";
}
let name = value;
```

This requires:
1. Checking for undefined
2. Conditional assignment
3. Similar to nullish coalescing but for `undefined` only

---

### 4.3 Rest Patterns

**Current Status:** Partially implemented (spread in arrays/calls), not in destructuring

**What it should do:**
Handle rest elements in destructuring.

**Example:**
```javascript
const [first, ...rest] = arr;
const { a, ...others } = obj;
```

**Implementation hints:**
1. For arrays: use `Array.prototype.slice()` or index from position
2. For objects: need to exclude named properties (complex)

---

### 4.4 Optional Chaining

**Current Status:** Not implemented

**Location:** `src/hir/lowering.rs` - need to handle in expressions

**What it should do:**
Handle `?.` operator for safe property access.

**Example:**
```javascript
const name = user?.profile?.name;
const result = obj?.method?.();
```

**Expected transformation:**
```javascript
// user?.profile?.name becomes:
const t0 = user;
const t1 = (t0 == null) ? undefined : t0.profile;
const t2 = (t1 == null) ? undefined : t1.name;
```

**Implementation hints:**
1. Similar structure to nullish coalescing
2. Check for null/undefined before each property access
3. Short-circuit to `undefined` if any part is nullish

Look for these AST node types:
- `Expression::ChainExpression`
- Inside: `ChainElement::CallExpression` with `optional: true`
- Inside: `ChainElement::StaticMemberExpression` with `optional: true`

---

## 5. Expressions

### 5.1 Template Literals

**Current Status:** Not implemented

**Location:** `src/hir/lowering.rs` in `lower_expression()`

**What it should do:**
Handle template strings with interpolation.

**Example:**
```javascript
const greeting = `Hello, ${name}!`;
const multiline = `
  Line 1
  Line 2
`;
```

**Expected transformation:**
```javascript
// `Hello, ${name}!` becomes:
const t0 = "Hello, ";
const t1 = name;
const t2 = t0 + t1;
const t3 = "!";
const t4 = t2 + t3;
```

**Implementation:**
```rust
Expression::TemplateLiteral(template) => {
    // template.quasis = the string parts
    // template.expressions = the interpolated expressions

    // Start with first quasi
    let mut result = if let Some(first_quasi) = template.quasis.first() {
        self.push_instruction(InstructionValue::Constant(
            Constant::String(first_quasi.value.raw.to_string())
        ))
    } else {
        self.push_instruction(InstructionValue::Constant(Constant::String(String::new())))
    };

    // Alternate: expression, quasi, expression, quasi, ...
    for (i, expr) in template.expressions.iter().enumerate() {
        // Concatenate expression
        let expr_val = self.lower_expression(expr);
        result = self.push_instruction(InstructionValue::BinaryOp {
            op: BinaryOperator::Add,
            left: result,
            right: expr_val,
        });

        // Concatenate next quasi
        if let Some(quasi) = template.quasis.get(i + 1) {
            let quasi_val = self.push_instruction(InstructionValue::Constant(
                Constant::String(quasi.value.raw.to_string())
            ));
            result = self.push_instruction(InstructionValue::BinaryOp {
                op: BinaryOperator::Add,
                left: result,
                right: quasi_val,
            });
        }
    }

    result
}
```

---

### 5.2 Tagged Templates

**Current Status:** Not implemented

**What it should do:**
Handle tagged template literals like `html\`<div>${content}</div>\``.

**Example:**
```javascript
const result = tag`Hello ${name}`;
```

**Implementation:**
Tagged templates are function calls where:
- First argument: array of string parts
- Rest arguments: interpolated values

---

### 5.3 New Expressions

**Current Status:** Not implemented

**What it should do:**
Handle `new Constructor()` expressions.

**Example:**
```javascript
const date = new Date();
const obj = new MyClass(arg1, arg2);
```

**Implementation hints:**
1. Add a new `InstructionValue` variant:
```rust
New {
    constructor: Place,
    args: Vec<Argument>,
}
```

2. Handle in lowering:
```rust
Expression::NewExpression(new_expr) => {
    let constructor = self.lower_expression(&new_expr.callee);
    let args = new_expr.arguments.iter().map(|arg| {
        // ... same as call arguments
    }).collect();

    self.push_instruction(InstructionValue::New { constructor, args })
}
```

3. Update codegen:
```rust
ReactiveValue::New { constructor, args } => {
    format!("new {}({})", self.identifier_name(constructor), args_str.join(", "))
}
```

---

### 5.4 Sequence Expressions

**Current Status:** Not implemented

**What it should do:**
Handle comma-separated expressions where only the last value is returned.

**Example:**
```javascript
const x = (a++, b++, a + b);
```

**Implementation:**
```rust
Expression::SequenceExpression(seq) => {
    let mut last_value = self.create_temp();
    for expr in &seq.expressions {
        last_value = self.lower_expression(expr);
    }
    last_value
}
```

---

### 5.5 Conditional (Ternary) Expressions

**Current Status:** Not implemented

**What it should do:**
Handle `condition ? then : else` expressions.

**Example:**
```javascript
const max = a > b ? a : b;
```

**Implementation:**
Similar to logical expressions with short-circuit evaluation:
```rust
Expression::ConditionalExpression(cond) => {
    let test = self.lower_expression(&cond.test);

    let then_block = self.next_block_id();
    let else_block = self.next_block_id();
    let merge_block = self.next_block_id();

    let result_place = self.create_temp();

    self.terminate_block(Terminal::If {
        test,
        consequent: then_block,
        alternate: else_block,
    });

    // Then branch
    self.start_block(then_block);
    let then_val = self.lower_expression(&cond.consequent);
    self.push_instruction(InstructionValue::StoreLocal(result_place.clone(), then_val));
    self.terminate_block(Terminal::Goto(merge_block));

    // Else branch
    self.start_block(else_block);
    let else_val = self.lower_expression(&cond.alternate);
    self.push_instruction(InstructionValue::StoreLocal(result_place.clone(), else_val));
    self.terminate_block(Terminal::Goto(merge_block));

    // Merge
    self.start_block(merge_block);
    self.push_instruction(InstructionValue::LoadLocal(result_place))
}
```

---

### 5.6 Await Expressions

**Current Status:** Not implemented

**What it should do:**
Handle `await promise` in async functions.

**Example:**
```javascript
async function fetchData() {
  const response = await fetch(url);
  return response.json();
}
```

**Complexity:** HIGH - requires async function support

---

### 5.7 Yield Expressions

**Current Status:** Not implemented

**What it should do:**
Handle `yield` in generator functions.

**Example:**
```javascript
function* range(start, end) {
  for (let i = start; i < end; i++) {
    yield i;
  }
}
```

**Complexity:** HIGH - requires generator function support

---

## 6. Code Quality

### 6.1 Control Flow Reconstruction Issues

**Current Status:** The tree reconstruction from CFG has some issues

**Location:** `src/hir/reactive_function.rs`

**Problem:**
When converting CFG back to tree structure, some branches don't properly merge. For example, in the nullish coalescing output, the else branch doesn't have a proper continuation.

**Example of issue:**
```javascript
// Input:
const x = a ?? 'default';
return x;

// Current output (problematic):
if (t2) {
    // ... returns here
} else {
    const t1 = t0;
    // Missing: continuation to use t1
}
```

**What needs to be fixed:**
1. The tree builder needs to properly handle merge points
2. Phi nodes should be converted to assignments at merge blocks
3. The visited block tracking may be too aggressive

---

### 6.2 Unused Variable Elimination

**Current Status:** Generated code has many unused temporaries

**What it should do:**
Remove or inline variables that are only used once.

**Example:**
```javascript
// Current output:
const t0 = 1;
const t1 = 2;
const t2 = t0 + t1;
let sum_1 = t2;
const t4 = sum_1;
return t4;

// Could be simplified to:
const t0 = 1;
const t1 = 2;
const t2 = t0 + t1;
return t2;
```

**Implementation hints:**
1. Track use counts for each variable
2. Variables used exactly once can be inlined
3. Unused variables can be eliminated
4. This is a post-processing optimization pass

---

### 6.3 Better Error Messages

**Current Status:** Errors often panic or return generic messages

**Location:** Throughout the codebase

**What it should do:**
Provide helpful error messages with source locations.

**Implementation hints:**
1. Use `miette` (already a dependency) for fancy error reporting
2. Track source spans through lowering
3. Create specific error types for common issues

**Example improved errors:**
```
error: Unsupported syntax
  --> input.js:5:10
   |
 5 |   for await (const item of asyncItems) {
   |       ^^^^^
   |
   = help: async iteration is not yet supported
```

---

## Quick Reference: Files to Modify

| Feature Category | Primary File | Secondary Files |
|-----------------|--------------|-----------------|
| Control flow (for, switch, etc.) | `src/hir/lowering.rs` | `src/hir.rs` (if new terminals needed) |
| New operators | `src/hir.rs`, `src/hir/lowering.rs` | `src/hir/reactive_function.rs`, `src/codegen.rs` |
| New expressions | `src/hir/lowering.rs` | `src/hir.rs` (if new instructions), codegen |
| Functions/classes | `src/lib.rs`, `src/hir/lowering.rs` | Many files |
| Code quality | `src/hir/reactive_function.rs`, `src/codegen.rs` | - |

---

## Testing Your Changes

After implementing a feature:

1. **Run unit tests:**
   ```bash
   cargo test
   ```

2. **Update snapshots if needed:**
   ```bash
   cargo insta accept
   ```

3. **Build the npm module:**
   ```bash
   cargo build --release --features napi --lib
   cp target/release/libreact_compiler_rust.so npm/react-compiler-rust.node
   ```

4. **Test via npm:**
   ```bash
   cd npm && node test.js
   ```

5. **Test your specific feature:**
   ```javascript
   const { compile } = require('./index.js');
   const result = compile(`
     function test() {
       // Your test code here
     }
   `);
   console.log(result.code);
   ```

---

## Priority Recommendations

### High Priority (Most Common in React Code)
1. Template literals (very common in React)
2. Conditional expressions (ternary)
3. Update expressions (++/--)
4. Optional chaining (?.)
5. For loops

### Medium Priority
1. Arrow functions
2. Bitwise/in/instanceof operators
3. Switch statements
4. Unary operators (complete set)

### Lower Priority (Less Common)
1. Classes
2. Generators
3. Async/await
4. For-in/for-of loops

---

*Last updated: December 2024*
