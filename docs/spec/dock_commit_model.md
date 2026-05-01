# Dock Commit Model

## 1) Artifact Identity

A dock commit always targets an **immutable artifact version object** rather than a mutable in-memory value.

Each version object MUST contain at least:

- `artifact_id`: stable logical identity (for example, `package://team/service/config`).
- `version_hash`: content-addressed digest of the normalized payload (for example, SHA-256 over canonical bytes).
- `payload_format`: schema/type identifier used to decode and validate payload bytes.
- `created_at`: monotonic commit timestamp (or logical clock) assigned by the commit system.
- `producer_id`: principal or runtime identity that produced the candidate.
- `signature`: detached or envelope signature over `(artifact_id, version_hash, payload_format, metadata)`.

Optional but common fields:

- `parent_version_hash`: predecessor pointer for lineage/audit.
- `provenance`: build inputs, toolchain, environment, and policy evaluation records.
- `capability_attestations`: proofs used by capability checks.

### Identity invariants

1. **Immutability**: once published, a version object cannot be modified.
2. **Hash integrity**: `version_hash` MUST verify against canonical payload bytes.
3. **Signature integrity**: signature MUST verify against the declared `producer_id` trust root.
4. **Referential stability**: any handle to `version_hash` always resolves to identical payload bytes.

---

## 2) `yield policy` Return Contract

`yield policy` decides what a docked computation exports to a potential commit target.

It MUST return exactly one of the following result classes:

1. **Material value**
   - A concrete serialized value payload.
   - Commit path: system computes `version_hash` from canonicalized payload and creates a new version object.

2. **Symbol binding**
   - A named binding in a symbol table, e.g., `symbol://env/prod/current` -> candidate value or handle.
   - Commit path: system resolves binding target and either materializes a new version or re-points symbol (policy-dependent).

3. **Version handle**
   - A direct reference to an existing immutable version object (e.g., by `version_hash` URI).
   - Commit path: no payload re-materialization required; system validates handle existence and admissibility.

### Canonical return envelope

To avoid ambiguity, the policy engine SHOULD emit:

```text
YieldResult {
  kind: "value" | "binding" | "version_handle",
  payload: <kind-specific body>,
  contract_ref: <type/contract id>,
  capability_ref: <required capability set>,
}
```

Any multi-value output MUST be wrapped as one value object; multi-target commits are orchestrated above this layer.

---

## 3) Commit Validation Pipeline

`commit` is an all-or-nothing transaction over a single target install point.

1. **Target snapshot + precondition capture**
   - Read target head (`target_head_before`) and policy revision.
   - Record optimistic concurrency precondition (`expected_head`).

2. **Capability authorization**
   - Verify caller has required capabilities for this artifact namespace and operation.
   - Verify delegated tokens/attestations are unexpired and scope-matching.

3. **Yield decoding + kind validation**
   - Decode `YieldResult` and verify `kind` is allowed for target class.
   - Resolve symbol bindings or version handles as needed.

4. **Type and contract checks**
   - Validate payload against schema/type (`contract_ref`).
   - Run semantic contract checks (invariants, compatibility, downgrade/upgrade rules).
   - Reject unsafe transitions (for example, forbidden ABI break).

5. **Integrity and provenance checks**
   - Recompute/verify `version_hash`.
   - Verify signatures, provenance requirements, and policy guardrails.

6. **Atomic install prepare**
   - Build install record: `(target, new_version_hash, expected_head, metadata)`.
   - Acquire transaction/lock primitive for target install point.

7. **Atomic compare-and-swap install**
   - Install succeeds only if current head still equals `expected_head`.
   - On success, write new head + append immutable audit event in one atomic transaction.

8. **Post-commit publication**
   - Emit commit event (`Committed`) with old/new heads and validation receipts.
   - Release lock/transaction resources.

---

## 4) Failure Semantics

### 4.1 Partial commit prohibition

A commit MUST be **atomic**:

- No state where target head advances without a corresponding audit/event record.
- No state where audit/event record exists for a head that was not installed.
- If any stage fails before atomic CAS, externally visible target state remains unchanged.

### 4.2 Retries

Retries are allowed only for **transient** failures (lock timeout, temporary dependency outage).

- Retries MUST re-run validation that depends on mutable context (capabilities, policy revision, target head).
- Retries SHOULD use bounded exponential backoff with jitter.
- Non-transient failures (schema violation, signature failure, policy denial) are terminal and non-retriable without input change.

### 4.3 Conflict handling after dock fork rewrite

If the target was rewritten after the dock fork (i.e., `current_head != expected_head` at CAS time):

1. Return explicit conflict (`CommitConflict`) with:
   - `expected_head`
   - `observed_head`
   - optional `observed_commit_metadata`
2. Do not auto-merge silently.
3. Caller may:
   - rebase/recompute from `observed_head`, then re-attempt commit, or
   - abort and keep fork result as non-installed artifact.

---

## 5) Example Timelines

### 5.1 Successful commit

```text
T0  Fork dock from target head H10
T1  Compute candidate C (YieldResult.kind = value)
T2  Validate capabilities + contracts + integrity
T3  Begin atomic install with expected_head=H10
T4  CAS succeeds: H10 -> H11 (version_hash=V11)
T5  Append Committed event {old:H10,new:H11}
T6  Return CommitSuccess(new_head=H11)
```

Outcome: exactly one new head (`H11`) is visible; audit trail matches installed head.

### 5.2 Race-condition conflict

```text
T0  Actor A forks at H10
T1  Actor B forks at H10
T2  Actor B commits first: CAS H10 -> H11B succeeds
T3  Actor A reaches install with expected_head=H10
T4  CAS fails because observed_head=H11B
T5  Actor A gets CommitConflict(expected=H10, observed=H11B)
T6  Actor A rebase/recompute on H11B, producing C2
T7  Actor A retries commit with expected_head=H11B
T8  CAS succeeds: H11B -> H12A
```

Outcome: no partial install for Actor A's first attempt; history remains linearizable by successful CAS transitions.
