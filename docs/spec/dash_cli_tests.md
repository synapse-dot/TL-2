# `dash` CLI Conformance Test Plan

This plan defines black-box tests that validate CLI behavior described in `docs/spec/dash_cli.md`.

## 1. Test Matrix Overview

Dimensions covered:

- Command parsing and help/version behavior.
- Exit code compliance.
- Structured output (`--json`) schema compliance.
- Failure behavior (missing env, corruption, incompatible snapshots).
- Concurrency behavior with two shells targeting one environment.

## 2. Harness Requirements

- Test runner capable of capturing:
  - process exit status,
  - stdout/stderr,
  - wall clock duration.
- Fixture utilities for creating:
  - valid environment,
  - corrupted journal fixture,
  - incompatible snapshot fixture.
- JSON schema validator (draft-07+ acceptable).

## 3. Global Behavior Tests

### 3.1 Help/version

1. `dash --help` returns exit `0` and usage text.
2. `dash --version` returns exit `0` and semantic version string.
3. `dash status --help` returns exit `0` and command-specific flags.

### 3.2 Argument validation

1. Unknown flag: `dash status --nope` exits `2` with usage guidance.
2. Missing required positional: `dash snapshot restore` exits `2`.

## 4. Command Conformance Tests

For each command below, validate:

- expected success exit code;
- expected failure exit code(s);
- `--json` output matches schema and envelope (`ok`, `command`, `env`, `timestamp`, `result|error`).

### 4.1 `init`

- Success: creates environment and returns `0`.
- Failure: missing env target without defaults returns `10`.
- Failure: incompatible `--from-snapshot` returns `13`.

### 4.2 `status`

- Success: returns `0` and state field.
- Failure: corrupted journal fixture returns `14`.

### 4.3 `snapshot create`

- Success: returns `0` with `snapshot_id` and `size_bytes`.
- Failure: read-only filesystem fixture returns `12`.

### 4.4 `snapshot restore`

- Success: returns `0`, revision transition fields present.
- Failure: unknown snapshot ID returns `13` or `14` per fixture semantics.

### 4.5 `journal verify`

- Success with valid journal returns `0` and `valid=true`.
- Failure with corruption returns `14` and `first_bad_seq` set.
- `--repair` partial remediation returns `21`.

### 4.6 `run`

- Success path returns `0` for payload success.
- Payload failure code propagates in range `30-255` (or mapped implementation policy).
- Missing payload executable returns `12`.

## 5. Structured Output Tests

## 5.1 Success envelope

For each successful command with `--json`, assert:

- top-level `ok=true`;
- `command` equals invoked command path;
- `timestamp` parses as RFC3339 UTC;
- `result` exists and is object;
- no `error` key.

## 5.2 Error envelope

For each forced failure with `--json`, assert:

- top-level `ok=false`;
- `error.code` belongs to allowed stable code set;
- non-empty `error.message`;
- `details` present (may be empty object);
- no `result` key.

## 6. Failure Scenario Tests

## 6.1 Missing environment

- Clear `$DASH_ENV`, remove discovery files, run `dash status`.
- Expect exit `10`, code `EX_ENV_MISSING`.

## 6.2 Corrupted journal

- Mutate checksum or truncate segment in fixture.
- Run `dash journal verify`.
- Expect exit `14`, code `EX_DATA_CORRUPT`.

## 6.3 Incompatible snapshot

- Provide snapshot with unsupported version marker.
- Run `dash snapshot restore <id>`.
- Expect exit `13`, code `EX_SNAPSHOT_INCOMPATIBLE`.

## 7. Concurrency Tests (Two Shells)

Use shell/session A and B against same env.

1. A acquires exclusive lock via long-running mutator (e.g., instrumented `snapshot create`).
2. B executes another mutator (`snapshot restore`): expect timeout/fail exit `20` with `EX_LOCK_CONFLICT`.
3. B executes read-only command (`status`): assert implementation-declared behavior:
   - either exit `0` with `state=locked`, or
   - exit `20` with lock conflict.
4. Release A lock; rerun B mutator; expect success `0`.
5. Crash A mid-operation (kill -9 in fixture) and verify stale lock recovery permits progress after lease/liveness check.

## 8. Atomicity & Recovery Tests

1. Inject failure during `snapshot restore` commit phase.
2. Verify environment is either fully restored or unchanged (no partial mixed revision).
3. Verify journal sequence continuity after concurrent command retries.

## 9. Non-Functional Conformance Targets

- Lock timeout default approximately 5s (tolerance ±1s).
- No malformed JSON in `--json` mode under any tested error path.
- Human-readable mode must still print actionable diagnostics for every non-zero exit.

## 10. Reporting Format

Each test case report MUST include:

- test ID,
- command line executed,
- fixture preconditions,
- observed exit code,
- stdout/stderr excerpts,
- pass/fail reason tied to spec section.
