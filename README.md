# Erdos Graph Proofs

This repository contains formal verification specifications and proofs for the [Erdos Graph](https://github.com/liam-potter/erdos-graph) project, a high-performance graph scraping and ingestion engine.

## 🎯 Goal

The primary goal of this project is to mathematically prove the correctness of critical components within the Erdos Graph engine. Instead of relying solely on unit tests, which check specific scenarios, we use formal verification to ensure that the code behaves correctly for *all* possible inputs and states defined by our specifications.

We focus on verifying:
- **Thread Safety**: Proving that concurrent data structures (like queues) satisfy safety properties.
- **Data Consistency**: Ensuring ingestion logic preserves database invariants.
- **Configuration Logic**: Verifying that defaults and constraints are respected.

## 🛠️ Powered by Verus

We use [Verus](https://github.com/verus-lang/verus), a tool for verifying the correctness of Rust programs.

Verus adds a "ghost" layer to Rust, allowing us to write:
- **Specifications**: What the code *should* do (preconditions, postconditions, invariants).
- **Proofs**: Code that guides the verifier to check that the implementation matches the specification.

Verus analyzes the code statically. If it passes verification, it means there are no violations of the specified properties (e.g., no panics, no integer overflows, no invariant breaks).

## 🚀 Setup & Usage

This project uses a standalone Verus installation located in `tools/verus-bin`.

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

## 📂 Project Structure

- **`erdos-graph/`**: The submodule containing the actual source code we are verifying.
- **`verified/`**: The crate containing the verification logic.
- **`tools/`**: Contains the Verus binaries.

See [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to write new proofs.