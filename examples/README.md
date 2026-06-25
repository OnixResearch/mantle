# Mantle examples

This directory is a supported examples gallery. `examples/catalog.ncl` is the source of truth for support tiers, prerequisites, and validation rails.

## Beginner

Start here. These examples are local, fast, and do not need generated seed material unless the capability column says so.

| File | Command | Expected output shape | Capability |
|---|---|---|---|
| `examples/hello.ncl` | `mantle build examples/hello.ncl --no-substitute` | flat file containing `Hello, mantle!` | local + fast |
| `examples/multi-step.ncl` | `mantle build examples/multi-step.ncl --no-substitute` | flat file containing `name: multi-step` | local + fast |
| `examples/mk-hello.ncl` | `mantle build examples/mk-hello.ncl -I examples -I builders --no-substitute` | store path containing `bin/hello` | generated seed |

## Diagnostics

| File | What it shows | Expected result |
|---|---|---|
| `examples/fail.ncl` | A builder that fails intentionally. | negative build diagnostic |

## Fetcher cookbook

Real-network examples stay useful for operators, but deterministic validation uses generated offline fixtures in `tests/examples_build.rs` for each fetcher helper family.

| File | Command | Expected output shape | Capability | Offline validation rail |
|---|---|---|---|---|
| `examples/fetch-file.ncl` | `mantle build examples/fetch-file.ncl` | fixed-output file | real network | `offline-fetchurl-fixture` + `fixed-output-negative` |
| `examples/fetch-tarball.ncl` | `mantle build examples/fetch-tarball.ncl` | unpacked source tree | real network | `offline-fetch-tarball-fixture` + `fixed-output-negative` |
| `examples/fetch-git.ncl` | `mantle build examples/fetch-git.ncl` | checkout tree without `.git/` | real network | `offline-fetchgit-fixture` + `fixed-output-negative` |
| `examples/fetch-crate-crc64.ncl` | `mantle build examples/fetch-crate-crc64.ncl` | crate source tree containing `Cargo.toml` | real network | `offline-fetch-tarball-fixture` + `fixed-output-negative` |

## Package composition

After basic derivations and fetchers, move to output layouts and package relationships.

| File | Command | Expected output shape | Capability |
|---|---|---|---|
| `examples/local-output-layout.ncl` | `mantle build examples/local-output-layout.ncl --no-substitute` | named outputs with `bin/show-layout`, `include/local_output_layout.h`, and `share/doc/local-output-layout/README` | local + fast |
| `examples/build-from-source.ncl` | `mantle build examples/build-from-source.ncl -I examples --no-substitute` | installed library and binary | generated seed |
| `examples/multi-output.ncl` | `mantle build examples/multi-output.ncl -I examples --no-substitute` | named outputs: `out`, `dev`, and `man` | generated seed |
| `examples/override.ncl` | `mantle eval examples/override.ncl -I examples -I builders` | overridden derivation metadata | generated seed |
| `examples/package-set.ncl` | `mantle eval examples/package-set.ncl -I examples -I builders` | related package records | generated seed |

## Project workflow

Project examples show selector syntax after package composition. See `examples/project/README.md` for the full command table.

| File | Command | Expected output shape | Capability |
|---|---|---|---|
| `examples/project/crunch.ncl` | `cd examples/project && mantle build` | default package store path with `bin/hello` | generated seed |
| `examples/project/crunch.ncl` | `cd examples/project && mantle build .#hello` | named package store path with `bin/hello` | generated seed |
| `examples/project/crunch.ncl` | `cd examples/project && mantle build .#goodbye` | named package store path with `bin/goodbye` | generated seed |
| `examples/project/crunch.ncl` | `cd examples/project && mantle build .#checks.test-hello` | check output with `result` text `ok` | generated seed |
| `examples/rust_compatibility_rail.rs` | `cargo test -p mantle --test rust_compatibility_rail` | generated representative Rust compatibility rail; sandboxed offline Cargo smoke plus rust-plan bounded success/blocker receipt | fast + bwrap |

The representative Rust compatibility rail is lane-scoped evidence, not proof of
full Cargo compatibility, compiler correctness, release reproducibility, or
bootstrap correctness. The offline rail reports `cargo-inside-mantle-sandbox`;
the native rail reports either `cargo-free-bounded-topology` or a deterministic
`blocked-unsupported-surface` receipt.

## Trust/provenance

These commands inspect local build evidence. They are not release or witness proofs, and placeholder output must not be treated as proof evidence.

| Example or recipe | Command | Expected evidence shape | Non-claim |
|---|---|---|---|
| JSON build report for `examples/hello.ncl` | `mantle --json build examples/hello.ncl --store /tmp/mantle-examples-store --state-dir /tmp/mantle-examples-state --no-substitute` | build-report JSON with `outputs[].artifact_attestation.path` | proves only local build/report shape |
| `examples/crunch.ncl` | `mantle eval examples/crunch.ncl -I examples` | self-build derivation skeleton shape | does not prove release, witness, or fixed-point self-hosting success |

## Advanced bootstrap

| File | Command | Expected output shape | Capability |
|---|---|---|---|
| `examples/bootstrap-no-nix.ncl` | `mantle build examples/bootstrap-no-nix.ncl --no-substitute` | store path containing `bin/hello` | heavy + bwrap |
| `examples/build-crate-crc64.ncl` | `mantle build examples/build-crate-crc64.ncl --no-substitute` | store path containing `bin/crc64` | heavy + real network + bwrap |
| `examples/hello-static.ncl` | `mantle build examples/hello-static.ncl -I examples --no-substitute` | static `hello` binary | generated seed |
| `examples/hello-world.ncl` | `mantle build examples/hello-world.ncl -I examples --no-substitute` | C hello binary | generated seed |
| `examples/seed.ncl` | generated by `mantle bootstrap -o examples/seed.ncl` | host-specific seed paths | generated support file |

## Benchmarks

| File | What it shows | Validation |
|---|---|---|
| `examples/benchmark_eval_smoke.rs` | Cheap evaluation benchmark bundle. | `tests/benchmark_harness.rs` |
| `examples/benchmark_suite.rs` | Full checked-in benchmark workload matrix. | `tests/benchmark_harness.rs` |
| `examples/benchmark_compare.rs` | Compares two benchmark bundles. | `tests/benchmark_harness.rs` |
| `examples/benchmark_eval_backends.rs` | Evaluation backend benchmark. | `tests/benchmark_harness.rs` |
| `examples/benchmark_lazy_eval.rs` | Lazy selected-root evaluation benchmark. | `tests/benchmark_harness.rs` |

## Common commands

```bash
# Evaluate only
mantle eval examples/fetch-crate-crc64.ncl

# Build into writable temp roots
mkdir -p /tmp/mantle-examples-store /tmp/mantle-examples-state

# Fetch a real crate source tarball
mantle build examples/fetch-crate-crc64.ncl \
  --store /tmp/mantle-examples-store \
  --state-dir /tmp/mantle-examples-state

# Build a real Rust crate from crates.io; heavyweight, run explicitly
mantle build examples/build-crate-crc64.ncl \
  --store /tmp/mantle-examples-store \
  --state-dir /tmp/mantle-examples-state \
  --no-substitute
```
