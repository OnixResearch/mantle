# Design: Native build topology dev-dependency scope

## Problem

`[dev-dependencies]` can include options that are meaningful only for tests/examples/benches. The normal build topology does not execute those test units, but native package planning previously rejected dev-dependencies with unsupported options before the build topology could proceed.

## Approach

1. Change `native_dev_dependencies` to collect only source-material facts that are explicitly readable from a path dependency or already-declared registry source.
2. Do not call the generic path dependency parser for dev-dependencies in build-mode package planning, because that parser intentionally fails closed on unsupported dependency options used by build dependencies.
3. When deriving native build units, restrict package iteration to package IDs present in the Cargo build unit graph. This keeps dev-only package facts from creating unresolved normal-build dependency edges.
4. Keep the dedicated dev-dependency topology execution path responsible for any future dev/test feature semantics.

## Safety

Normal `--execute-topology` must not execute dev-dependency units or emit the dedicated `native_rust_dev_dependency_test_topology_execution` claim. Unsupported dev/test option surfaces remain outside the normal build claim.
