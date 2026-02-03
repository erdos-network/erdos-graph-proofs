# Contributing to Erdos Graph Proofs

We welcome contributions to help verify more parts of the Erdos Graph engine! This document outlines the specific workflow we use, which is slightly unique because we are verifying an existing "external" codebase without modifying it heavily.

## 🏗️ The Verification Pattern

Since `erdos-graph` is a standard Rust project and Verus requires a specific toolchain and syntax, we do **not** verify the `erdos-graph` crate by adding it as a `Cargo.toml` dependency.

Instead, we **include the source files directly** into our verification crate.

### 1. Include the Source File

In `verified/src/lib.rs`, import the module you want to verify using the `#[path]` attribute to point to the submodule file:

```rust
// verified/src/lib.rs

// Include the source file directly from the submodule
#[path = "../../erdos-graph/src/utilities/thread_safe_queue.rs"]
mod thread_safe_queue;
```

### 2. Handle External Types

Since the source code in `erdos-graph` doesn't know about Verus (it doesn't use the `verus!` macro), Verus treats it as "external" code. We cannot write specs directly inside those files.

We use **External Type Specifications** to tell Verus about the types and functions in that external code.

#### Example: Verifying a Struct

If `thread_safe_queue.rs` has:

```rust
// In erdos-graph/src/...
pub struct QueueConfig {
    pub max_queue_size: usize,
}
```

We write a spec in `verified/src/lib.rs` (inside the `verus!` block):

```rust
verus! {

// 1. Tell Verus about the struct layout
#[verifier::external_type_specification]
pub struct ExQueueConfig(crate::thread_safe_queue::QueueConfig);

// 2. Define behavior for functions you can't verify (shim)
// This tells Verus: "Assume default() returns a config with max_queue_size == 10000"
#[verifier::external_body]
pub fn default_queue_config() -> (c: crate::thread_safe_queue::QueueConfig)
    ensures c.max_queue_size == 10000
{
    crate::thread_safe_queue::QueueConfig::default()
}

// 3. Write your proof/test using the shim
fn check_queue_config() {
    let config = default_queue_config();
    assert(config.max_queue_size == 10000);
}

}
```

### 3. Writing Proofs

Once the types are bridged, you can write normal Verus proofs.

- **`proof fn`**: Ghost code used only for verification (no runtime cost).
- **`spec fn`**: Mathematical functions used in specifications.
- **`exec` functions**: Executable code (usually the tests/wrappers calling the external code).

## 🛡️ Guidelines

1.  **Do NOT Modify `erdos-graph` Source**: Avoid adding Verus specific syntax (`verus!`, `requires`, `ensures`) directly into the `erdos-graph/` submodule unless absolutely necessary. We want the main project to remain standard Rust.
2.  **Trust but Verify**: When using `#[verifier::external_body]`, you are telling Verus "Trust me, the code does this". Be very careful. Ideally, we want to verify the body, but for `external` code, we often have to assume specs for basic functions to verify higher-level logic.
3.  **Start Small**: Pick small, pure functions or simple structs to verify first before tackling complex async logic.

## 🔧 Running Verus

Run the verifier from the project root:

```bash
make verify
```

If you see `verification results:: X verified, 0 errors`, you're good!
