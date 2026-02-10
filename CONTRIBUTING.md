# Contributing to Erdos Graph Proofs

We welcome contributions to help verify more parts of the Erdos Graph engine! This document outlines the specific workflow we use, which is slightly unique because we are verifying an existing "external" codebase.

## The Verification workflow

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

### Key Concepts

- **External Type Specification**: Tells Verus about the layout of structs defined in the `erdos-graph` source code.
- **`crate::module_name`**: Since the source is included at the crate root in `lib.rs`, you access it globally via `crate::`.
- **Modularity**: Keep `lib.rs` clean. It should mostly just be a list of modules. All logic goes into `*_specs.rs` files.

## Guidelines

1.  **Do NOT Modify `erdos-graph` Source**: Avoid adding Verus specific syntax (`verus!`, `requires`, `ensures`) directly into the `erdos-graph/` submodule unless absolutely necessary. We want the main project to remain standard Rust.
2.  **Trust but Verify**: When using `#[verifier::external_body]`, you are telling Verus "Trust me, the code does this". Be very careful with this. Ideally, we want to verify the body, but for `external` code, we often have to assume specs for basic functions to verify higher-level logic.
3.  **Start Small**: Pick small, pure functions or simple structs to verify first before tackling complex async logic.

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

## Running Verus

Run the verifier from the project root:

```bash
make verify
```

If you see `verification results:: X verified, 0 errors`, you're ready to open a PR with your new specs! 
