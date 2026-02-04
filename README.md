# Erdos Graph Proofs

This repository contains formal verification specifications and proofs for [Erdos Graph](https://github.com/erdos-network/erdos-graph/). The goal of this repo is to prove the correctness of critical components within the Erdos Graph engine. Instead of relying solely on unit and integration tests, we use formal verification to ensure that the code behaves correctly for *all* possible inputs and states defined by our specifications.

## Rust Verification with Verus

We use [Verus](https://www.microsoft.com/en-us/research/project/practical-system-verification/) to verify the correctness of our codebase. Verus adds a "ghost" layer to Rust, allowing us to write specifications on what the code *should* do with preconditions, postconditions and invariants. 

## Setup & Usage

This project uses a standalone Verus installation located in `tools/verus-bin`. The `tools` directory includes an installation script.

### Prerequisites
- Linux or macOS
- Rust installed

### Running Verification

To verify all specifications:

```bash
make verify
```

Or manually:

```bash
./tools/verus-bin/verus --crate-type=lib verified/src/lib.rs
```

## Project Structure

- **`erdos-graph/`**: The submodule containing the actual source code we are verifying.
- **`verified/`**: The crate containing the verification logic.
- **`tools/`**: Contains the Verus binaries.

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to write new specifications.
