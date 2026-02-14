# React Compiler Rewrite Plan (Rust Edition)

This document outlines the end-to-end plan for rewriting the React Compiler (formerly React Forget) in **Rust**. 

## 1. Executive Summary

**Goal:** Create a high-performance, memory-safe compiler that automatically memoizes React components, eliminating the need for manual `useMemo` and `useCallback`.

**Why Rust?**
*   **Performance:** Native compilation speed is crucial for developer experience.
*   **Ecosystem:** The Rust JavaScript ecosystem is mature. Tools like [SWC](https://swc.rs/) and [Oxc](https://github.com/oxc-project/oxc) provide robust AST parsers and code generation capabilities, allowing us to focus specifically on the *compiler logic* rather than infrastructure.
*   **Safety:** Rust's ownership model fits well with graph-based IR manipulations and complex transformations.

## 2. Architecture Overview

The compiler pipeline will mirror the existing TypeScript implementation but leverage Rust's type system for stricter guarantees.

```mermaid
graph TD
    A[Source Code (JS/TS)] -->|Parser (Oxc/SWC)| B[AST]
    B -->|Lowering| C[HIR (Control Flow Graph)]
    C -->|SSA Transformation| D[HIR (SSA Form)]
    D -->|Analysis Phases| E[Annotated HIR]
    E -->|Scope Reconstruction| F[ReactiveFunction (Tree)]
    F -->|Codegen| G[Optimized AST]
    G -->|Printer| H[Output Code]
```

### Key Intermediate Representations
1.  **HIR (High-level IR):** A Control Flow Graph (CFG) where functions are broken into `BasicBlock`s containing flat `Instruction`s.
2.  **ReactiveFunction:** A tree-structured representation used in the final stages, grouping instructions into `ReactiveScope`s (memoization blocks).

## 3. Implementation Phases

### Phase 0: Setup & Infrastructure
**Goal:** Initialize the project and integrate a JS parser.

*   **Action Items:**
    *   Initialize a new Rust project: `cargo new react-compiler-rs`.
    *   Select a parser: **Oxc** is currently the fastest and most compliant, but **SWC** is also a strong contender. *Recommendation: Oxc*.
    *   Set up the test harness early. The existing `fixtures` from the TS repo are invaluable.
    *   **Implement "Sprout" equivalent:** Create a test runner that can not only compare output snapshots (using `insta`) but also execute the resulting JS (via Node.js subprocesses) to verify semantic correctness, mirroring the existing `sprout` implementation.

### Phase 1: Core Data Structures (HIR)
**Goal:** Define the internal language of the compiler.

*   **Action Items:**
    *   Define `Instruction`: `let x = <opcode> <operands>`.
    *   Define `BasicBlock`: List of instructions + Terminal (goto, return, if).
    *   Define `HIRFunction`: The container for blocks.
    *   Implement pretty-printing (Debug trait) for HIR to aid debugging.

### Phase 2: Lowering (AST -> HIR)
**Goal:** Convert tree-based AST into graph-based HIR.

*   **Action Items:**
    *   Implement `LoweringContext` to manage temporary variable generation.
    *   Traverse the AST (Oxc/SWC AST) and emit HIR instructions.
    *   Handle control flow (if/else, loops) by creating new BasicBlocks and linking predecessors/successors.
    *   *Challenge:* Handling closures and hoisting correctly.

### Phase 3: Analysis & Inference
**Goal:** Understand data flow and mutability.

*   **Action Items:**
    *   **SSA (Static Single Assignment):** Convert HIR to SSA form. This simplifies tracking where values come from.
        *   Implement Φ (Phi) nodes for control flow merges.
    *   **Type Inference:** Infer basic types (Primitive, Object, Function) to know what *can* be optimized.
    *   **Effect Analysis:** Determine side effects. A function that mutates global state behaves differently than a pure calculation.
    *   **Mutability Analysis:** The "borrow checker" of the React Compiler. Track which values are mutated and extend their "live ranges".

### Phase 4: Reactive Scope Construction
**Goal:** The heart of the compiler—grouping instructions into memoizable units.

*   **Action Items:**
    *   **Align Scopes:** identifying safe boundaries to insert `useMemo`.
    *   **Merge Scopes:** If two scopes overlap or have entangled dependencies, merge them.
    *   **Propagate Dependencies:** Calculate inputs (dependencies) for each scope.

### Phase 5: Reactive Function & Codegen
**Goal:** Convert the graph back to a tree and output JS.

*   **Action Items:**
    *   **Reconstruct Tree:** Convert the CFG back into a block-structured tree (`ReactiveFunction`). This involves detecting loop structures and if/else blocks from the graph.
    *   **Codegen:** Traverse the `ReactiveFunction`.
        *   For normal instructions: Emit equivalent JS AST.
        *   For `ReactiveScope`: Emit a `useMemoCache` read/write pattern (or `useMemo` for compatibility).

### Phase 6: Distribution & Integration
**Goal:** Deliver the compiler to users in their build pipelines (Next.js, Vite, Rolldown).

*   **Strategy A: NAPI-RS (Broad Compatibility):**
    *   **Tool:** [napi-rs](https://github.com/napi-rs/napi-rs).
    *   **Output:** A standard npm package with native binary bindings.
    *   **Usage:** Allows the compiler to be called from a JS context. This makes it instantly compatible with **Vite**, **Webpack**, and **Rollup** via standard plugin adapters.
*   **Strategy B: SWC Plugin (Next.js):**
    *   **Tool:** `swc_plugin_macro`.
    *   **Output:** A `.wasm` binary.
    *   **Usage:** Plugs directly into Next.js and Turbopack, running during the SWC transform phase.
*   **Strategy C: Rolldown Native (Ultimate Performance):**
    *   **Context:** **Rolldown** is a fast bundler written in Rust, built on top of **Oxc**.
    *   **Synergy:** Since this rewrite plans to use **Oxc** (Phase 0), it aligns perfectly with Rolldown.
    *   **Integration:** You can implement the compiler as a *native Rust plugin* (or internal pass) for Rolldown. This allows the compiler to operate directly on the shared Oxc AST (Abstract Syntax Tree), completely avoiding the overhead of parsing, serialization, and data transfer between JS and Rust.

## 4. Dependencies Strategy

| Component | Rust Crate Recommendation |
| :--- | :--- |
| **Parser / AST** | `oxc_parser`, `oxc_ast` (or `swc_core`) |
| **Bitsets / Vectors** | `bitvec` (useful for dataflow analysis) |
| **Graph Algorithms** | `petgraph` (or custom implementation for specific CFG needs) |
| **Error Reporting** | `miette` (beautiful error diagnostics) |
| **Testing** | `insta` (snapshot testing - crucial for compiler work) |

## 5. Existing Testing Infrastructure (Critical)

The existing compiler relies on a sophisticated testing framework that must be replicated or bridged to ensure 100% compatibility.

### A. Snap (The Test Runner)
Located in `compiler/packages/snap`, this is a custom runner that manages thousands of fixtures.
*   **Role:** Parallel execution, snapshot management, worker orchestration, and reporting.
*   **Rust Equivalent:** `cargo test` combined with a library like `insta` for snapshots. You may need a custom `harness` (e.g., `libtest-mimic`) to dynamically generate tests from the fixture directories.

### B. Sprout (Runtime Verification)
Located in `compiler/packages/snap/src/sprout`.
*   **Goal:** Verifies semantic equivalence. It executes *both* the original code and the compiled code, asserting that:
    1.  Return values are identical.
    2.  Console logs match.
    3.  No unexpected errors occur.
*   **Mechanism:** Fixtures export a `FIXTURE_ENTRYPOINT` object containing the function to test and its inputs.
*   **Porting Strategy:** Your Rust test runner needs to shell out to Node.js (or embed a JS runtime like V8/QuickJS/Deno) to execute the generated JavaScript and verify behavior, not just string snapshots.

### C. E2E Tests
Located in `compiler/packages/babel-plugin-react-compiler/src/__tests__/e2e`.
*   These test full React components in a simulated DOM environment, checking interactions (e.g. state updates after button clicks).

## 6. First Steps for You

1.  **Clone the existing repo** (you are here).
2.  **Create the Rust project** adjacent to `compiler/`.
3.  **Port the fixtures:** Write a script to verify your Rust compiler against the existing thousands of test cases in `compiler/fixtures`. Start with simple snapshots, then integrate runtime execution (Sprout).

## 7. Resources

*   **Existing Code:** `compiler/packages/babel-plugin-react-compiler/src` is your source of truth.
*   **Test Runner (Snap):** `compiler/packages/snap`
*   **Runtime Evaluator (Sprout):** `compiler/packages/snap/src/sprout`
*   **HIR Definition:** `src/HIR/HIR.ts`.
*   **Lowering Logic:** `src/HIR/BuildHIR.ts`.
