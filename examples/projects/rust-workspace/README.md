# Rust workspace project

This dependency-free Cargo workspace contains a library and CLI, with positive and negative unit tests. Mantle builds it through the bounded offline Cargo project lane.

Run source tests directly while developing:

```bash
cargo test --workspace
```

Build through Mantle from this directory:

```bash
mantle build .#source
mantle build
mantle build .#workspace-app
mantle build .#checks.smoke
```

The first Mantle build can realize the source-built Rust, seed-toolchain, and musl inputs. Therefore, this example is heavyweight.

The evidence applies to Cargo running offline inside Mantle's sandbox. It does not prove Cargo-free execution or compiler correctness.
