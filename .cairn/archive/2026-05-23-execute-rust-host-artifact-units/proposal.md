# Change: execute-rust-host-artifact-units

## Problem

Mantle can now execute target-only Rust unit topologies from explicit unit derivation graph evidence, but any graph that consumes build-script or proc-macro host artifacts still blocks before target execution. That is the next major Cargo-compatibility seam because common Rust crates require host-built artifacts before target units can compile.

## Proposed Change

Add a bounded host-artifact execution rail for explicit Rust unit derivation graphs:

- execute supported host units (`proc-macro` and simple `custom-build` build-script units) before target consumers;
- bind host-produced artifacts back into target `consumed_host_artifacts`, derivation inputs, and matching `--extern` surfaces;
- preserve ordered host and target unit execution receipts in one host-artifact topology receipt;
- expose the rail through `mantle rust-plan` JSON evidence with an explicit output root;
- fail closed for missing host producers, failed host execution, missing produced host artifacts, unsupported host/target shapes, or target consumption without declared host material.

The change remains Cargo-free at execution time: Cargo may be used only to capture oracle metadata/unit graph material for planning evidence, not to orchestrate host or target builds.

## Non-Goals

- Full Cargo compatibility.
- General scheduling beyond a bounded host-first topology.
- Native-link probing, doctest/test/example/run modes, or build-script-generated native link semantics beyond explicit captured metadata.
- Trusting ambient Cargo target directories or registry caches as repair inputs.
