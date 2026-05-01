# `dash` CLI Specification

This document defines command-line behavior for the `dash` tool, including syntax, flags, exit codes, machine-readable output, failure behavior, and concurrency rules.

## 1. Global CLI Contract

## 1.1 Invocation

```bash
dash [GLOBAL_FLAGS] <COMMAND> [COMMAND_ARGS]
```

## 1.2 Global flags

- `-h`, `--help`: print help for the selected scope (root or command) and exit `0`.
- `-V`, `--version`: print version string and exit `0`.
- `--json`: enable structured output mode for supported commands.
- `--no-color`: disable ANSI color output.
- `-q`, `--quiet`: suppress non-essential human-readable output.
- `-v`, `--verbose`: increase diagnostics verbosity (repeatable up to 3 times).
- `--env <name-or-path>`: target environment by logical name or explicit path.
- `--cwd <path>`: treat `<path>` as working root for environment discovery.

## 1.3 Environment resolution order

When `--env` is omitted, `dash` resolves the target environment in this order:

1. `$DASH_ENV` if set.
2. `.dash/env` in `--cwd` (or current working directory).
3. Error if unresolved (`EX_ENV_MISSING`, exit `10`).

## 1.4 Structured output envelope

When `--json` is present, all successful command responses MUST conform to:

```json
{
  "ok": true,
  "command": "<string>",
  "env": "<string>",
  "timestamp": "<RFC3339 UTC>",
  "result": { }
}
```

And all failures MUST conform to:

```json
{
  "ok": false,
  "command": "<string>",
  "env": "<string|null>",
  "timestamp": "<RFC3339 UTC>",
  "error": {
    "code": "<stable_error_code>",
    "message": "<human_readable>",
    "details": { }
  }
}
```

---

## 2. Commands

## 2.1 `dash init`

Create or bootstrap an environment.

### Syntax

```bash
dash init [--env <name-or-path>] [--force] [--from-snapshot <snapshot-id>]
```

### Flags

- `--force`: overwrite existing incomplete bootstrap state.
- `--from-snapshot <snapshot-id>`: initialize from an existing snapshot.

### Exit codes

- `0`: environment initialized.
- `10`: missing environment target.
- `12`: filesystem/permission error.
- `13`: incompatible snapshot.
- `14`: corrupted snapshot or journal bootstrap payload.

### `--json` result schema

```json
{
  "result": {
    "created": true,
    "env_path": "<string>",
    "snapshot": "<string|null>",
    "generation": "<integer>"
  }
}
```

---

## 2.2 `dash status`

Inspect current environment health and metadata.

### Syntax

```bash
dash status [--env <name-or-path>] [--detail]
```

### Flags

- `--detail`: include extended counters and integrity metadata.

### Exit codes

- `0`: status available.
- `10`: environment missing.
- `11`: environment metadata missing required fields.
- `14`: journal corrupted.

### `--json` result schema

```json
{
  "result": {
    "state": "ready|degraded|locked|recovering",
    "head_revision": "<string>",
    "journal": {
      "healthy": "<boolean>",
      "last_seq": "<integer>"
    },
    "locks": {
      "holder_pid": "<integer|null>",
      "mode": "none|shared|exclusive"
    }
  }
}
```

---

## 2.3 `dash snapshot create`

Create a point-in-time snapshot.

### Syntax

```bash
dash snapshot create [--env <name-or-path>] [--label <text>] [--message <text>]
```

### Flags

- `--label <text>`: short stable identifier (user-provided alias).
- `--message <text>`: optional annotation.

### Exit codes

- `0`: snapshot created.
- `10`: missing environment.
- `12`: write failure.
- `14`: journal corrupted.
- `20`: lock acquisition timeout / concurrent writer conflict.

### `--json` result schema

```json
{
  "result": {
    "snapshot_id": "<string>",
    "created_at": "<RFC3339 UTC>",
    "parent_revision": "<string>",
    "size_bytes": "<integer>"
  }
}
```

---

## 2.4 `dash snapshot restore`

Restore environment state to a snapshot.

### Syntax

```bash
dash snapshot restore [--env <name-or-path>] <snapshot-id> [--force]
```

### Flags

- `--force`: bypass interactive or safety checks for dirty runtime state.

### Exit codes

- `0`: restore complete.
- `10`: environment missing.
- `13`: incompatible snapshot format or version.
- `14`: snapshot payload corrupted.
- `20`: unable to acquire exclusive lock.

### `--json` result schema

```json
{
  "result": {
    "restored_snapshot": "<string>",
    "previous_revision": "<string>",
    "new_revision": "<string>",
    "replayed_journal_entries": "<integer>"
  }
}
```

---

## 2.5 `dash journal verify`

Validate journal integrity and continuity.

### Syntax

```bash
dash journal verify [--env <name-or-path>] [--repair]
```

### Flags

- `--repair`: attempt non-destructive repair where possible.

### Exit codes

- `0`: journal valid.
- `10`: missing environment.
- `14`: journal corrupted (unrecoverable or repair failed).
- `21`: partial repair applied; operator action required.

### `--json` result schema

```json
{
  "result": {
    "valid": "<boolean>",
    "segments_checked": "<integer>",
    "first_bad_seq": "<integer|null>",
    "repair": {
      "attempted": "<boolean>",
      "applied": "<boolean>"
    }
  }
}
```

---

## 2.6 `dash run`

Execute a command within a resolved environment context.

### Syntax

```bash
dash run [--env <name-or-path>] [--] <program> [args...]
```

### Flags

- `--`: end of dash flags and start of payload command.

### Exit codes

- `0`: payload succeeded.
- `10`: missing environment.
- `12`: failed to exec payload.
- `20`: environment lock conflict.
- `30-255`: propagated payload process exit status.

### `--json` result schema

```json
{
  "result": {
    "argv": ["<string>", "..."],
    "duration_ms": "<integer>",
    "exit_status": "<integer>",
    "signal": "<string|null>"
  }
}
```

---

## 3. Stable Exit Code Table

- `0` success.
- `1` generic internal error.
- `2` invalid CLI usage / argument parse error.
- `10` missing environment.
- `11` invalid environment metadata/config.
- `12` filesystem or OS I/O error.
- `13` incompatible snapshot format/version.
- `14` corrupted journal/snapshot data.
- `20` concurrency lock conflict/timeout.
- `21` repaired-with-warnings state requiring operator follow-up.

## 4. Required Failure Cases

Implementations MUST surface the following with stable error codes and actionable messages:

1. **Missing env**: no resolvable environment target (`10`).
2. **Corrupted journal**: checksum mismatch, sequence gap, or malformed segment (`14`).
3. **Incompatible snapshot**: unknown snapshot schema or unsupported runtime generation (`13`).
4. **Filesystem failures**: permission denied, disk full, missing required path (`12`).
5. **Invalid arguments**: malformed flags/arity (`2`).

In `--json` mode, `error.code` MUST be one of:

- `EX_USAGE`
- `EX_ENV_MISSING`
- `EX_ENV_INVALID`
- `EX_IO`
- `EX_SNAPSHOT_INCOMPATIBLE`
- `EX_DATA_CORRUPT`
- `EX_LOCK_CONFLICT`
- `EX_REPAIR_PARTIAL`
- `EX_INTERNAL`

## 5. Concurrency Semantics

When two shells target the same environment simultaneously, implementations MUST enforce lock-based serialization.

## 5.1 Lock model

- Read-only commands (`status`, `journal verify` without `--repair`) SHOULD acquire **shared** locks.
- Mutating commands (`init`, `snapshot create`, `snapshot restore`, `journal verify --repair`, `run` when it mutates env state) MUST acquire **exclusive** locks.
- Shared lock holders MUST block exclusive lock acquisition until released.

## 5.2 Conflict behavior

- If lock cannot be obtained within timeout (default 5 seconds), command fails with exit `20` and error code `EX_LOCK_CONFLICT`.
- `status` MAY return state `locked` when lock metadata is observable.
- Stale lock detection SHOULD verify holder liveness (PID probe or lease expiry) before failing.

## 5.3 Atomicity guarantees

- Mutating commands MUST either fully commit or leave pre-command state intact.
- Snapshot restore MUST be all-or-nothing with crash-safe rollback marker.
- Journal append MUST preserve monotonic sequence IDs, even under contention.

## 5.4 Cross-shell examples

- Shell A starts `dash snapshot create` (exclusive lock acquired).
- Shell B runs `dash status`:
  - if shared read allowed while writer active is disallowed by implementation, return `20`; or
  - if lock metadata is readable, return `0` with `state=locked`.
- Shell B runs `dash snapshot restore`: MUST fail with `20` until A releases lock.
