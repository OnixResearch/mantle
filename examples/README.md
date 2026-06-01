# Mantle examples

This directory is a supported examples gallery. `examples/catalog.ncl` is the source of truth for support tiers, prerequisites, and validation rails.

## Beginner

| File | What it shows | Fast rail |
|---|---|---|
| `examples/hello.ncl` | Smallest derivation; writes a flat hello output with `/bin/sh`. | eval + fast build + output inspection |
| `examples/multi-step.ncl` | Multi-line output using shell builtins only. | eval + fast build + output inspection |
| `examples/mk-hello.ncl` | `mkDerivation` wrapper with generated seed paths. | manual seed eval/build |

## Diagnostics

| File | What it shows | Expected result |
|---|---|---|
| `examples/fail.ncl` | A builder that fails intentionally. | negative build diagnostic |

## Fetcher cookbook

Real-network examples stay useful for operators, but deterministic validation uses generated offline fixtures in `tests/examples_build.rs` for each fetcher helper family.

| File | What it shows | Capability | Offline validation rail |
|---|---|---|---|
| `examples/fetch-file.ncl` | Fixed-output single file fetch from a real URL. | real network | `offline-fetchurl-fixture` + `fixed-output-negative` |
| `examples/fetch-tarball.ncl` | Fixed-output tarball fetch/unpack from a real URL. | real network | `offline-fetch-tarball-fixture` + `fixed-output-negative` |
| `examples/fetch-git.ncl` | Fixed-output git checkout from a real repository. | real network | `offline-fetchgit-fixture` + `fixed-output-negative` |
| `examples/fetch-crate-crc64.ncl` | Fetch the published `crc64` crate source. | real network | `offline-fetch-tarball-fixture` + `fixed-output-negative` |

## Package composition

| File | What it shows | Capability |
|---|---|---|
| `examples/build-from-source.ncl` | Multi-file C project with `make`, library, binary, and install phase. | generated seed |
| `examples/multi-output.ncl` | Named outputs: `out`, `dev`, and `man`. | generated seed |
| `examples/override.ncl` | `overrideAttrs` without rewriting the original package. | generated seed |
| `examples/package-set.ncl` | Related packages in one Nickel package set. | generated seed |

## Project workflow

| File | What it shows | Capability |
|---|---|---|
| `examples/project/crunch.ncl` | Project outputs: default package, named packages, and checks. | generated seed |

Useful commands after generating `examples/project/seed.ncl`:

```bash
cd examples/project
mantle build
mantle build .#hello
mantle run .#hello
mantle build .#checks.test-hello
```

## Advanced bootstrap

| File | What it shows | Capability |
|---|---|---|
| `examples/bootstrap-no-nix.ncl` | Compile C with the shared reduced bootstrap seed provider. | heavy + bwrap |
| `examples/build-crate-crc64.ncl` | Build a real crates.io Rust crate with Mantle's bootstrap Rust toolchain. | heavy + real network + bwrap |
| `examples/hello-static.ncl` | Static C hello with generated seed paths. | generated seed |
| `examples/hello-world.ncl` | C hello with generated seed paths. | generated seed |
| `examples/crunch.ncl` | Self-build derivation structure skeleton. Real proof uses `mantle self-build`. | skeleton / non-claim |
| `examples/seed.ncl` | Generated host seed paths used by seed-dependent examples. | generated support file |

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
