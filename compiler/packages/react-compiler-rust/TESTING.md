# React Compiler Testing Guide

## Two Testing Approaches

### Approach 1: Pattern-Based Testing
Create focused fixtures covering common React patterns.

| Pattern | What It Tests |
|---------|---------------|
| Hooks | `useState`, `useEffect`, `useMemo`, `useCallback` |
| Props | Destructuring, spread, children |
| Conditionals | Ternary, short-circuit, if/else in JSX |
| Lists | `.map()`, `.filter()`, keys |
| Events | onClick, onChange, form handling |

### Approach 2: Official React Fixtures
Test against the 3657+ official fixtures in `fixtures-react/compiler/`.

**Fixture Format:**
```
fixtures-react/compiler/
├── alias-capture-in-method-receiver.js      # Input
├── alias-capture-in-method-receiver.expect.md  # Expected output
├── error.invalid-mutate-props.js            # Error cases
├── error.invalid-mutate-props.expect.md
└── ...
```

---

## Running Tests

### Pattern Tests
```bash
cd tests/e2e && node runner.js fixtures/counter.js
```

### Batch Fixture Testing
```bash
cargo test --test fixtures_test
```

### React Compiler Fixtures
```bash
# Run the batch tester
node tests/e2e/batch-runner.js tests/e2e/fixtures-react/compiler
```

---

## Fixture Categories

| Category | Pattern | Count |
|----------|---------|-------|
| **Normal** | `*.js`, `*.ts`, `*.tsx` | ~1800 |
| **Errors** | `error.*.js` | ~300 |
| **Flow** | `*.flow.js` | ~50 |
| **Subdirs** | `fbt/`, `inner-function/`, etc. | ~500 |
