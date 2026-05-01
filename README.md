# TL/2 — Temporal Language for Self-Modifying Systems

TL/2 is a systems language where **time is a first-class dimension**, code can safely rewrite its own future, and processes form a persistent, evolving system image.

## Status: v0.1.0 — Parser & Core Temporal State Engine (MVP Phase)

This version includes the foundation of TL/2 with the first two phases of the implementation roadmap complete:

✅ **Grammar and Parser** - Lexical scanner and full AST parser for TL/2 syntax  
✅ **Temporal State Engine** - Interval-based state storage with conflict detection and query semantics

The full specification and implementation plan are available in [`docs/spec/TL2_SPEC.md`](docs/spec/TL2_SPEC.md).

## Features Implemented

### 1. Lexical Analysis & Parser
- Full tokenization of TL/2 keywords, identifiers, and time expressions
- Recursive descent parser generating Abstract Syntax Trees
- Support for all core language constructs: `state`, `at`, `fn`, `rewrite`, `morph`, `dock`, `spawn`, etc.

### 2. Temporal State Engine
- Time-indexed variable storage with half-open intervals `[start, end)`
- Conflict detection for overlapping state assertions
- Query interface for state values at specific time points or across intervals
- History traversal for timeline inspection

### 3. Core Language Features (Parsed)
- **State declarations**: `state <name> == <value> from t=<start> to t=<end>`
- **Instantaneous changes**: `at t=<instant>: <name> becomes <value>`
- **Function definitions**: `fn <name>(<params>) -> <return-type> from t=<start>: <body>`
- **Self-modification primitives**: `rewrite`, `morph`
- **Process model**: `spawn`, `send`, `receive`
- **Speculative execution**: `dock`, `commit`, `yield`
- **Safety contracts**: `pre`, `post` conditions with `old` references

## Build Instructions

### Prerequisites
- Rust 1.70+
- Cargo (bundled with Rust)

### Build
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

### Development Build
```bash
cargo build
```

## Example Output

### Basic Parser Test
```
$ cargo run --example parser
Input: fn counter() -> i64 from t=now:
         state count == 0 until t=+60s
         at t=+1s: count becomes 1

Parse tree:
├─ FunctionDef
│  ├─ name: counter
│  ├─ params: []
│  ├─ return_type: i64
│  └─ from: now
│     ├─ StateDecl
│     │  ├─ name: count
│     │  ├─ value: 0
│     │  └─ interval: [now, +60s)
│     └─ AtTransition
│        ├─ instant: +1s
│        ├─ name: count
│        └─ value: 1
```

### Temporal Query Test
```
$ cargo run --example temporal_state
Creating timeline for variable 'balance'
  Set balance = 100 from t=0 to t=100
  Set balance = 200 from t=100 to t=200

Queries:
  balance @ t=50:  100 ✓
  balance @ t=150: 200 ✓
  balance @ t=250: <undefined> (query beyond defined intervals)

Conflict detection (overlapping intervals):
  Attempt: balance = 150 from t=50 to t=150
  Result: ConflictError - overlaps with existing interval [0, 100)
```

## File Extension and Runtime

- **Source extension:** `.tlh`
- **Runtime manager:** `dash` (planned for future phase)

## Next Steps

The following components are planned for upcoming releases:

1. **Phase 3** (v0.2.0): Function versioning & rewrite scheduling
2. **Phase 4** (v0.3.0): Journal & deterministic replay engine
3. **Phase 5** (v0.4.0): Process runtime with mailbox implementation
4. **Phase 6** (v0.5.0): Dock/commit mechanism for speculative execution
5. **Phase 7** (v0.6.0): Contract system & capability enforcement
6. **Phase 8** (v1.0.0): `dash` CLI integration and full runtime

## Documentation

- **Language Specification:** [`docs/spec/TL2_SPEC.md`](docs/spec/TL2_SPEC.md)
- **Implementation Roadmap:** See "Next Steps" section above
- **Examples:** Check `/examples` directory for runnable code samples

## Contributing

TL/2 is under active development. Please refer to GitHub Issues for known limitations and open tasks.

## License

See LICENSE file for details.