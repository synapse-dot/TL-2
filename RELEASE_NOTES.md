# v0.1.0 — Parser & Core Temporal State Engine

This is the initial MVP release of TL/2 featuring the first two phases of the implementation roadmap.

## Phase 1: Grammar & Parser
- Complete lexical scanner for TL/2 syntax
- Recursive descent parser generating full ASTs
- Support for all core language keywords

## Phase 2: Temporal State Engine
- Time-indexed variable storage with interval semantics
- Conflict detection
- Query interface for state lookup

## Build
- Build with: `cargo build --release`

## Tests
- Run tests with: `cargo test`