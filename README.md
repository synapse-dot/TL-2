# TL/2 — Temporal Language for Self-Modifying Systems

TL/2 is a systems language where **time is a first-class dimension**, code can safely rewrite its own future, and processes form a persistent, evolving system image.

## Status

This repository currently contains the TL/2 v1.0 specification and recommended implementation plan.

- Full spec: [`docs/spec/TL2_SPEC.md`](docs/spec/TL2_SPEC.md)
- Suggested build order (MVP): grammar/parser → temporal state engine → function versioning/rewrite scheduling → journal/replay → process runtime → dock/commit → contracts/capabilities → `dash` integration.

## File extension and runtime

- Source extension: `.tlh`
- Environment/runtime manager: `dash`

## Next step

Implement the runtime in the build order described in the specification, starting with parser and temporal interval storage semantics.
