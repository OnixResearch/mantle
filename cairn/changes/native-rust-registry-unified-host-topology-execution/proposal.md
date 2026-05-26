# Proposal: Native Rust registry unified host topology execution

## Summary

Extend Mantle's main `rust-plan --execute-topology` rail so vendored registry-backed `custom-build` and `proc-macro` host units flow through unified topology execution, not only the dedicated host-artifact topology rail.

## Motivation

Mantle can now execute bounded vendored registry library topologies and dedicated registry host-artifact topologies from explicit `native_registry_source_planning` and `native_host_unit_graph_planning` evidence. The remaining main-path gap is the unified topology executor: real consumers will invoke `--execute-topology`, so registry-backed build scripts and proc macros must be gated, ordered, executed, and receipt-bound there without falling back to Cargo orchestration or ambient registry/cache material.

## Scope

- Require ready native registry source facts and ready native host-unit graph facts for registry-backed host producers and affected target consumers on `rust-plan --execute-topology`.
- Execute supported vendored registry-backed `custom-build` and `proc-macro` host producers before target consumers in unified topology order.
- Bind produced host artifacts, build-script metadata, registry source identity/checksum/vendor root/source digest, and target consumer output digests into the unified topology receipt.
- Add positive CLI evidence for local root + vendored registry build-script or proc-macro dependency through `--execute-topology`.
- Add negative CLI evidence that missing, stale, unsupported, or ambient-cache-dependent registry host material blocks before host or target `rustc`.

## Non-goals

- No general Cargo registry compatibility claim.
- No Cargo orchestration for producer, build-script, proc-macro, or target builds.
- No network, index fetch, `$CARGO_HOME`, Cargo registry cache, git checkout, or target-dir fallback.
- No broad build-script/native-link probing beyond explicit bounded metadata surfaces already modeled.
- No version solving or unsupported Cargo feature/target expansion.
