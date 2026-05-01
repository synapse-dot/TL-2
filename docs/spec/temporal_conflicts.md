# Temporal Conflict Resolution (Normative)

This section is **normative**. Implementations MUST follow these rules to ensure portable behavior across runtimes, replayers, and distributed executors.

## 1) Overlap Resolution for `state` Intervals

An overlap occurs when two or more interval-valued writes target the same `state` path and their effective logical-time ranges intersect.

### 1.1 Conflict policy

Implementations MUST expose and document one of the following policies; the policy in force MUST be stable for a given program execution:

1. **`error` (recommended default):**
   - Any overlapping write to the same `state` path MUST fail the enclosing transaction.
   - No partial visibility is allowed from that transaction.

2. **`priority`:**
   - Overlap is resolved by an explicit numeric or ordinal priority attached to each write source.
   - Higher priority wins for each conflicting instant in the overlap.
   - Equal-priority overlaps MUST raise a deterministic conflict error unless a documented tie-breaker is configured.

3. **`last-writer-wins` (LWW):**
   - Overlap is resolved by total ordering keys (see §5.2).
   - The write with the greater ordering tuple wins for each conflicting instant.

### 1.2 Required ordering source (for `priority` and LWW)

If policy is `priority` or LWW, the implementation MUST define a deterministic ordering source and expose it in execution metadata. The ordering source MUST be one of:

- A monotonic journal append order; or
- A deterministic tuple derived from journal keys in §5.2.

Using wall-clock arrival time, thread scheduling order, or hash-map iteration order as an ordering source is NOT permitted.

## 2) `rewrite` Semantics for State and Other Targets

`rewrite <target> => <expr>` is the normative mutation form for versioned entities.

1. For `state` paths, `rewrite` updates the value at the current logical time.
2. On successful commit, the rewritten value becomes effective from that timestamp forward until superseded by a later write.
3. On abort/rollback, rewritten values are discarded as if never applied.
4. If a runtime supports `rewrite` of function bodies or other versioned targets, it MUST apply the same transaction and ordering semantics defined in this document.

## 3) Same-Timestamp Event Ordering

When multiple operations share the same logical timestamp, they MUST execute in the following phase order:

1. `at` (time activation / scope entry)
2. `rewrite`
3. `send`
4. `commit`

### 3.1 Intra-phase ordering

Within the same phase at the same timestamp, operations MUST execute by ascending journal order key (§5.2). If two operations are otherwise tied, the implementation MUST break ties using a deterministic statement index captured at parse/compile time.

### 3.2 Visibility rules

- `rewrite` effects in a timestamp are visible to subsequent `send` and `commit` in that same timestamp.
- `send` side effects are not visible as committed durable state until `commit` phase succeeds.
- If `commit` fails, all same-timestamp effects in the enclosing transaction MUST be rolled back.

### 3.3 `commit` payload semantics

`commit` MAY include an expression payload (for example, `commit "label"`).

- The payload is treated as commit metadata for diagnostics, journaling, and audit.
- The payload MUST NOT alter ordering semantics unless an implementation explicitly documents a deterministic mapping to ordering keys.
- If an implementation uses commit payloads for deduplication or recovery correlation, that behavior MUST be documented as part of journal format guarantees.

## 4) Transaction Boundaries for Multi-Statement Blocks at Same Logical Time

For a multi-statement block scheduled at one logical timestamp (e.g., `at 10ms { ... }`):

1. The whole block MUST be treated as a single atomic transaction by default.
2. Statement execution order inside the block MUST follow source order unless overridden by explicit semantics in §3.
3. Failures in any statement MUST abort and roll back the entire block.
4. Nested same-time blocks inherit the parent transaction unless the language/runtime defines an explicit `transaction` escape hatch; absent such syntax, split commits are forbidden.
5. Observers external to the transaction MUST see either pre-state or post-state, never an intermediate state.

### 4.1 Writer identity and optional priority

To support deterministic `priority` policy behavior, each mutation-capable operation MUST be attributable to a stable `writer_id`.

- `writer_id` may be derived from runtime principal, process identity, module identity, or an explicit declaration mechanism.
- If `priority` policy is enabled, each writer MUST resolve to a deterministic `writer_priority` value.
- Missing writer priority under `priority` policy MUST raise a configuration error before execution.

## 5) Replay Determinism and Journal Requirements

### 5.1 Determinism guarantee

Given identical input events and identical initial state, replay MUST produce bit-for-bit equivalent final state and equivalent emitted event sequence (payload + order), modulo explicitly documented nondeterministic payload fields (e.g., random IDs) that are journaled.

### 5.2 Required journal ordering keys

Each journal record that can affect state or outputs MUST carry, at minimum, the following ordering tuple:

1. `logical_time` (normalized scalar or comparable tuple)
2. `phase` (`at` < `rewrite` < `send` < `commit`)
3. `transaction_id`
4. `statement_index` (source order within transaction)
5. `writer_id` (stable origin identity)
6. `writer_seq` (per-writer monotonic sequence)

When `priority` policy is active, journal records MUST also carry `writer_priority` or a deterministic reference that resolves to it.

Implementations MUST replay using lexicographic ordering over this tuple unless an equivalent documented total order is proven.

### 5.3 Validation requirements

A conforming runtime MUST reject journals with:

- Non-monotonic `writer_seq` per `writer_id`
- Missing required ordering keys
- Missing `writer_priority` metadata when `priority` policy is active
- Incomparable `logical_time` values
- Ambiguous ties after key evaluation

## 6) Relationship to `from ... to ...` Blocks

This document defines conflict and transaction semantics for same-timestamp execution points. `from ... to ...` constructs MAY schedule repeated or interval-scoped execution, but each concrete execution instant MUST still obey §2-§5 semantics.
