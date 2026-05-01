# Process Model Specification

This document defines the runtime lifecycle for processes, mailbox semantics, supervision behavior, and hot-rewrite behavior.

## 1. Process Lifecycle States

A process MUST be in exactly one of the following states at any time:

- **`starting`**: process has been allocated an identity and runtime context, but user code has not yet begun event handling.
- **`running`**: process is actively eligible for scheduling and may send/receive messages.
- **`waiting`**: process is blocked pending external work (mailbox input, timer, I/O completion, or supervisor signal). It remains alive.
- **`exited`**: process terminated intentionally (normal return or explicit stop).
- **`crashed`**: process terminated due to unhandled fault/panic/error.
- **`restarting`**: supervisor has selected process for restart and is reinitializing runtime state.

### 1.1 State transition constraints

- `starting -> running` when initialization succeeds.
- `starting -> crashed` when initialization fails.
- `running -> waiting` when no runnable work remains or process blocks.
- `waiting -> running` when awaited condition resolves.
- `running|waiting -> exited` on graceful termination.
- `running|waiting|starting -> crashed` on unhandled failure.
- `crashed|exited -> restarting` only if a supervisor strategy mandates restart.
- `restarting -> starting` after restart preparation is complete.
- `exited` is terminal when restart policy is `none` or restart budget exhausted.
- `crashed` is terminal when restart policy is `none` or restart budget exhausted.

### 1.2 Reference lifecycle diagram

```text
                       +----------------+
         init ok       |                |   no work / block
   +------------------>|    running     +------------------+
   |                   |                |                  |
   |                   +--------+-------+                  v
   |                            |                    +-------------+
   |                            | graceful stop      |   waiting   |
   |                            v                    +------+------+ 
+--+-------+               +---------+                      |
| starting |               | exited  |<---------------------+
+--+---+---+               +----+----+      wake/event
   |   |                         |
   |   | init fail               | supervised restart
   |   v                         v
   | +---------+            +-----------+
   | | crashed |----------->| restarting|
   | +----+----+  supervised+-----+-----+
   |      ^                      |
   |      +----------------------+ restart prepared
   |                     terminal when policy/budget disallows restart
   +---------------------------------------------------------------
```

## 2. Mailbox Semantics

### 2.1 Ordering guarantees

Mailbox delivery uses per-sender FIFO ordering and process-local dequeue ordering:

1. **Per-sender FIFO**: for any sender `S` and receiver `R`, if `S` sends `m1` then `m2` to `R`, `R` MUST NOT observe `m2` before `m1`.
2. **Cross-sender interleaving**: messages from distinct senders may interleave nondeterministically.
3. **Single-consumer dequeue**: each process dequeues from its own mailbox serially; message handlers do not run concurrently within a single process unless explicitly modeled by child processes.
4. **At-most-once local delivery**: once dequeued by a process, the same mailbox record MUST NOT be dequeued again.

### 2.2 Persistence across checkpoint/recovery

During checkpoint:

- mailbox contents for live processes in `starting|running|waiting` MUST be snapshotted with dequeue cursor position;
- in-flight but not yet enqueued transport frames MAY be replayed by transport layer, but mailbox snapshot defines canonical process-visible queue.

During recovery:

- restored process mailbox MUST preserve relative order of persisted entries;
- entries dequeued before checkpoint MUST NOT reappear;
- entries still queued at checkpoint MUST remain available in the same relative order;
- process resumes in `waiting` or `running` according to scheduler policy, with mailbox semantics unchanged.

### 2.3 Reference mailbox event sequence

```text
Actors: SenderA, SenderB, Receiver

1) SenderA -> Receiver: A1
2) SenderB -> Receiver: B1
3) SenderA -> Receiver: A2
4) Receiver dequeues A1
5) CHECKPOINT
6) Receiver dequeues B1
7) CRASH + RECOVER from checkpoint
8) Receiver dequeues B1 (again, because dequeue at step 6 happened after checkpoint)
9) Receiver dequeues A2

Guarantees:
- A1 always before A2.
- B1 relative to A1/A2 may vary before checkpoint, but post-recovery order follows snapshot state.
```

## 3. Exit Signals and Supervision

### 3.1 Exit signal structure

On any termination (`exited` or `crashed`), runtime emits an exit signal to linked/supervising processes with fields:

- `process_id`: terminated process identifier.
- `reason`: machine-readable reason code (`normal`, `shutdown`, `error`, `panic`, `killed`, ...).
- `detail`: optional structured payload (error text, stack hash, metadata).
- `final_state`: `exited` or `crashed`.
- `timestamp`: monotonic/runtime time of termination.
- `restart_count`: restarts attempted in current supervision window.
- `correlation_id` (optional): propagation key for tracing cascading failures.

### 3.2 Supervisor strategies

#### one-for-one

- Only the failed child is restarted.
- Siblings are unaffected.
- Suitable for isolated workers.

#### one-for-all

- Failure of any child causes supervisor to stop all children.
- Supervisor then restarts whole child set in declared start order.
- Suitable for tightly coupled components with shared invariants.

#### backoff restart policy

Backoff defines restart pacing independent of one-for-one vs one-for-all scope:

- delay increases per consecutive failure (e.g., exponential or capped linear);
- jitter SHOULD be applied to avoid synchronized restart storms;
- restart budget/window limits attempts; exhaustion transitions failure to terminal and escalates exit signal.

### 3.3 Reference supervision event sequences

```text
Sequence A: one-for-one
1) Child C2 crashes.
2) Supervisor receives exit(C2, crashed).
3) Supervisor schedules restart for C2 only.
4) C1/C3 continue running.
5) C2 enters restarting -> starting -> running.
```

```text
Sequence B: one-for-all
1) Child C2 crashes.
2) Supervisor receives exit(C2, crashed).
3) Supervisor stops C1 and C3 (orderly shutdown).
4) Supervisor applies restart policy (possibly backoff delay).
5) Supervisor restarts C1, then C2, then C3.
```

## 4. Rewritten Code Behavior

When code is rewritten/reloaded, behavior differs for existing versus new processes.

### 4.1 Already-running processes

- Existing process instances continue executing their currently loaded code image by default.
- Their in-memory state and mailbox are preserved.
- They switch to rewritten logic only at a defined upgrade boundary, if supported (e.g., explicit migrate callback, restart, or next spawn generation).
- Without an explicit upgrade boundary, no mid-handler code swap occurs.

### 4.2 Newly-spawned processes

- Any process spawned after rewrite commit point MUST load the newest code image.
- New process behavior reflects updated handlers, initialization, and protocol logic immediately.

### 4.3 Mixed-generation system behavior

The system MAY temporarily run mixed generations:

- Generation N: already-running processes.
- Generation N+1: newly spawned (or restarted) processes after rewrite.

Protocols and message formats SHOULD remain backward-compatible across at least one generation boundary, or supervisors SHOULD force coordinated restart to eliminate skew.

### 4.4 Reference rewrite sequence

```text
1) Generation N workers W1, W2 are running.
2) Rewrite to Generation N+1 is applied.
3) W1/W2 continue on Generation N code.
4) New worker W3 is spawned; W3 runs Generation N+1.
5) W2 crashes; supervisor restarts it.
6) Restarted W2 now runs Generation N+1.
7) Optional rolling restart converges all workers to N+1.
```

## 5. Conformance Notes

Implementations are conformant if they:

- enforce the lifecycle and transition constraints;
- preserve mailbox ordering and checkpoint/recovery guarantees;
- emit structured exit signals and apply declared supervision strategy;
- apply rewrite semantics distinguishing existing versus newly spawned processes.
