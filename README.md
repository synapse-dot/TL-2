
# TL/2 — Temporal Language for Self-Modifying Systems

![Status](https://img.shields.io/badge/status-specification-blue)
![Version](https://img.shields.io/badge/version-1.0.0--draft-lightgrey)
![File Extension](https://img.shields.io/badge/file-.tlh-333)
![Runtime](https://img.shields.io/badge/runtime-dash-333)

**TL/2** is a systems programming language where **time is a first-class dimension** and **code can safely rewrite its own future**. A TL/2 program is not a fixed sequence of instructions but an evolving entity spread across versioned timelines, designed for long-running autonomous agents, fault-tolerant infrastructure, and deterministic replay.

---

## Core Concepts

| Primitive | Description |
|-----------|-------------|
| **Time‑indexed state** | Every variable has a history; values are declared over intervals (`from … to …`, `until`). |
| **Versioned functions** | Function bodies can be scheduled for future instants (`at … rewrite …`) or replaced immediately (`rewrite … with …`). |
| **Transactional `at` blocks** | `at 500ms { … }` executes as an atomic transaction at a precise logical instant. |
| **Speculative `dock` scopes** | `dock trial { … }` forks an isolated timeline; `commit` merges winning mutations back. |
| **Lightweight processes (PIDs)** | Actor‑based concurrency with message passing, capability grants, and supervision. |
| **Deterministic replay** | An append‑only journal with total‑order keys guarantees bit‑identical replay of any execution. |
| **Contracts** | Pre‑/post‑conditions enforced before any `rewrite` is accepted. |
| **Hot code swapping** | JIT‑compiled native code is patched atomically – no downtime. |

---

## Quick Example

```tlh
state mode = "explore"

fn policy(sensor) -> action {
  if sensor.danger > 0.7 { yield action.evade }
  yield action.explore
}

process worker(stream) {
  observe stream as source
  loop {
    receive sensor_data -> {
      let act = policy(sensor_data)
      send act to executor
    }
  }
}

// Self-improvement loop: every 60 s, try a mutated policy in isolation
at 60s {
  dock trial {
    rewrite policy => evolve(policy)
    simulate(window = 60s)
    yield policy if score > baseline
  }
  commit trial.policy if trial.score > baseline
}
```

---

## Tooling & Environment Manager

TL/2 programs live inside persistent **environments** managed by the `dash` runtime.

```bash
dash create-env my_system        # create a new environment
dash my_system -load agent.tlh   # load source into the environment
dash my_system -load rules.tlh
dash my_system                   # interactive REPL
dash my_system -quit             # save snapshot & exit
dash delete-env my_system        # destroy an environment
```

All source files carry the `.tlh` extension.

---

## Specifications (Normative)

- **[Grammar](./docs/spec/grammar.ebnf)** – disambiguated command/temporal syntax in EBNF.
- **[Temporal Conflict Resolution](./docs/spec/temporal_conflicts.md)** – overlap policies, event ordering, transaction boundaries, and deterministic replay guarantees.
- **[Conflict Test Cases](./docs/spec/conflict_cases.tlh)** – conformance suite for runtime implementors.

---

## Getting Started

> TL/2 is currently a **research‑grade specification**; a prototype interpreter is under active development.

You can explore the grammar with any EBNF parser or study the conflict test cases to understand the intended runtime behaviour. Contributions to the specification or to a reference implementation are welcome.

---

## Acknowledgements

TL/2 was designed and refined with the assistance of **DeepSeek AI** and **OpenAI Codex**, as part of a human‑driven, machine‑augmented research process. The language spec, grammar, and conflict resolution model were iterated collaboratively, combining formal rigour with AI‑accelerated exploration.

---

## License

This specification is released under the [Creative Commons Attribution 4.0 International License](https://creativecommons.org/licenses/by/4.0/).

---

*TL/2 treats time not as a side effect, but as the fundamental dimension along which all computation is expressed — and it gives code the power to evolve within that timeline, safely and audibly.*
```
