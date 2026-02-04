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

## 🔧 Running Verus

Run the verifier from the project root:

```bash
make verify
```

If you see `verification results:: X verified, 0 errors`, you're ready to open a PR with your new specs! 
