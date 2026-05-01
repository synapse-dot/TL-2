# TL/2 Language Specification
**Version 1.0**

**File extension:** `.tlh`  
**Runtime environment manager:** `dash`  
**Design philosophy:** A systems language where time is a first-class dimension, code can safely rewrite its own future, and processes form a persistent, evolving system image.

## Suggested build order (minimal viable implementation)

1. Grammar + parser
2. Core temporal state engine (interval storage + query + conflict rules)
3. Function version table + rewrite scheduling
4. Journal + deterministic replay
5. Process/mailbox runtime
6. Dock/commit mechanism
7. Contracts + capability enforcement
8. `dash` CLI integration

## 1. Lexical Conventions

### 1.1 Keywords
The following are reserved words:

```text
state   at      until   from    to      now
fn      rewrite morph   dock    commit  yield
spawn   send    receive self    grant   revoke
if      else    loop    while   for     in
and     or      not     true    false   null
observe old     pre     post    process
```

### 1.2 Identifiers
Identifiers consist of letters, digits, and underscores, starting with a letter or underscore. They are case-sensitive.

### 1.3 Time Expressions
Time values are denoted with `t=...` in assertions, or as bare numeric literals with an optional unit suffix (`s`, `ms`, `min`, `h`, `d`). The keyword `now` represents the current instant during execution.

### 1.4 Comments
Single-line comments start with `//`. Block comments are delimited by `/*` and `*/`.

## 2. Core Model: Time-Indexed State

A TL/2 program is not a sequence of assignments but a set of temporal assertions over named timelines.

### 2.1 Variables as Timelines
A variable is a binding with a history. Its value is defined over half-open intervals `[start, end)`.

```tlh
state <name> == <value> from t=<start> to t=<end>
state <name> == <value> until t=<end>   // from now to <end>
```

- `from t=<start>` defaults to the current time if omitted.
- `to t=<end>` may be omitted for an open-ended future interval.
- `until t=<end>` is sugar for `from t=now to t=<end>`.

**Instantaneous change:**

```tlh
at t=<instant>: <name> becomes <value>
```

This schedules a discrete transition. At the given instant, the variable’s timeline switches to the new value.

**Querying history:**

```tlh
<name> @ t=<time>      // value at a specific instant
<name> now             // current value
<name>.history()       // returns list of (interval, value) tuples
```

### 2.2 The Moving `now`
`now` is a dynamic token representing the current real time. It is not a variable; it can only appear in time-specification positions.

## 3. Functions as Versioned Timelines

A function definition is not a static code block; it is a mapping from time intervals to implementations.

```tlh
fn <name>(<params>) -> <return-type> from t=<start>:
    <body>
```

If `from t=...` is omitted, `from t=now` is assumed (the function exists from the moment of definition).

**Scheduling a future version:**

```tlh
at t=<instant>:
    rewrite <name>(<params>) -> <return-type> from t=<instant>:
        <new body>
```

This uses the `rewrite` keyword with an explicit `from` clause, scheduling a new definition that takes effect at the given instant. The previous version remains active for all calls before that instant.

**Immediate rewrite:**

```tlh
rewrite <name> with <new implementation>
```

Inside a running function, `rewrite` immediately installs a new body for all future invocations. It does not alter the current call stack. The old version is archived.

**Self-reference in rewrite:**
Inside any function, `self` refers to the current function’s version.

```tlh
rewrite self with mutated_body
```

## 4. Self-Modification Primitives

### 4.1 `rewrite` – Future calls
- Changes the function’s definition for all new invocations, effective immediately (in the sense that the next time the function is called, the new version is used).
- Must pass contract checks (see §8) before being installed. Failure aborts the rewrite and preserves the old version.
- When targeting another process, a capability is required (see §8).

### 4.2 `morph` – Continuation morphing
`morph` replaces the currently executing function with another one, discarding the current stack frame in a tail-call fashion. The process continues running with the new code, preserving its PID and mailbox.

```tlh
morph <target-fn> with (<args>)
```

- `<target-fn>` must be a callable function (named or dynamically generated).
- The arguments must match the expected parameters of the target.
- The return type must be compatible with the original function’s return.
- It is type-checked at compile time and runtime.

## 5. Speculative Execution: Docks

A dock is an isolated timeline where code can freely mutate without affecting the main timeline. The outcome can be inspected and selectively merged using `commit`.

```tlh
dock <name>:
    // code that can rewrite, morph, simulate, etc.
    observe <var> = <expression>
```

- Inside a dock, the entire program state is a copy-on-write fork of the main timeline.
- Any `rewrite`, `morph`, or message send only affects the dock’s internal state.
- The dock’s execution is purely functional in terms of its outer effect: at the end of the block, the dock either produces a value (via `yield`) or is discarded.

Shared with the outer scope via `observe` and `yield`:
- `observe` marks a variable to be visible outside the dock after execution, without committing code changes.
- `yield` is used to explicitly transfer a piece of code (e.g., a new function version) out of the dock, but it does not automatically install it. That requires a subsequent `commit`.

## 6. Processes and Concurrency

### 6.1 Process Creation
Processes are lightweight, isolated actors. Each has a PID, a mailbox, and an executing function.

```tlh
spawn <fn-reference> [as <pid-var>]
```

### 6.2 Communication
Processes communicate exclusively through asynchronous message passing.

```tlh
send(<pid>, <message>)
receive <pattern> -> <body>
```

### 6.3 Self-Awareness and Supervision

```tlh
self()               // returns the PID of the calling process
process_info(<pid>)  // returns a record with parent, children, current function, etc.
```

### 6.4 Process Code Management
- A process can `rewrite` its own top-level function or any helper function it defines.
- A supervisor with the right capability can `rewrite` a function inside a worker process.

## 7. Runtime and Execution Model

### 7.1 JIT Compilation and Hot Swapping
TL/2 source is compiled on-the-fly into a versioned intermediate representation (V-IR). The runtime maintains a code cache keyed by function name and time interval.

### 7.2 Persistence and Crash Recovery
Every state assertion, `at` transition, message send, `rewrite`, and `morph` is written to an append-only event journal. The journal plus process snapshots allows pause/save and resume through `dash`.

### 7.3 Time-Travel Debugging
Because the full history of state and code is preserved, a debugger can step forwards and backwards, inspect active versions, and branch via docks.

## 8. Safety and Security

### 8.1 Contract System
Functions can declare pre- and post-conditions:

```tlh
fn transfer(from, to, amount):
    pre: balance[from] >= amount
    post: balance[from] == old(balance[from]) - amount
    post: balance[to] == old(balance[to]) + amount
    body: ...
```

### 8.2 Capabilities for Remote Rewrites

```tlh
grant <target_pid>:<function_name> to <delegate_pid> [until t=<expiry>]
```

Without a valid grant at rewrite time, the attempt fails with `CapabilityError`.

### 8.3 Type Safety of `morph`
The compiler statically checks morph target signature compatibility and runtime validates before jump.

## 9. Language Integration: The `dash` Environment Manager

### 9.1 Environment Lifecycle

```bash
dash create-env <name>
dash load-env <name>
dash <name> -load <file.tlh>
dash <name> -quit
dash delete-env <name>
dash list
```

### 9.2 Interactive Development
Running `dash <name>` opens a REPL attached to the environment.

### 9.3 File Extension
All TL/2 source files use `.tlh`.

## 10. Full Example: A Self-Improving Agent

```tlh
state mode == "explore" until t=midnight

fn policy(sensor) -> action from t=now:
    if sensor.danger > 0.7:
        return action.evade
    else:
        return action.explore

process main_loop:
    state pid = self()
    loop every 100ms:
        let data = sense()
        let act = policy(data)
        execute(act)

        if now % 60s < 100ms:
            dock experiment:
                rewrite policy with mutate(policy)
                simulate_history(window=60s)
                observe score = average_reward
                if score > baseline:
                    yield policy

            if experiment.score > baseline:
                at now + 500ms:
                    commit experiment.policy
```

## 11. Summary of Keywords

| Keyword  | Purpose |
|----------|---------|
| `state`  | Declare a value over a time interval |
| `at`     | Schedule a discrete time transition |
| `until`  | Shorthand for “from now to a given instant” |
| `from`   | Start of a time interval |
| `to`     | End of a time interval |
| `now`    | Current execution time |
| `fn`     | Define a function |
| `rewrite`| Change a function’s code (immediate or future) |
| `morph`  | Tail-call into a different function right now |
| `dock`   | Isolated speculative execution timeline |
| `commit` | Apply code changes from a dock to main timeline |
| `yield`  | Pass a value/code out of a dock |
| `observe`| Mark a variable for external inspection |
| `spawn`  | Create a new process |
| `send`   | Send a message to a process |
| `receive`| Receive a message |
| `self`   | PID of the current process |
| `grant`  | Allow another process to rewrite one’s code |
| `revoke` | Cancel a previous grant |
| `old`    | In contracts, refer to value before execution |
| `pre`    | Precondition |
| `post`   | Postcondition |
