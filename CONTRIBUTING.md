# Contributing to Erdos Graph Proofs

We welcome contributions to help verify more parts of the Erdos Graph engine! This document outlines the specific workflow we use, which is slightly unique because we are verifying an existing "external" codebase.

## Formal Verification vs Testing

Formal verification with Verus differs fundamentally from unit and integration testing in several ways.

### Purpose

- **Unit/Integration Tests**: Check that code works correctly for *specific inputs* and *example scenarios*
- **Formal Verification**: Proves that code works correctly for *all possible inputs* and *all possible executions*

### Structure

- **Tests**: Execute actual code with concrete values, check outputs match expectations
- **Verification**: Write mathematical specifications (preconditions, postconditions, invariants) and prove the code satisfies them for all cases

### Writing Approach

- **Tests**: Think "What examples should I test?" Write assertions for specific cases
- **Verification**: Think "What properties must always hold?" Write specifications that capture the essential behavior

Example:
```rust
// Unit test - checks one case
#[test]
fn test_register_producer() {
    let q = Queue::new();
    q.register_producer();
    assert_eq!(q.active_producer_count(), 1);
}

// Formal verification - proves all cases
fn verify_producer_registration() {
    let q = new_queue(config);
    let old_count = active_producer_count_exec(&q);
    register_producer(&q);
    let new_count = active_producer_count_exec(&q);
    assert(new_count == old_count + 1);  // Proves this holds for ANY starting count
}
```

## The Verification Workflow

Since `erdos-graph` is a standard Rust project and Verus requires a specific toolchain and syntax, we do **not** verify the `erdos-graph` crate by adding it as a `Cargo.toml` dependency.

Instead, we **include the source files directly** and write specifications in **separate modular files**.

### 1. Register Modules in `verified/src/lib.rs`

In `verified/src/lib.rs`, you need to do two things:
1. Import the original source code module using `#[path]`.
2. Register your new specification module.

```rust
// verified/src/lib.rs

use vstd::prelude::*;

// 1. Include the source file directly from the submodule
#[path = "../../erdos-graph/src/utilities/thread_safe_queue.rs"]
pub mod thread_safe_queue;

// 2. Register the module where you write specs for it
pub mod thread_safe_queue_specs;
```

### 2. Create the Spec File

Create a new file in `verified/src/` (e.g., `verified/src/thread_safe_queue_specs.rs`). This is where all the specifications live.

Structure the file like this:

```rust
// verified/src/thread_safe_queue_specs.rs
use vstd::prelude::*;

verus! {

// Refer to types in the source module using `crate::module_name::Type`

// External Type Specification
#[verifier::external_type_specification]
pub struct ExQueueConfig(crate::thread_safe_queue::QueueConfig);

// External Body Specification
#[verifier::external_body]
pub fn default_queue_config() -> (c: crate::thread_safe_queue::QueueConfig)
    ensures c.max_queue_size == 10000
{
    crate::thread_safe_queue::QueueConfig::default()
}

// Proofs and Tests
fn check_queue_config() {
    let config = default_queue_config();
    assert(config.max_queue_size == 10000);
}

}
```

### 3. Axiomatic Stubs for External Libraries

Some `erdos-graph` source files import third-party crates (`chrono`, `helix_db`, etc.) that are not available in `erdos-proofs`. You cannot include those files directly via `#[path]`. Instead, create **abstract stub types** in `verified/src/axioms/` and write specs against them.

**When to use this pattern**: the file you want to spec has `use some_external_crate::...` at the top.

**Directory layout**:
```
verified/src/
  axioms/
    mod.rs            ← pub mod entries for each stub file
    *_stubs.rs        ← abstract stub type definitions
  *_specs.rs  ← specs using those stub types
```

**Step-by-step**:

1. For each external type you need, define an abstract struct in `verified/src/axioms/`, here are some examples:

```rust
// axioms/chrono_stubs.rs
#[derive(Clone, Copy)]
pub struct DateTime {
    pub ts: u64,  // abstract timestamp
}

// open spec fn: transparent, Verus can unfold the definition
pub open spec fn dt_lt(a: DateTime, b: DateTime) -> bool {
    a.ts < b.ts
}

// #[verifier::external_body] constructor = trust boundary
#[verifier::external_body]
pub fn datetime_from_ts(ts: u64) -> (d: DateTime)
    ensures d.ts == ts
{ DateTime { ts } }
```

2. Register the stub module in `axioms/mod.rs` and import it from your spec file like this:

```rust
// axioms/mod.rs
pub mod chrono_stubs;

// ingestion_specs.rs
use crate::axioms::chrono_stubs::*;
```

3. Write `#[verifier::external_body]` wrappers for the functions under spec, using the abstract types in their signatures:

```rust
#[verifier::external_body]
pub fn chunk_date_range(
    start: DateTime,
    end: DateTime,
    chunk_size_days: u64,
) -> (chunks: Vec<(DateTime, DateTime)>)
    requires dt_lt(start, end)
    ensures
        chunks@.len() > 0,
        chunks@[0].0 == start,
        chunks@[chunks@.len() - 1].1 == end,
        forall|i: int| 0 <= i < chunks@.len() as int - 1
            ==> chunks@[i].1 == chunks@[i + 1].0,
{ unimplemented!() }
```

**Key rules for abstract stub types**:
- **Never use empty structs** (`struct Foo {}`). Verus treats all instances of an empty struct as equal, breaking any ordering or identity reasoning. Always include at least one primitive field.
- **Derive `Copy + Clone`** for value types like timestamps, durations ids. This avoids move errors when the same value appears in multiple assertions.
- Prefer `open spec fn` for properties derivable from fields. Reserve `uninterp spec fn` for predicates that must stay opaque (e.g., `valid_source` over a string field).

### Key Concepts

- **External Type Specification**: Tells Verus about the layout of structs defined in the `erdos-graph` source code.
- **`crate::module_name`**: Since the source is included at the crate root in `lib.rs`, you access it globally via `crate::`.
- **Modularity**: Keep `lib.rs` clean. It should mostly just be a list of modules. All logic goes into `*_specs.rs` files.
- **`verified/src/axioms/`**: Home for stub types representing external library types. Each file covers one library or logical group.

## Verus Features: What to Use and What to Avoid

### ✅ Features We Use

Due to our constraint of verifying external code with a different Rust toolchain, we are limited on the Verus features we can use. We can and should use:

- **`#[verifier::external_body]`**: Wrap external functions and trust their specifications
- **`#[verifier::external_type_specification]`**: Declare types from external code
- **`pub uninterp spec fn`**: Create abstract specifications for observable behavior
- **`#[verifier::when_used_as_spec]`**: Connect exec functions to their spec counterparts
- **`ensures` clauses**: Specify postconditions on external_body functions
- **`assert` statements**: Prove properties hold in verification functions
- **Capturing old/new values**: Store state before/after operations to prove transitions

Example pattern:
```rust
// Spec function (abstract)
pub uninterp spec fn active_producer_count<T>(q: &ThreadSafeQueue<T>) -> usize;

// Exec wrapper (concrete)
#[verifier::external_body]
#[verifier::when_used_as_spec(active_producer_count)]
pub fn active_producer_count_exec<T>(q: &ThreadSafeQueue<T>) -> (count: usize)
    ensures count == active_producer_count(q)
{
    q.active_producer_count()
}

// Verification function
fn verify_something() {
    let old_count = active_producer_count_exec(&q);  // Capture before
    register_producer(&q);
    let new_count = active_producer_count_exec(&q);  // Capture after
    assert(new_count == old_count + 1);  // Prove property
}
```

### ❌ Features We Cannot Use

- **`old()` in ensures clauses**: Requires mutable references, doesn't work with our spec functions
- **`requires` clauses on external functions**: We can't add preconditions to code we don't control
- **Direct `proof` blocks with ghost state**: The external code doesn't have ghost state
- **`AtomicInvariant` / `open_atomic_invariant!`**: Would require modifying erdos-graph to use vstd primitives
- **Modifying function signatures**: We must keep erdos-graph as is

### ⚠️ Common Pitfalls

These are the errors you will likely to encounter when writing specs.

**1. Integer arithmetic widens to `int` inside `verus!` spec fns**

Inside a `verus! {}` block, arithmetic on machine integers (`u64`, `usize`, etc.) is performed in the mathematical integer type `int` to avoid overflow. This means the result type is `int`, not the original machine type:

```rust
// ❌ Type error: expected u64, found int
pub open spec fn dt_add(dt: DateTime, d: ChronoDuration) -> DateTime {
    DateTime { ts: dt.ts + d.days * 86400 }
}

// ✅ Cast back to the concrete type
pub open spec fn dt_add(dt: DateTime, d: ChronoDuration) -> DateTime {
    DateTime { ts: (dt.ts + d.days * 86400) as u64 }
}
```

**2. `Seq::len()` is spec-mode — cannot bind to an exec `let`**

`vec@.len()` calls `Seq::len()`, a spec function. You can use it freely inside `assert!(...)`, but you cannot assign it to an exec variable:

```rust
// ❌ Cannot call spec function in exec context
let n = chunks@.len();
assert(chunks@[n - 1].1 == end);

// ✅ Inline the spec call inside assert
assert(chunks@[chunks@.len() - 1].1 == end);
```

**3. `forall` trigger warnings**

Z3 (the underlying SMT solver) needs a **trigger** — a pattern it watches for in the proof context — to know when to instantiate a `forall`. When Verus can't confidently choose one it prints a note like `"automatically chose triggers"`. This is not an error, but an unoptimised trigger can silently fail to fire in complex proofs.

There are three annotation options:

```rust
// #![auto] — accept the auto-chosen trigger, suppress the note
// Right when the auto-choice is a good one (e.g. sequence indexing)
forall|i: int| #![auto] 0 <= i < n ==> dt_lt(chunks@[i].0, chunks@[i].1)

// #[trigger] — pin a specific sub-expression as the trigger
// Right when you want explicit control over what fires the quantifier
forall|i: int| 0 <= i < n ==> dt_lt(#[trigger] chunks@[i].0, chunks@[i].1)

// #![trigger f(i), g(i)] — multiple trigger patterns (fires on any match)
// Right when no single trigger is reliable on its own
forall|i: int| #![trigger chunks@[i].0, chunks@[i].1] ...
```

**Practical rule**: start with `#![auto]` to silence the warning. Only switch to explicit `#[trigger]` if you see a verification failure where a `forall` isn't being instantiated when you expect it to be.

### Workarounds

Instead of `old()`, capture values explicitly:
```rust
// ❌ Cannot do this
ensures active_producer_count(q) == old(active_producer_count(q)) + 1

// ✅ Do this instead
let old_count = active_producer_count_exec(&q);
register_producer(&q);
let new_count = active_producer_count_exec(&q);
assert(new_count == old_count + 1);
```

## Guidelines

1.  **Do NOT Modify `erdos-graph` Source**: Avoid adding Verus specific syntax (`verus!`, `requires`, `ensures`) directly into the `erdos-graph/` submodule unless absolutely necessary. We want the main project to remain as is.
2.  **Trust but Verify**: When using `#[verifier::external_body]`, you are telling Verus "Trust me, the code does this". Be very careful with this. Ideally, we want to verify the body, but for `external` code, we often have to assume specs for basic functions to verify higher-level logic.
3.  **Start Small**: Pick small, pure functions or simple structs to verify first before tackling complex async logic.
4.  **Capture State Transitions**: To verify behavior, capture values before and after operations, then assert properties about the transition.
5.  **Verify Properties, Not Implementations**: Focus on what the code guarantees (e.g., "count increases by 1") rather than how it does it.

## Style Guide

### Naming Conventions

Use clear prefixes for proof/test functions to indicate their purpose:

- **`check_*`**: Simple assertions on basic properties (e.g., checking field values, simple construction)
- **`verify_*`**: Complex specifications involving multiple operations or behavioral properties

Example:
```rust
// Simple property check
fn check_queue_config() {
    let config = default_queue_config();
    assert(config.max_queue_size == 10000);
}

// Complex behavioral verification
fn verify_producer_registration() {
    let config = default_queue_config();
    let q = new_queue(config);
    assert(active_producer_count(&q) == 0);
    register_producer(&q);
}
```

### Comments

- Keep comments concise with short phrases
- Avoid multi-line explanations or numbered lists
- Avoid full sentences when possible
- Focus on "what" not "why" (the code should be self-explanatory)

Good:
```rust
// Clone preserves max_queue_size
```

Avoid:
```rust
// This function verifies that the Clone implementation for QueueConfig works as expected.
// Specifically, it ensures that the `max_queue_size` field is preserved in the new copy.
```

### Types with Private Fields

For types with private fields (like those containing `Arc`, `Mutex`, etc.), use both attributes:

```rust
#[verifier::external_body]
#[verifier::reject_recursive_types(T)]
#[verifier::external_type_specification]
pub struct ExThreadSafeQueue<T>(crate::thread_safe_queue::ThreadSafeQueue<T>);
```

### `open spec fn` vs `uninterp spec fn`

Choose based on whether Verus should be able to see the definition:

| | `open spec fn` | `uninterp spec fn` |
|---|---|---|
| Definition visible to Verus | ✅ yes — can unfold | ❌ no — treated as oracle |
| Use when | property is derivable from struct fields | property involves strings, external state, or is intentionally opaque |
| SMT reasoning | arithmetic / structural | only via `ensures` axioms |

Here are some examples:

```rust
// open: Verus can prove dt_lt(a, b) from a.ts < b.ts directly
pub open spec fn dt_lt(a: DateTime, b: DateTime) -> bool {
    a.ts < b.ts
}

// uninterp: Verus cannot look inside; truth comes from external_body ensures
pub uninterp spec fn valid_source(r: &PublicationRecord) -> bool;

#[verifier::external_body]
pub fn mk_valid_publication(year: u32) -> (r: PublicationRecord)
    requires year > 0u32
    ensures valid_source(&r)   // ← only way Verus learns this is true
{ unimplemented!() }
```

### Uninterpreted Spec Functions

For abstract specifications, use `pub uninterp spec fn`:

```rust
pub uninterp spec fn active_producer_count<T>(q: &ThreadSafeQueue<T>) -> usize;
```

Then create an exec wrapper with `#[verifier::when_used_as_spec]`:

```rust
#[verifier::external_body]
#[verifier::when_used_as_spec(active_producer_count)]
pub fn active_producer_count_exec<T>(q: &ThreadSafeQueue<T>) -> (count: usize)
    ensures count == active_producer_count(q)
{
    q.active_producer_count()
}
```

## What Properties Can We Verify?

Given our constraints, we can verify:

### ✅ Sequential Behavior
- State transitions (e.g., "register increments count by 1")
- Invariant maintenance (e.g., "count reaches 0 ⟹ done flag is set")
- State isolation (e.g., "producer operations don't affect queue size")
- Atomic transitions (e.g., "count and flag updated together")

### ✅ Functional Correctness
- Return values match specifications
- Field values after construction
- Consistency across multiple reads

### ⚠️ Limited Concurrency Properties
We can verify properties that hold in sequential execution:
- Operations are atomic (no partial updates visible)
- Locks protect state (operations don't interfere)
- Sequential consistency (operations maintain invariants)

We **cannot** verify:
- True concurrent interleavings (multiple threads executing simultaneously)
- Deadlock freedom
- Liveness properties (e.g., "operation eventually completes")

### Example: What We Verified for ThreadSafeQueue

```rust
// ✅ Verified: Register increments count by exactly 1
fn verify_producer_registration() {
    let old_count = active_producer_count_exec(&q);
    register_producer(&q);
    let new_count = active_producer_count_exec(&q);
    assert(new_count == old_count + 1);
}

// ✅ Verified: Atomic state transition
fn verify_atomic_transitions() {
    let count_before = active_producer_count_exec(&q);
    let finished_before = producers_finished_exec(&q);
    unregister_producer(&q);
    let count_after = active_producer_count_exec(&q);
    let finished_after = producers_finished_exec(&q);
    assert(count_before == 1 && !finished_before);
    assert(count_after == 0 && finished_after);
}

// ❌ Cannot verify: True concurrent behavior
// (Would require AtomicInvariant and modifying erdos-graph)
```

## Running Verus

Run the verifier from the project root:

```bash
make verify
```

If you see `verification results:: X verified, 0 errors`, you're ready to open a PR with your new specs! 
