# Type System Specification

This document defines the static and runtime typing rules for the language, including base and composite types, inference and annotation behavior, coercion policy, transformation signature compatibility (`rewrite`, `morph`), contract-expression typing, and runtime failure semantics.

## 1. Core Types

### 1.1 Primitive scalar types

- `int`: Signed integer values with exact arithmetic in the language semantics.
- `float`: IEEE-754 floating-point values.
- `bool`: Logical truth values (`true`, `false`).
- `string`: Immutable Unicode text values.
- `time`: Timestamp/datetime value used for temporal comparisons and arithmetic defined by the standard library.
- `pid`: Process identifier / entity identifier value; opaque and non-arithmetic.
- `null`: The sole null sentinel value indicating absence.

### 1.2 Structural and nominal compound types

- **Record type**: Product type with named fields.
  - Syntax (schematic): `{ field1: T1, field2: T2, ... }`
  - Field names are unique within a record type.
  - Width subtyping is not implicit unless a rule explicitly allows it (see compatibility sections).
- **Enum type**: Tagged sum type with finite constructors/variants.
  - Syntax (schematic): `enum E { A, B, C }` or payload variants where supported.
  - Distinct enum declarations are distinct types, even if variant names match.
- **Function type**: Mapping from argument tuple to return type.
  - Syntax: `(T1, T2, ...) -> R`
  - Arity is part of the type.
  - Parameter and result variance are governed by compatibility rules in §4.

### 1.3 `null` and nullable values

- `null` has type `null`.
- A non-nullable type `T` does not implicitly accept `null`.
- Optionality must be explicit via a nullable/union form when supported by the type context (e.g., `T | null`).

## 2. Inference, Annotations, and Cast Policy

### 2.1 Type inference

The compiler infers types from:

- Literal forms (`1` → `int`, `1.0` → `float`, etc.).
- Local initializer expressions.
- Function return flow where uniquely determined.
- Generic/contextual constraints from call sites and operators.

Inference must produce a principal type where possible. If multiple incompatible candidates remain, compilation fails with a type ambiguity error and requires an explicit annotation.

### 2.2 Explicit annotations

Programmers may annotate:

- Variable declarations.
- Function parameters and returns.
- Record fields.
- Intermediate expressions where disambiguation is required.

An explicit annotation is authoritative: the expression must be type-checkable against the annotation (possibly with allowed coercions from §2.3). If not, compilation fails.

### 2.3 Allowed coercions (implicit)

Only the following implicit coercions are permitted:

1. `int -> float` (widening numeric coercion).
2. `T -> T` (identity).
3. Contextual lifting into explicitly nullable target forms where the source already matches the non-null arm (e.g., `T -> T | null`).

No other implicit coercions are permitted.

### 2.4 Forbidden implicit casts

The checker must reject implicit conversion for:

- `float -> int` (lossy).
- `bool <-> int`.
- `string <-> int/float/bool/time/pid`.
- `time <-> int/float/string`.
- `pid <-> string/int`.
- `null -> T` where `T` is non-nullable.
- Cross-enum conversion between distinct enum declarations.
- Record-to-record implicit field dropping or filling unless exact-type/compatibility rules explicitly permit.

Explicit conversion APIs may exist in libraries; they are not implicit casts.

## 3. Operator and Expression Typing (summary)

- Arithmetic operators require numeric operands (`int`/`float`) and follow numeric promotion (`int` + `float` => `float`).
- Logical operators require `bool` operands.
- Equality operators require comparable pairings (same type or allowed coercion domain).
- Ordering operators require ordered domains (`int`, `float`, `time`, and strings if language-defined lexicographic order is enabled).

## 4. Signature Compatibility for `rewrite` and `morph`

Let a source signature be `S_in -> S_out` and candidate target signature be `T_in -> T_out`.

### 4.1 `rewrite` compatibility

`rewrite` is shape-preserving transformation over the same semantic entity class.

A candidate function is compatible with `rewrite` iff:

1. **Input compatibility**: `S_in` and `T_in` are equivalent types (after alias expansion), except that `T_in` may be a supertype only when the checker can prove all required fields/variants used in the body are available.
2. **Output compatibility**: `T_out` is assignable to `S_out` with no lossy coercion.
3. **Effect/contract preservation**: pre/post contract types remain well-typed under original binding names.

In practice, `rewrite` should be treated as near-invariant for both input and output; any widening/narrowing requires an explicit adapter layer, not implicit acceptance.

### 4.2 `morph` compatibility

`morph` is intentional type-changing transformation.

A candidate is compatible with `morph` iff:

1. **Input acceptance**: `T_in` can consume `S_in` (contravariant-safe input use).
2. **Output declaration**: `T_out` may differ from `S_out`, but must match the declared morph target type exactly.
3. **Total mapping proof**: every source variant/field path is handled, or unmatched cases are explicitly marked and rejected at compile time.
4. **Contract rebindability**: contract expressions referencing source values through `old(...)` remain typeable under source snapshot typing.

### 4.3 Function-typed fields and higher-order positions

Where function types appear as fields/parameters:

- Parameter positions are contravariant.
- Return positions are covariant.
- For `rewrite`, higher-order function members are treated invariant unless explicitly declared variance-safe.

## 5. Contract Expression Typing

Contracts are checked in scoped environments.

### 5.1 `pre` scope

- `pre` is evaluated before execution.
- Available bindings: function parameters, immutable globals/constants, and pure helper functions.
- Return value binding is unavailable.

### 5.2 `post` scope

- `post` is evaluated after successful execution (or after tentative state in transactional runtimes before commit).
- Available bindings:
  - Parameters (final view unless explicitly shadowed).
  - Result binding (e.g., `result`) with function return type.
  - `old(x)` snapshots from pre-state.

### 5.3 `old(...)` typing

- `old(e)` is type-checked in the `pre` environment.
- `type(old(e)) == type_pre(e)`.
- `old(...)` is read-only and cannot appear on assignment LHS.
- `old(...)` is valid only in `post` (and derived post-like assertions), not in `pre`.

### 5.4 Contract boolean requirement

Each top-level `pre`/`post` expression must type-check to `bool`. Non-boolean contract expressions are compile-time type errors.

## 6. Runtime Type Checks and Failure Modes

### 6.1 Runtime checks inserted

Runtime checks occur at:

- Dynamic boundaries (FFI, deserialization, reflective calls).
- `morph`/`rewrite` boundaries when static proof is incomplete.
- Contract evaluation points (`pre`, `post`).

### 6.2 `TypeError`

A `TypeError` is raised when a runtime value violates expected type shape/domain, including:

- Missing record field.
- Wrong enum variant/payload type.
- Failed cast at dynamic boundary.
- Contract expression accessing ill-typed dynamic value.

`TypeError` payload should include expected type, actual runtime shape, and source location.

### 6.3 Rollback semantics

Operations with transactional semantics (including `morph`/`rewrite` in transactional contexts) follow:

1. Evaluate transformation and postconditions against tentative state.
2. If any `TypeError` or contract failure occurs, rollback all tentative mutations.
3. Surface failure to caller as typed runtime error (`TypeError` for type faults, contract violation for boolean-false assertions).

For non-transactional contexts, partial side effects may persist unless explicitly wrapped in a transaction.

## 7. Compatibility Matrix (`morph` / `rewrite`)

| Source -> Target case | `rewrite` | `morph` | Notes |
|---|---:|---:|---|
| Exact same input/output types | ✅ | ✅ | Trivially compatible. |
| Input widened, output same | ⚠️ Limited | ✅ | `rewrite` only if proven body-safe; `morph` allowed via contravariant input acceptance. |
| Input narrowed, output same | ❌ | ⚠️ Conditional | Unsafe unless guarded and proven total in `morph`. |
| Output widened (supertype) | ⚠️ Limited | ✅ | `rewrite` only if still assignable to declared output contract. |
| Output narrowed (subtype) | ❌ | ✅ (if declared target) | `rewrite` forbids observable contract narrowing. |
| Primitive change (`int -> string`) | ❌ | ✅ | Requires explicit `morph` mapping logic. |
| Record field add/drop/rename | ❌ | ✅ | `morph` must define full mapping and defaults/derivations. |
| Enum variant remap | ❌ | ✅ | Must handle all variants or fail compile-time exhaustiveness. |
| Function arity change | ❌ | ✅ | Treated as type-changing transform only. |
| Nullable/non-nullable flip | ❌ | ✅ | `morph` must explicitly handle `null` cases. |

Legend:

- ✅ Allowed by default.
- ⚠️ Allowed only under additional static proof obligations.
- ❌ Disallowed.
