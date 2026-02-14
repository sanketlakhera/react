# React Compiler Rust - Remaining Work

This document tracks implementation progress and remaining features. Items marked ✅ are complete, ⚠️ are partial, and ❌ are not yet started.

*Last updated: February 2026*

---

## Status Summary

| Category | Done | Partial | Todo | Total |
|----------|------|---------|------|-------|
| Control Flow | 4 | 0 | 3 | 7 |
| Operators | 4 | 0 | 0 | 4 |
| Functions & Classes | 0 | 0 | 4 | 4 |
| Advanced Patterns | 0 | 2 | 2 | 4 |
| Expressions | 2 | 0 | 5 | 7 |
| Code Quality | 0 | 1 | 2 | 3 |
| **Total** | **10** | **3** | **16** | **29** |

---

## 1. Control Flow Statements

### 1.1 ✅ For Loops — DONE

Implemented in `lowering.rs`. Creates proper CFG with init → header → body → update → back-edge structure. Tested with `sprout_for_loop_basic`.

---

### 1.2 ❌ For-In Loops

**Status:** Not implemented

**What it should do:** Transform `for (key in object)` loops.

```javascript
function logKeys(obj) {
  for (const key in obj) {
    console.log(key);
  }
}
```

**Implementation hints:**
- Needs new HIR instructions `ForInInit` and `ForInNext` for iterator protocol
- Simpler approach: treat as unsupported and bail out

---

### 1.3 ❌ For-Of Loops

**Status:** Not implemented

**What it should do:** Transform `for (item of iterable)` loops.

```javascript
function sumArray(arr) {
  let total = 0;
  for (const num of arr) {
    total += num;
  }
  return total;
}
```

---

### 1.4 ✅ Switch Statements — DONE

Implemented with `Terminal::Switch` in HIR, proper case/default lowering, fallthrough support, and break handling. Also handles `continue` inside switch-inside-for-loop correctly. Tested with `sprout_switch` and `sprout_simple_switch`.

---

### 1.5 ❌ Try-Catch-Finally

**Status:** Not implemented (try block body is lowered, catch/finally ignored)

**What it should do:** Handle exception handling blocks.

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

**Complexity:** HIGH — requires new terminal types (`TryStart`, `Throw`) and exception handler tracking in the CFG.

---

### 1.6 ✅ Break and Continue — DONE

Implemented with a `loop_stack` in `LoweringContext` that tracks break targets and continue targets. Works for while, for, and switch statements (including `continue` inside switch-inside-for-loop).

---

### 1.7 ✅ While / Do-While Loops — DONE

While loops fully implemented with proper header → body → back-edge CFG structure and correct codegen reconstruction. Tested with `sprout_while_loop`.

---

## 2. Operators

### 2.1 ✅ Bitwise Operators — DONE

All bitwise operators added to `BinaryOperator` enum and properly mapped in lowering and codegen:
`&`, `|`, `^`, `<<`, `>>`, `>>>`. Tested with `sprout_operators_comprehensive`.

---

### 2.2 ✅ instanceof / in Operators — DONE

Both `instanceof` and `in` added to `BinaryOperator` enum and mapped correctly in lowering and codegen.

---

### 2.3 ✅ Unary Operators — DONE (complete set)

All unary operators implemented: `!`, `-`, `+`, `~`, `typeof`, `void`, `delete`, plus internal `IsNullish` for `??` support.

---

### 2.4 ✅ Update Expressions — DONE

`++` and `--` (both prefix and postfix) implemented with correct semantics. Tested with `sprout_update_expressions`.

---

## 3. Functions and Classes

### 3.1 ❌ Arrow Functions

**Status:** Not implemented — returns empty temp

**What it should do:** Handle `(x) => x * 2` and `(a, b) => { return a + b; }`.

**Implementation hints:**
- Handle `Expression::ArrowFunctionExpression` in `lower_expression()`
- Create nested `HIRFunction` for the arrow body
- Expression body → implicit return; block body → normal lowering
- Arrow functions capture `this` lexically

**Priority:** HIGH — most React components use arrow functions

---

### 3.2 ❌ Class Declarations

**Status:** Not implemented

**Complexity:** HIGH — classes are syntactic sugar for constructor + prototype methods.

---

### 3.3 ❌ Method Definitions

**Status:** Not implemented — needed for object shorthand methods `{ greet() {} }`.

---

### 3.4 ❌ Getters and Setters

**Status:** Not implemented — `get`/`set` property definitions.

---

## 4. Advanced Patterns

### 4.1 ⚠️ Destructuring — PARTIAL

**Done:** Top-level object `{ a, b } = obj` and array `[a, b] = arr` destructuring in assignments.

**Missing:**
- Nested destructuring: `{ a: { b } } = obj`
- Nested array: `[a, [b, c]] = arr`
- Default values: `{ name = "Anonymous" } = user`
- Computed keys in destructuring

**Implementation:** Requires recursive `lower_destructuring_target` function.

---

### 4.2 ❌ Rest Patterns

**Status:** Spread in arrays/calls works, but rest in destructuring (`const [first, ...rest] = arr`) is not implemented.

---

### 4.3 ❌ Optional Chaining

**Status:** Not implemented — `?.` stripped through to inner expression

```javascript
const name = user?.profile?.name;
const result = obj?.method?.();
```

**Implementation:** Similar to nullish coalescing — check for null/undefined before each access, short-circuit to `undefined`.

**Priority:** HIGH — ubiquitous in modern React code

---

### 4.4 ⚠️ Spread — PARTIAL

**Done:** Spread in function call arguments and array literals.
**Missing:** Spread in object literals (partially done via `ObjectProperty::Spread`).

---

## 5. Expressions

### 5.1 ✅ Template Literals — DONE

Implemented by lowering quasis and expressions into `BinaryOp::Add` concatenation chains. Uses `cooked` value for proper escape handling. Also fixed string escaping in `codegen.rs` to handle `\n`, `\r`, `\t`, `\0`. Tested with `sprout_template_literals` (6 cases: simple, multi-expression, plain, empty, nested ternary, escape sequences).

---

### 5.2 ❌ Tagged Templates

**Status:** Not implemented

```javascript
const result = tag`Hello ${name}`;
```

Lower as a function call: `tag(["Hello ", ""], name)`.

---

### 5.3 ❌ New Expressions

**Status:** Not implemented — returns empty temp

```javascript
const date = new Date();
```

**Implementation:** Add `InstructionValue::New { constructor, args }` and corresponding `ReactiveValue::New` + codegen.

---

### 5.4 ❌ Sequence Expressions

**Status:** Not implemented (partially handled in `ForStatementInit`)

```javascript
const x = (a++, b++, a + b);
```

Simple: evaluate all expressions, return the last value.

---

### 5.5 ✅ Conditional (Ternary) Expressions — DONE

Implemented using same CFG pattern as logical expressions: `If` terminal → two branches storing to shared `result_place` → merge block. Nested ternaries work automatically. Tested with `sprout_conditionals`.

---

### 5.6 ❌ Await Expressions

**Status:** Not implemented

**Complexity:** HIGH — requires async function support

---

### 5.7 ❌ Yield Expressions

**Status:** Not implemented

**Complexity:** HIGH — requires generator function support

---

## 6. Code Quality

### 6.1 ⚠️ Control Flow Reconstruction — IMPROVED

**Location:** `src/hir/reactive_function.rs`

Recent improvements:
- ✅ While loops now reconstruct as proper `while(true) { if (!cond) break; body; }` 
- ✅ For loops reconstruct with init, test, body, update
- ✅ Switch handles fallthrough, break, and nested continue correctly
- ✅ Phi nodes resolved at merge points and loop back-edges

**Remaining issues:**
- Merge block phi resolution can be over-eager in some edge cases
- No support for labeled break/continue

---

### 6.2 ❌ Unused Variable Elimination

**Status:** Not implemented — generated code has many redundant temporaries

```javascript
// Current output:
const t0 = 1;
const t1 = 2;
const t2 = t0 + t1;
let sum_1 = t2;
return sum_1;

// Could be simplified to:
return 1 + 2;
```

**Implementation:** Track use counts, inline single-use variables, eliminate dead ones. This is a post-processing optimization pass.

---

### 6.3 ❌ Better Error Messages

**Status:** Errors often panic or return generic messages. Should use `miette` for fancy diagnostics with source locations.

---

## Quick Reference

| Feature | Primary File | Status |
|---------|-------------|--------|
| For loops | `lowering.rs` | ✅ |
| For-in/of | `lowering.rs`, `hir.rs` | ❌ |
| Switch | `lowering.rs`, `hir.rs` | ✅ |
| Try-catch | `lowering.rs`, `hir.rs` | ❌ |
| Break/Continue | `lowering.rs` | ✅ |
| All operators | `hir.rs`, `lowering.rs`, `reactive_function.rs` | ✅ |
| Arrow functions | `lib.rs`, `lowering.rs` | ❌ |
| Destructuring (nested) | `lowering.rs` | ❌ |
| Optional chaining | `lowering.rs` | ❌ |
| Template literals | `lowering.rs`, `codegen.rs` | ✅ |
| Ternary | `lowering.rs` | ✅ |
| New expressions | `lowering.rs`, `hir.rs`, `codegen.rs` | ❌ |
| Unused var elimination | `codegen.rs` | ❌ |

---

## Priority Order (Next Steps)

### 🔴 High Priority — needed to process real React code
1. ~~**Template literals**~~ ✅
2. **Arrow functions** — most React components use these
3. **Optional chaining** — ubiquitous in modern React
4. **New expressions** — `new Date()`, `new Map()`, etc.
5. **JSX support** — the compiler's core purpose (see PARITY.md)

### 🟡 Medium Priority — correctness improvements
6. **Nested destructuring + defaults**
7. **For-in / For-of loops**
8. **Sequence expressions**
9. **Tagged templates**
10. **Rest patterns in destructuring**

### 🟢 Lower Priority — advanced / rare
11. **Try-catch-finally**
12. **Class declarations**
13. **Getters/setters**
14. **Async/await**
15. **Generators/yield**

### 🔵 Code Quality
16. **Unused variable elimination**
17. **Better error messages with source locations**

---

## Testing

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --test sprout_test      # Runtime verification
cargo test --test patterns_test    # Pattern compilation
cargo test --test fixtures_test    # Snapshot tests

# Update snapshots after changes
cargo insta accept

# Test via CLI
cargo run --bin react-compiler-rust -- --input test.js
```
